use std::fs;
use std::io;
use std::path::{Component, Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use super::{Applied, EffectError, Roots, SkipReason, exists, io_error};
use crate::record::{Record, split_extension};
use crate::stages::{Move, OnConflict};

/// Moves the file of a planned record to its plan under the target, or says where it would go.
///
/// Nothing is overwritten and no link is followed. A path that could leave the source or the target is
/// refused before anything is touched.
pub fn move_file(
    stage: &Move,
    record: &Record,
    roots: &Roots,
    dry_run: bool,
) -> Result<Applied, EffectError> {
    ensure_inside(record.origin())?;
    ensure_inside(record.plan())?;
    let from = roots.source.join(record.origin());
    let mut to = roots.target.join(record.plan());

    if let Some(reason) = skip_reason(&roots.source, record.origin())? {
        return Ok(Applied::Skipped(reason));
    }
    if from == to {
        return Ok(Applied::Unchanged(from));
    }
    if exists(&to) {
        to = settle_conflict(stage, &to)?;
    }
    if dry_run {
        return Ok(Applied::Preview { from, to });
    }

    place(&from, &to)?;
    Ok(Applied::Moved { from, to })
}

/// A relative path made only of names, so joining it to a root cannot leave that root.
fn ensure_inside(path: &Path) -> Result<(), EffectError> {
    let inside = path.components().count() > 0
        && path
            .components()
            .all(|part| matches!(part, Component::Normal(_)));
    if inside {
        Ok(())
    } else {
        Err(EffectError::Unsafe(path.to_path_buf()))
    }
}

/// Walks from the source down to the file without following anything: a link on the way, or a file that
/// is not a regular file, means it is left alone.
fn skip_reason(source: &Path, origin: &Path) -> Result<Option<SkipReason>, EffectError> {
    let mut current = source.to_path_buf();
    let last = origin.components().count() - 1;
    for (position, part) in origin.components().enumerate() {
        current.push(part);
        let metadata = fs::symlink_metadata(&current).map_err(|error| match error.kind() {
            io::ErrorKind::NotFound => EffectError::Missing(current.clone()),
            kind => io_error("inspect", &current, kind),
        })?;
        if metadata.file_type().is_symlink() {
            return Ok(Some(SkipReason::Link));
        }
        if position == last && !metadata.is_file() {
            return Ok(Some(SkipReason::NotAFile));
        }
    }
    Ok(None)
}

/// What to do when the target is taken: refuse, or try once more with the suffix before the extension.
fn settle_conflict(stage: &Move, taken: &Path) -> Result<PathBuf, EffectError> {
    match stage.on_conflict {
        OnConflict::Reject => Err(EffectError::Conflict(taken.to_path_buf())),
        OnConflict::Suffix => {
            let Some(file_name) = taken.file_name().and_then(|name| name.to_str()) else {
                return Err(EffectError::Conflict(taken.to_path_buf()));
            };
            let (name, ext) = split_extension(file_name);
            let suffixed = if ext.is_empty() {
                format!("{name}{}", stage.suffix)
            } else {
                format!("{name}{}.{ext}", stage.suffix)
            };
            let candidate = taken.with_file_name(suffixed);
            if exists(&candidate) {
                Err(EffectError::Conflict(candidate))
            } else {
                Ok(candidate)
            }
        }
    }
}

fn place(from: &Path, to: &Path) -> Result<(), EffectError> {
    if let Some(folder) = to.parent() {
        fs::create_dir_all(folder)
            .map_err(|error| io_error("create the folder", folder, error.kind()))?;
    }
    match put(from, to) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == io::ErrorKind::CrossesDevices => move_across(from, to),
        Err(error) => Err(failed("rename", from, to, &error)),
    }
}

/// Between filesystems the file is handed to `mv` under a temporary name beside its destination, so a
/// move cut short never leaves a file that looks complete, and then renamed into place without replacing
/// anything. The name is new each time, so what an interrupted move left behind never blocks the next one.
fn move_across(from: &Path, to: &Path) -> Result<(), EffectError> {
    let temporary = to.with_file_name(format!(
        ".{}.{}.part",
        to.file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_default(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |elapsed| elapsed.as_nanos())
    ));
    hand_to_mv(from, &temporary)?;
    put(&temporary, to)
        .map_err(|error| io_error("put in place the file kept at", &temporary, error.kind()))
}

/// Runs `mv -n`, which says nothing when it leaves the file where it is, so a file still there afterwards
/// is a move that did not happen.
fn hand_to_mv(from: &Path, temporary: &Path) -> Result<(), EffectError> {
    let status = Command::new("mv")
        .arg("-n")
        .arg("--")
        .arg(from)
        .arg(temporary)
        .status()
        .map_err(|error| io_error("run mv for", from, error.kind()))?;
    if !status.success() {
        return Err(io_error("move with mv", from, io::ErrorKind::Other));
    }
    if fs::symlink_metadata(from).is_ok() {
        return Err(io_error("move with mv", from, io::ErrorKind::AlreadyExists));
    }
    Ok(())
}

/// Puts a file where it is wanted without ever replacing one that is there.
///
/// On Linux the file is renamed with `RENAME_NOREPLACE`, which fails when the destination exists, so
/// nothing can appear between the check and the move and be lost, and a watcher sees the file moved in
/// rather than created and never closed. Where the filesystem cannot do that, and elsewhere, a hard
/// link takes the same care, and a filesystem without hard links renames the file.
pub(crate) fn put(from: &Path, to: &Path) -> io::Result<()> {
    #[cfg(target_os = "linux")]
    {
        use rustix::fs::{CWD, RenameFlags, renameat_with};
        use rustix::io::Errno;

        match renameat_with(CWD, from, CWD, to, RenameFlags::NOREPLACE) {
            Ok(()) => return Ok(()),
            Err(Errno::INVAL | Errno::NOSYS) => {}
            Err(errno) => return Err(errno.into()),
        }
    }
    match fs::hard_link(from, to) {
        Ok(()) => fs::remove_file(from),
        Err(error) if error.kind() == io::ErrorKind::AlreadyExists => Err(error),
        Err(_) => fs::rename(from, to),
    }
}

/// A move that failed: a conflict when something is already at the destination, otherwise the error of the action.
fn failed(action: &'static str, from: &Path, to: &Path, error: &io::Error) -> EffectError {
    match error.kind() {
        io::ErrorKind::AlreadyExists if exists(to) => EffectError::Conflict(to.to_path_buf()),
        kind => io_error(action, from, kind),
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::{hand_to_mv, move_across, put};

    fn scratch(name: &str) -> std::path::PathBuf {
        let folder =
            std::env::temp_dir().join(format!("auto-renamer-put-{}-{name}", std::process::id()));
        let _ = fs::remove_dir_all(&folder);
        fs::create_dir_all(&folder).unwrap();
        folder
    }

    // @behavior MV-018
    #[test]
    fn should_not_replace_a_file_that_appeared_at_the_plan_after_it_was_checked() {
        let folder = scratch("appeared");
        fs::write(folder.join("from"), "new").unwrap();
        fs::write(folder.join("to"), "other").unwrap();

        let put = put(&folder.join("from"), &folder.join("to"));

        assert_eq!(put.unwrap_err().kind(), std::io::ErrorKind::AlreadyExists);
        assert_eq!(fs::read_to_string(folder.join("to")).unwrap(), "other");
        assert_eq!(fs::read_to_string(folder.join("from")).unwrap(), "new");
        let _ = fs::remove_dir_all(folder);
    }

    // @behavior MV-026
    #[test]
    fn should_refuse_a_move_that_mv_does_not_carry_out() {
        let folder = scratch("mv-skipped");
        fs::write(folder.join("from"), "new").unwrap();
        fs::write(folder.join(".to.part"), "someone else's").unwrap();

        let handed = hand_to_mv(&folder.join("from"), &folder.join(".to.part"));

        assert!(handed.is_err());
        assert_eq!(fs::read_to_string(folder.join("from")).unwrap(), "new");
        assert_eq!(
            fs::read_to_string(folder.join(".to.part")).unwrap(),
            "someone else's"
        );
        let _ = fs::remove_dir_all(folder);
    }

    // @behavior MV-027
    #[test]
    fn should_keep_the_file_under_its_temporary_name_when_the_plan_is_taken_at_the_last_step() {
        let folder = scratch("taken-last");
        fs::write(folder.join("from"), "new").unwrap();
        fs::write(folder.join("to"), "other").unwrap();

        let moved = move_across(&folder.join("from"), &folder.join("to"));

        let kept: Vec<_> = fs::read_dir(&folder)
            .unwrap()
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .filter(|path| path.to_string_lossy().contains("/.to."))
            .collect();
        assert_eq!(kept.len(), 1);
        assert!(
            moved
                .unwrap_err()
                .to_string()
                .contains(&*kept[0].to_string_lossy())
        );
        assert_eq!(fs::read_to_string(&kept[0]).unwrap(), "new");
        assert_eq!(fs::read_to_string(folder.join("to")).unwrap(), "other");
        assert!(!folder.join("from").exists());
        let _ = fs::remove_dir_all(folder);
    }
}

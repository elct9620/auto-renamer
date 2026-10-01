//! The filesystem behind the tree a batch is processed in: what it holds, and how a file is put in
//! place without replacing anything, between filesystems too.

use std::fs;
use std::io::{self, Read};
use std::path::Path;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use chrono::{DateTime, Utc};

use crate::effects::{EffectError, Kind, Tree, io_error};

/// The tree the filesystem holds.
pub struct FsTree;

impl Tree for FsTree {
    fn kind(&self, path: &Path) -> io::Result<Kind> {
        let kind = fs::symlink_metadata(path)?.file_type();
        Ok(if kind.is_symlink() {
            Kind::Link
        } else if kind.is_file() {
            Kind::File
        } else if kind.is_dir() {
            Kind::Folder
        } else {
            Kind::Other
        })
    }

    fn modified(&self, path: &Path) -> io::Result<DateTime<Utc>> {
        Ok(DateTime::from(fs::symlink_metadata(path)?.modified()?))
    }

    fn read(&self, path: &Path, limit: u64) -> io::Result<String> {
        let mut text = String::new();
        fs::File::open(path)?
            .take(limit)
            .read_to_string(&mut text)?;
        Ok(text)
    }

    fn files(&self, folder: &Path) -> Vec<String> {
        let Ok(entries) = fs::read_dir(folder) else {
            return Vec::new();
        };
        entries
            .flatten()
            .filter(|entry| entry.file_type().is_ok_and(|kind| kind.is_file()))
            .filter_map(|entry| entry.file_name().into_string().ok())
            .collect()
    }

    fn is_empty(&self, folder: &Path) -> io::Result<bool> {
        Ok(fs::read_dir(folder)?.next().is_none())
    }

    fn place(&self, from: &Path, to: &Path) -> Result<(), EffectError> {
        place(from, to)
    }

    fn remove_folder(&self, folder: &Path) -> io::Result<()> {
        fs::remove_dir(folder)
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

/// Runs `mv -n` onto a temporary name nothing holds, so whatever lies there afterwards is this move's.
/// When `mv` fails with the file still in the source, what it left there is removed, as `mv` removes a
/// copy it could not finish; a `mv` that reports success without the file arriving did not move it.
fn hand_to_mv(from: &Path, temporary: &Path) -> Result<(), EffectError> {
    if fs::symlink_metadata(temporary).is_ok() {
        return Err(io_error("move to", temporary, io::ErrorKind::AlreadyExists));
    }
    let status = Command::new("mv")
        .arg("-n")
        .arg("--")
        .arg(from)
        .arg(temporary)
        .status()
        .map_err(|error| io_error("run mv for", from, error.kind()))?;
    if !status.success() {
        if fs::symlink_metadata(from).is_ok() {
            let _ = fs::remove_file(temporary);
            return Err(io_error("move with mv", from, io::ErrorKind::Other));
        }
        return Err(io_error(
            "finish the move, kept at",
            temporary,
            io::ErrorKind::Other,
        ));
    }
    if fs::symlink_metadata(temporary).is_err() {
        return Err(io_error("move with mv", from, io::ErrorKind::NotFound));
    }
    Ok(())
}

/// Puts a file where it is wanted without ever replacing one that is there.
///
/// On Linux the file is renamed with `RENAME_NOREPLACE`, which fails when the destination exists, so
/// nothing can appear between the check and the move and be lost, and a watcher sees the file moved in
/// rather than created and never closed. Where the filesystem cannot do that, and elsewhere, a hard
/// link takes the same care, and a filesystem without hard links renames the file.
fn put(from: &Path, to: &Path) -> io::Result<()> {
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

fn exists(path: &Path) -> bool {
    fs::symlink_metadata(path).is_ok()
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
    fn should_not_hand_a_taken_temporary_name_to_mv() {
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

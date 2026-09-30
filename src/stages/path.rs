use std::path::{Component, Path, PathBuf};

use super::{Flow, Lift, Stop, refused_by};
use crate::record::{Record, is_usable_file_name};
use crate::template::Template;

pub(super) fn lift(lift: &Lift, mut record: Record) -> Flow {
    let plan = record.plan().to_path_buf();
    let Some(file_name) = plan.file_name() else {
        return Ok(record);
    };
    let folders: Vec<Component<'_>> = plan
        .parent()
        .map(|parent| parent.components().collect())
        .unwrap_or_default();

    let kept = match lift {
        Lift::Levels(levels) => match folders.len().checked_sub(*levels) {
            Some(kept) => kept,
            None => {
                return Err(Stop::rejected(
                    "lift",
                    format!("cannot lift {levels} levels above the source"),
                ));
            }
        },
        Lift::To(pattern) => match folders
            .iter()
            .rposition(|folder| pattern.is_match(folder.as_os_str()))
        {
            Some(nearest) => nearest + 1,
            None => return Ok(record),
        },
    };

    let mut lifted: PathBuf = folders[..kept].iter().collect();
    lifted.push(file_name);
    record.set_plan(lifted);
    Ok(record)
}

pub(super) fn folder(template: &Template, mut record: Record) -> Flow {
    let rendered = template.render(&record).map_err(refused_by("folder"))?;
    let segments: Vec<&str> = rendered.split('/').collect();
    if let Some(unusable) = segments
        .iter()
        .find(|segment| !is_usable_file_name(segment))
    {
        return Err(Stop::rejected(
            "folder",
            format!("`{unusable}` is not a usable folder name"),
        ));
    }

    let plan = record.plan().to_path_buf();
    let Some(file_name) = plan.file_name() else {
        return Ok(record);
    };
    let mut planned = plan.parent().map(Path::to_path_buf).unwrap_or_default();
    planned.extend(segments);
    planned.push(file_name);
    record.set_plan(planned);
    Ok(record)
}

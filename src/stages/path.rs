use std::path::{Component, Path, PathBuf};

use super::{Lift, Outcome};
use crate::record::{Record, is_usable_file_name};
use crate::template::Template;

pub(super) fn lift(lift: &Lift, mut record: Record) -> Outcome {
    let plan = record.plan().to_path_buf();
    let Some(file_name) = plan.file_name() else {
        return Outcome::Continue(record);
    };
    let folders: Vec<Component<'_>> = plan
        .parent()
        .map(|parent| parent.components().collect())
        .unwrap_or_default();

    let kept = match lift {
        Lift::Levels(levels) => match folders.len().checked_sub(*levels) {
            Some(kept) => kept,
            None => {
                return Outcome::rejected(
                    "lift",
                    format!("cannot lift {levels} levels above the source"),
                );
            }
        },
        Lift::To(pattern) => match folders
            .iter()
            .rposition(|folder| pattern.is_match(folder.as_os_str()))
        {
            Some(nearest) => nearest + 1,
            None => return Outcome::Continue(record),
        },
    };

    let mut lifted: PathBuf = folders[..kept].iter().collect();
    lifted.push(file_name);
    record.set_plan(lifted);
    Outcome::Continue(record)
}

pub(super) fn folder(template: &Template, mut record: Record) -> Outcome {
    let rendered = match template.render(&record) {
        Ok(rendered) => rendered,
        Err(error) => return Outcome::rejected("folder", error.to_string()),
    };
    let segments: Vec<&str> = rendered.split('/').collect();
    if let Some(unusable) = segments
        .iter()
        .find(|segment| !is_usable_file_name(segment))
    {
        return Outcome::rejected(
            "folder",
            format!("`{unusable}` is not a usable folder name"),
        );
    }

    let plan = record.plan().to_path_buf();
    let Some(file_name) = plan.file_name() else {
        return Outcome::Continue(record);
    };
    let mut planned = plan.parent().map(Path::to_path_buf).unwrap_or_default();
    planned.extend(segments);
    planned.push(file_name);
    record.set_plan(planned);
    Outcome::Continue(record)
}

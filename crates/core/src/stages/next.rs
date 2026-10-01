use std::path::Path;

use regex::Regex;

use super::{Batch, Flow, Next, Stage, Stop, refused_by, write_field};
use crate::context::Context;
use crate::record::{Record, Value, split_extension};

impl Stage for Next {
    fn name(&self) -> &'static str {
        "next"
    }

    fn run(&self, batch: Batch, context: &mut Context) -> Batch {
        batch.each(|record| apply(self, record, context))
    }
}

fn apply(next: &Next, mut record: Record, context: &mut Context) -> Flow {
    if record.field(&next.into).is_some() {
        return Ok(record);
    }
    let matcher = next
        .like
        .matcher(&record, &next.into)
        .map_err(refused_by("next"))?;

    let folder = record
        .plan()
        .parent()
        .unwrap_or(Path::new(""))
        .to_path_buf();
    let key = (folder, matcher.as_str().to_string());
    // A number handed out in this batch is already past whatever the target holds.
    let last = context
        .last_handed_out(&key)
        .or_else(|| highest_in_target(context, &key.0, &matcher));
    let Some(number) = last.unwrap_or(0).checked_add(1) else {
        return Err(Stop::rejected(
            "next",
            "there is no number left after the highest one",
        ));
    };
    context.hand_out(key, number);

    write_field("next", &mut record, &next.into, Value::Number(number))?;
    Ok(record)
}

/// The highest number among the files of a target folder that are written the way the matcher expects.
fn highest_in_target(context: &mut Context, folder: &Path, matcher: &Regex) -> Option<u64> {
    context
        .files_in(folder)
        .iter()
        .filter_map(|file| {
            let (name, _) = split_extension(file);
            matcher.captures(name)?.get(1)?.as_str().parse::<u64>().ok()
        })
        .max()
}

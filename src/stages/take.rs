use std::path::{Path, PathBuf};

use super::{Outcome, Take, write_field};
use crate::record::{Record, split_extension};

/// What an earlier pipeline made of one file, as far as a take stage needs to know.
pub(crate) struct Earlier {
    pub pipeline: String,
    pub origin: PathBuf,
    pub planned: Option<Record>,
}

/// Copies fields from the earlier file whose name is the longest beginning of this record's name.
pub(super) fn apply(take: &Take, mut record: Record, earlier: &[Earlier]) -> Outcome {
    let own = main_name(record.origin());
    let candidates: Vec<(&Earlier, usize)> = earlier
        .iter()
        .filter(|other| {
            take.from
                .as_ref()
                .is_none_or(|from| &other.pipeline == from)
        })
        .filter_map(|other| {
            let theirs = main_name(&other.origin);
            (!theirs.is_empty() && own.starts_with(theirs)).then_some((other, theirs.len()))
        })
        .collect();

    let longest = candidates.iter().map(|(_, length)| *length).max();
    let best: Vec<&Earlier> = candidates
        .iter()
        .filter(|(_, length)| Some(*length) == longest)
        .map(|(other, _)| *other)
        .collect();

    let sibling = match best.as_slice() {
        [] => {
            return Outcome::rejected(
                "take",
                "there is no earlier file whose name begins this one",
            );
        }
        [only] => match &only.planned {
            Some(sibling) => sibling,
            None => return Outcome::rejected("take", "the file it takes from was not planned"),
        },
        _ => return Outcome::rejected("take", "more than one earlier file could be taken from"),
    };

    for field in &take.fields {
        let Some(value) = sibling.field(field) else {
            return Outcome::rejected("take", format!("the file it takes from has no `{field}`"));
        };
        if let Err(refused) = write_field("take", &mut record, field, value.clone()) {
            return refused;
        }
    }
    Outcome::Continue(record)
}

fn main_name(origin: &Path) -> &str {
    origin
        .file_name()
        .and_then(|name| name.to_str())
        .map_or("", |name| split_extension(name).0)
}

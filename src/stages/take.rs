use std::collections::HashMap;
use std::path::{Path, PathBuf};

use super::{Outcome, Take, write_field};
use crate::record::{Record, split_extension};

/// What an earlier pipeline made of one file, as far as a take stage needs to know.
pub(crate) struct Earlier {
    pub pipeline: String,
    pub origin: PathBuf,
    pub planned: Option<Record>,
}

/// The files earlier pipelines handled, found by the main file name they begin with.
pub(crate) struct EarlierFiles {
    files: Vec<Earlier>,
    by_name: HashMap<String, Vec<usize>>,
}

impl EarlierFiles {
    pub(crate) fn new(files: Vec<Earlier>) -> EarlierFiles {
        let mut by_name: HashMap<String, Vec<usize>> = HashMap::new();
        for (position, file) in files.iter().enumerate() {
            let name = main_name(&file.origin);
            if !name.is_empty() {
                by_name.entry(name.to_string()).or_default().push(position);
            }
        }
        EarlierFiles { files, by_name }
    }

    /// The files, from the pipeline asked for, whose main name is the longest beginning of `own`.
    fn longest_beginning(&self, own: &str, from: Option<&String>) -> Vec<&Earlier> {
        let ends = own
            .char_indices()
            .map(|(position, character)| position + character.len_utf8());
        for end in ends.collect::<Vec<_>>().into_iter().rev() {
            let found: Vec<&Earlier> = self
                .by_name
                .get(&own[..end])
                .into_iter()
                .flatten()
                .map(|position| &self.files[*position])
                .filter(|file| from.is_none_or(|from| &file.pipeline == from))
                .collect();
            if !found.is_empty() {
                return found;
            }
        }
        Vec::new()
    }
}

/// Copies fields from the earlier file whose name is the longest beginning of this record's name.
pub(super) fn apply(take: &Take, mut record: Record, earlier: &EarlierFiles) -> Outcome {
    let own = main_name(record.origin());
    let best = earlier.longest_beginning(own, take.from.as_ref());

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

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use super::{Batch, Flow, Stage, Stop, Take, refused_by, write_field};
use crate::context::Context;
use crate::record::{Record, split_extension};

/// What an earlier pipeline made of one file, as far as a take stage needs to know.
#[derive(Clone)]
pub(crate) struct Earlier {
    pub pipeline: String,
    pub origin: PathBuf,
    pub planned: Option<Record>,
}

/// The files earlier pipelines handled, found by the main file name they begin with.
struct EarlierFiles {
    files: Vec<Earlier>,
    by_name: HashMap<String, Vec<usize>>,
}

impl EarlierFiles {
    fn new(files: Vec<Earlier>) -> EarlierFiles {
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

impl Stage for Take {
    fn name(&self) -> &'static str {
        "take"
    }

    fn run(&self, batch: Batch, context: &mut Context) -> Batch {
        let earlier = EarlierFiles::new(context.earlier().to_vec());
        batch.each(|record| self.copy_into(record, &earlier))
    }
}

impl Take {
    /// Copies fields from what earlier pipelines planned for the file whose name is the longest
    /// beginning of this record's name.
    fn copy_into(&self, mut record: Record, earlier: &EarlierFiles) -> Flow {
        let own = main_name(record.origin());
        let best = earlier.longest_beginning(own, self.from.as_ref());

        let sibling = match best.as_slice() {
            [] => Err("there is no earlier file whose name begins this one"),
            [only] => only
                .planned
                .as_ref()
                .ok_or("the file it takes from was not planned"),
            _ => Err("more than one earlier file could be taken from"),
        }
        .map_err(refused_by("take"))?;

        for field in &self.fields {
            let value = sibling.field(field).ok_or_else(|| {
                Stop::rejected("take", format!("the file it takes from has no `{field}`"))
            })?;
            write_field("take", &mut record, field, value.clone())?;
        }
        Ok(record)
    }
}

fn main_name(origin: &Path) -> &str {
    origin
        .file_name()
        .and_then(|name| name.to_str())
        .map_or("", |name| split_extension(name).0)
}

use std::collections::BTreeMap;
use std::path::Path;

use super::{Outcome, Rank, refuse_unreadable, write_field};
use crate::record::{Record, Value};

impl Rank {
    /// Numbers the records of a batch among themselves, answering once for each record in its order.
    /// A file whose name is not valid UTF-8 is refused and takes no part in the numbering.
    pub(crate) fn run(&self, records: Vec<Record>) -> Vec<Outcome> {
        let (readable, unreadable): (Vec<_>, Vec<_>) = records
            .into_iter()
            .enumerate()
            .partition(|(_, record)| record.is_readable());
        let (positions, readable): (Vec<usize>, Vec<Record>) = readable.into_iter().unzip();
        let mut outcomes: Vec<(usize, Outcome)> =
            positions.into_iter().zip(apply(self, readable)).collect();
        outcomes.extend(unreadable.into_iter().filter_map(|(position, record)| {
            refuse_unreadable("rank", &record).map(|refused| (position, refused))
        }));
        outcomes.sort_by_key(|(position, _)| *position);
        outcomes.into_iter().map(|(_, outcome)| outcome).collect()
    }
}

/// Numbers the records among those whose `by` fields agree, in an order the preference can bend;
/// a record alone in its group is left without a number. The answers come in the order of the records.
fn apply(rank: &Rank, records: Vec<Record>) -> Vec<Outcome> {
    let keys: Vec<Result<Vec<String>, String>> = records
        .iter()
        .map(|record| group_key(record, &rank.by))
        .collect();

    let mut groups: BTreeMap<&Vec<String>, Vec<usize>> = BTreeMap::new();
    for (index, key) in keys.iter().enumerate() {
        if let Ok(key) = key {
            groups.entry(key).or_default().push(index);
        }
    }

    let mut numbers: Vec<Option<u64>> = vec![None; records.len()];
    for mut members in groups.into_values().filter(|members| members.len() > 1) {
        members.sort_by_cached_key(|&index| {
            (
                preference(rank, records[index].origin()),
                records[index].origin().to_path_buf(),
            )
        });
        for (position, index) in members.into_iter().enumerate() {
            numbers[index] = Some(position as u64 + 1);
        }
    }

    records
        .into_iter()
        .zip(keys)
        .zip(numbers)
        .map(|((mut record, key), number)| match (key, number) {
            (Err(reason), _) => Outcome::rejected("rank", reason),
            (Ok(_), None) => Outcome::Continue(record),
            (Ok(_), Some(number)) => {
                match write_field("rank", &mut record, &rank.into, Value::Number(number)) {
                    Ok(()) => Outcome::Continue(record),
                    Err(refused) => refused,
                }
            }
        })
        .collect()
}

fn group_key(record: &Record, by: &[String]) -> Result<Vec<String>, String> {
    by.iter()
        .map(|field| {
            record
                .field(field)
                .and_then(Value::written)
                .ok_or_else(|| format!("the field `{field}` has no value"))
        })
        .collect()
}

/// The place of the first preferred word the file name holds, without regard to case; names holding none come last.
fn preference(rank: &Rank, origin: &Path) -> usize {
    let name = origin
        .file_name()
        .map(|name| name.to_string_lossy().to_lowercase())
        .unwrap_or_default();
    rank.prefer
        .iter()
        .position(|word| name.contains(&word.to_lowercase()))
        .unwrap_or(rank.prefer.len())
}

use std::collections::BTreeMap;
use std::path::Path;

use super::{Batch, Rank, Run, Stop, unfit, write_field};
use crate::context::Context;
use crate::record::{Record, Value};

impl Run for Rank {
    fn name(&self) -> &'static str {
        "rank"
    }

    fn run(&self, batch: &mut Batch, _: &mut Context) {
        let mut numbers = self.number(batch.live().collect()).into_iter();
        batch.each(|mut record| {
            let number = numbers
                .next()
                .expect("a rank answers once for each record")?;
            if let Some(number) = number {
                write_field("rank", &mut record, &self.into, Value::Number(number))?;
            }
            Ok(record)
        });
    }
}

impl Rank {
    /// The number of each record among those whose `by` fields agree, in an order the preference can
    /// bend; a record alone in its group has none. The answers come in the order of the records.
    fn number(&self, records: Vec<&Record>) -> Vec<Result<Option<u64>, Stop>> {
        let keys: Vec<Result<Vec<String>, Stop>> = records
            .iter()
            .map(|record| group_key(record, &self.by))
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
                    preference(self, records[index].origin()),
                    records[index].origin().to_path_buf(),
                )
            });
            for (position, index) in members.into_iter().enumerate() {
                numbers[index] = Some(position as u64 + 1);
            }
        }

        keys.into_iter()
            .zip(numbers)
            .map(|(key, number)| key.map(|_| number))
            .collect()
    }
}

fn group_key(record: &Record, by: &[String]) -> Result<Vec<String>, Stop> {
    by.iter()
        .map(|field| {
            record
                .field(field)
                .and_then(Value::written)
                .ok_or_else(|| unfit("rank", field, "has no value"))
        })
        .collect()
}

/// The place of the first preferred word the file name holds, without regard to case; names holding none come last.
fn preference(rank: &Rank, origin: &Path) -> usize {
    let name = origin
        .file_name()
        .and_then(|name| name.to_str())
        .map(str::to_lowercase)
        .unwrap_or_default();
    rank.prefer
        .iter()
        .position(|word| name.contains(&word.to_lowercase()))
        .unwrap_or(rank.prefer.len())
}

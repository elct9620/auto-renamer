//! Reads the keys of one TOML table, for the configuration and the stage declarations alike.

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::time::Duration;

use toml::{Table, Value as Toml};

use crate::record::Value;

/// Where a table sits, which is what an error about one of its keys is made from.
pub(crate) trait Scope {
    type Error;

    /// A key whose value cannot be used.
    fn invalid(&self, key: &str, reason: String) -> Self::Error;

    /// A key the table may not hold.
    fn unknown(&self, key: &str) -> Self::Error;
}

/// The keys of one table, taken out one by one so that whatever is left over is a mistake.
pub(crate) struct Reader<S> {
    scope: S,
    table: Table,
}

impl<S: Scope> Reader<S> {
    pub(crate) fn new(scope: S, table: Table) -> Reader<S> {
        Reader { scope, table }
    }

    pub(crate) fn invalid(&self, key: &str, reason: impl Into<String>) -> S::Error {
        self.scope.invalid(key, reason.into())
    }

    pub(crate) fn take(&mut self, key: &str) -> Option<Toml> {
        self.table.remove(key)
    }

    pub(crate) fn table(&mut self, key: &str) -> Result<Option<Table>, S::Error> {
        match self.take(key) {
            None => Ok(None),
            Some(Toml::Table(table)) => Ok(Some(table)),
            Some(_) => Err(self.invalid(key, "must be a table")),
        }
    }

    pub(crate) fn string(&mut self, key: &str) -> Result<Option<String>, S::Error> {
        match self.take(key) {
            None => Ok(None),
            Some(Toml::String(text)) => Ok(Some(text)),
            Some(_) => Err(self.invalid(key, "must be text")),
        }
    }

    pub(crate) fn required_string(&mut self, key: &str) -> Result<String, S::Error> {
        self.string(key)?
            .ok_or_else(|| self.invalid(key, "is required"))
    }

    pub(crate) fn strings(&mut self, key: &str) -> Result<Option<Vec<String>>, S::Error> {
        match self.take(key) {
            None => Ok(None),
            Some(Toml::Array(items)) => items
                .into_iter()
                .map(|item| match item {
                    Toml::String(text) => Ok(text),
                    _ => Err(self.invalid(key, "must be a list of text")),
                })
                .collect::<Result<Vec<_>, _>>()
                .map(Some),
            Some(_) => Err(self.invalid(key, "must be a list of text")),
        }
    }

    pub(crate) fn boolean(&mut self, key: &str) -> Result<Option<bool>, S::Error> {
        match self.take(key) {
            None => Ok(None),
            Some(Toml::Boolean(flag)) => Ok(Some(flag)),
            Some(_) => Err(self.invalid(key, "must be true or false")),
        }
    }

    pub(crate) fn integer(&mut self, key: &str) -> Result<Option<i64>, S::Error> {
        match self.take(key) {
            None => Ok(None),
            Some(Toml::Integer(number)) => Ok(Some(number)),
            Some(_) => Err(self.invalid(key, "must be a whole number")),
        }
    }

    pub(crate) fn positive(&mut self, key: &str) -> Result<Option<usize>, S::Error> {
        match self.take(key) {
            None => Ok(None),
            Some(Toml::Integer(number)) if number >= 1 => usize::try_from(number)
                .map(Some)
                .map_err(|_| self.invalid(key, "is too large")),
            Some(_) => Err(self.invalid(key, "must be a positive whole number")),
        }
    }

    /// An absolute path written without `.` or `..`, so that comparing it with another is exact.
    pub(crate) fn absolute_path(&mut self, key: &str) -> Result<Option<PathBuf>, S::Error> {
        match self.string(key)? {
            None => Ok(None),
            Some(text) if text.split('/').any(|part| part == "." || part == "..") => {
                Err(self.invalid(key, "may not hold `.` or `..`"))
            }
            Some(text) if text.starts_with('/') => Ok(Some(PathBuf::from(text))),
            Some(_) => Err(self.invalid(key, "must be an absolute path")),
        }
    }

    /// A length of time written as a whole number and a unit: `s`, `m` or `h`.
    pub(crate) fn duration(&mut self, key: &str) -> Result<Option<Duration>, S::Error> {
        let Some(text) = self.string(key)? else {
            return Ok(None);
        };
        let unit_seconds = match text.chars().last() {
            Some('s') => 1,
            Some('m') => 60,
            Some('h') => 3600,
            _ => return Err(self.invalid(key, "needs a unit: s, m or h")),
        };
        let digits = &text[..text.len() - 1];
        match digits
            .parse::<u64>()
            .ok()
            .filter(|count| *count > 0)
            .and_then(|count| count.checked_mul(unit_seconds))
        {
            Some(seconds) => Ok(Some(Duration::from_secs(seconds))),
            None => Err(self.invalid(key, "must be a positive whole number followed by s, m or h")),
        }
    }

    /// Every key of the table as a fixed field value: text, or a whole number.
    pub(crate) fn into_values(self) -> Result<BTreeMap<String, Value>, S::Error> {
        let mut values = BTreeMap::new();
        for (name, value) in &self.table {
            let value = match value {
                Toml::String(text) => Value::Text(text.clone()),
                Toml::Integer(number) if *number >= 0 => Value::Number(*number as u64),
                _ => return Err(self.invalid(name, "must be text or a whole number")),
            };
            values.insert(name.clone(), value);
        }
        Ok(values)
    }

    pub(crate) fn finish(self) -> Result<(), S::Error> {
        match self.table.keys().next() {
            None => Ok(()),
            Some(key) => Err(self.scope.unknown(key)),
        }
    }
}

use std::path::PathBuf;
use std::time::Duration;

use toml::{Table, Value as Toml};

use super::ConfigError;

/// The keys of one table of the configuration, taken out one by one so that whatever is left over is a mistake.
pub(super) struct Reader {
    scope: String,
    table: Table,
}

impl Reader {
    pub(super) fn new(scope: impl Into<String>, table: Table) -> Reader {
        Reader {
            scope: scope.into(),
            table,
        }
    }

    pub(super) fn from_value(
        scope: impl Into<String>,
        value: &Toml,
    ) -> Result<Reader, ConfigError> {
        let scope = scope.into();
        match value {
            Toml::Table(table) => Ok(Reader::new(scope, table.clone())),
            _ => Err(invalid(&scope, &scope, "must be a table")),
        }
    }

    pub(super) fn invalid(&self, key: &str, reason: impl Into<String>) -> ConfigError {
        invalid(&self.scope, key, reason)
    }

    pub(super) fn take(&mut self, key: &str) -> Option<Toml> {
        self.table.remove(key)
    }

    pub(super) fn table(&mut self, key: &str) -> Result<Option<Table>, ConfigError> {
        match self.take(key) {
            None => Ok(None),
            Some(Toml::Table(table)) => Ok(Some(table)),
            Some(_) => Err(self.invalid(key, "must be a table")),
        }
    }

    pub(super) fn string(&mut self, key: &str) -> Result<Option<String>, ConfigError> {
        match self.take(key) {
            None => Ok(None),
            Some(Toml::String(text)) => Ok(Some(text)),
            Some(_) => Err(self.invalid(key, "must be text")),
        }
    }

    pub(super) fn strings(&mut self, key: &str) -> Result<Option<Vec<String>>, ConfigError> {
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

    pub(super) fn boolean(&mut self, key: &str) -> Result<Option<bool>, ConfigError> {
        match self.take(key) {
            None => Ok(None),
            Some(Toml::Boolean(flag)) => Ok(Some(flag)),
            Some(_) => Err(self.invalid(key, "must be true or false")),
        }
    }

    pub(super) fn positive(&mut self, key: &str) -> Result<Option<usize>, ConfigError> {
        match self.take(key) {
            None => Ok(None),
            Some(Toml::Integer(number)) if number >= 1 => usize::try_from(number)
                .map(Some)
                .map_err(|_| self.invalid(key, "is too large")),
            Some(_) => Err(self.invalid(key, "must be a positive whole number")),
        }
    }

    pub(super) fn absolute_path(&mut self, key: &str) -> Result<Option<PathBuf>, ConfigError> {
        match self.string(key)? {
            None => Ok(None),
            Some(text) if text.starts_with('/') => Ok(Some(PathBuf::from(text))),
            Some(_) => Err(self.invalid(key, "must be an absolute path")),
        }
    }

    /// A length of time written as a whole number and a unit: `s`, `m` or `h`.
    pub(super) fn duration(&mut self, key: &str) -> Result<Option<Duration>, ConfigError> {
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

    pub(super) fn finish(self) -> Result<(), ConfigError> {
        match self.table.keys().next() {
            None => Ok(()),
            Some(key) => Err(ConfigError::Unknown {
                scope: self.scope.clone(),
                key: key.clone(),
            }),
        }
    }
}

pub(super) fn invalid(scope: &str, key: &str, reason: impl Into<String>) -> ConfigError {
    ConfigError::Invalid {
        scope: scope.to_string(),
        key: key.to_string(),
        reason: reason.into(),
    }
}

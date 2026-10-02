use std::collections::btree_map::Entry;
use std::collections::{BTreeMap, BTreeSet};
use std::ffi::OsStr;
use std::fmt;
use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};

/// What a field holds.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value {
    Text(String),
    Number(u64),
    Date(DateTime<Utc>),
}

impl Value {
    /// The value as it is written into a name: text as it is, a number in digits, a date not at all.
    pub(crate) fn written(&self) -> Option<String> {
        match self {
            Value::Text(text) => Some(text.clone()),
            Value::Number(number) => Some(number.to_string()),
            Value::Date(_) => None,
        }
    }
}

/// Why no record was made of a file: its path is not valid UTF-8.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NotUtf8;

impl fmt::Display for NotUtf8 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("the file name is not valid UTF-8")
    }
}

impl std::error::Error for NotUtf8 {}

/// One file as it moves through a pipeline: a plan path and its named fields.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Record {
    origin: PathBuf,
    plan: PathBuf,
    fields: BTreeMap<String, Value>,
    /// The fields `vars` gave, which detection never overwrites.
    answers: BTreeSet<String>,
}

impl Record {
    /// Creates the record of a file from its path relative to the source and its modification time.
    ///
    /// A path that is not valid UTF-8 makes no record, since text made from it would stand for a
    /// different file.
    pub fn new(path: &Path, mtime: DateTime<Utc>) -> Result<Record, NotUtf8> {
        if path.to_str().is_none() {
            return Err(NotUtf8);
        }
        let file_name = text(path.file_name());
        let (name, ext) = split_extension(&file_name);
        let folder = path.parent().unwrap_or(Path::new(""));
        let fields = BTreeMap::from([
            ("name".to_string(), Value::Text(name.to_string())),
            ("ext".to_string(), Value::Text(ext.to_string())),
            ("dir".to_string(), Value::Text(text(folder.file_name()))),
            (
                "path".to_string(),
                Value::Text(text(Some(folder.as_os_str()))),
            ),
            ("mtime".to_string(), Value::Date(mtime)),
        ]);

        Ok(Record {
            origin: path.to_path_buf(),
            plan: path.to_path_buf(),
            fields,
            answers: BTreeSet::new(),
        })
    }

    /// Adds variables as fields, each an answer; a built-in field is never replaced.
    pub fn with_vars(mut self, vars: BTreeMap<String, Value>) -> Record {
        for (name, value) in vars {
            if let Entry::Vacant(field) = self.fields.entry(name.clone()) {
                field.insert(value);
                self.answers.insert(name);
            }
        }
        self
    }

    /// The record as a rejected route takes it: the fields the path gives start over from the origin, so
    /// the file keeps its own name and place, while every other field stays, with the plan it had as
    /// `planned` and the stage that refused it as `reason`.
    pub(crate) fn restarted(self, planned: &Path, reason: &str) -> Record {
        let mtime = match self.fields.get("mtime") {
            Some(Value::Date(mtime)) => *mtime,
            _ => DateTime::<Utc>::UNIX_EPOCH,
        };
        let mut restarted = Record::new(&self.origin, mtime)
            .expect("a record was made from this origin, so its path is valid UTF-8");
        for (name, value) in self.fields {
            restarted.fields.entry(name).or_insert(value);
        }
        restarted.answers = self.answers;
        restarted.fields.insert(
            "planned".to_string(),
            Value::Text(planned.display().to_string()),
        );
        restarted
            .fields
            .insert("reason".to_string(), Value::Text(reason.to_string()));
        restarted
    }

    /// Whether `vars` gave this field.
    pub(crate) fn is_answer(&self, name: &str) -> bool {
        self.answers.contains(name)
    }

    /// The path the record was made from, which no stage rewrites.
    pub fn origin(&self) -> &Path {
        &self.origin
    }

    /// The path the file is planned to be moved to, relative to the source.
    pub fn plan(&self) -> &Path {
        &self.plan
    }

    pub fn set_plan(&mut self, plan: PathBuf) {
        self.plan = plan;
    }

    pub fn field(&self, name: &str) -> Option<&Value> {
        self.fields.get(name)
    }

    /// Every field with its value, in the order of their names.
    pub fn fields(&self) -> impl Iterator<Item = (&str, &Value)> {
        self.fields
            .iter()
            .map(|(name, value)| (name.as_str(), value))
    }

    /// Writes a field, replacing any earlier value. The file name of the plan is always `name` and `ext`
    /// put together, so writing either one renames the plan; a result that cannot be a file name leaves the plan as it was.
    pub fn set_field(&mut self, name: &str, value: Value) {
        self.fields.insert(name.to_string(), value);
        if name == "name" || name == "ext" {
            self.sync_file_name();
        }
    }

    fn sync_file_name(&mut self) {
        let part = |field: &str| {
            self.fields
                .get(field)
                .and_then(Value::written)
                .unwrap_or_default()
        };
        let (name, ext) = (part("name"), part("ext"));
        let file_name = if ext.is_empty() {
            name
        } else {
            format!("{name}.{ext}")
        };
        if is_usable_file_name(&file_name) {
            self.plan.set_file_name(file_name);
        }
    }
}

/// A file name is one path component: not empty, not `.` or `..`, and without a separator or NUL.
pub(crate) fn is_usable_file_name(name: &str) -> bool {
    !name.is_empty() && name != "." && name != ".." && !name.contains(['/', '\0'])
}

/// The extension is what follows the last dot when it is 1 to 5 ASCII letters or digits
/// and the dot does not lead the name.
pub(crate) fn split_extension(file_name: &str) -> (&str, &str) {
    match file_name.rfind('.') {
        Some(dot) if dot > 0 => {
            let ext = &file_name[dot + 1..];
            let plausible =
                (1..=5).contains(&ext.len()) && ext.bytes().all(|b| b.is_ascii_alphanumeric());
            if plausible {
                (&file_name[..dot], ext)
            } else {
                (file_name, "")
            }
        }
        _ => (file_name, ""),
    }
}

/// The text of a part of a path that is valid UTF-8 as a whole.
fn text(part: Option<&OsStr>) -> String {
    part.and_then(OsStr::to_str).unwrap_or_default().to_string()
}

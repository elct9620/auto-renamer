use std::collections::BTreeMap;
use std::ffi::OsStr;
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

/// One file as it moves through a pipeline: a plan path and its named fields.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Record {
    plan: PathBuf,
    fields: BTreeMap<String, Value>,
}

impl Record {
    /// Creates the record of a file from its path relative to the source and its modification time.
    pub fn new(path: &Path, mtime: DateTime<Utc>) -> Record {
        let file_name = lossy(path.file_name());
        let (name, ext) = split_extension(&file_name);
        let folder = path.parent().unwrap_or(Path::new(""));

        let fields = BTreeMap::from([
            ("name".to_string(), Value::Text(name.to_string())),
            ("ext".to_string(), Value::Text(ext.to_string())),
            ("dir".to_string(), Value::Text(lossy(folder.file_name()))),
            (
                "path".to_string(),
                Value::Text(folder.to_string_lossy().into_owned()),
            ),
            ("mtime".to_string(), Value::Date(mtime)),
        ]);

        Record {
            plan: path.to_path_buf(),
            fields,
        }
    }

    /// Adds variables as fields; a built-in field is never replaced.
    pub fn with_vars(mut self, vars: BTreeMap<String, Value>) -> Record {
        for (name, value) in vars {
            self.fields.entry(name).or_insert(value);
        }
        self
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
fn split_extension(file_name: &str) -> (&str, &str) {
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

fn lossy(name: Option<&OsStr>) -> String {
    name.map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_default()
}

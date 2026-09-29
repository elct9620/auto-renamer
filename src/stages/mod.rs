//! The stages a pipeline is stacked from, as declared in configuration.

mod declare;
mod filter;
mod format;
mod next;
mod number;
mod path;
mod pattern;
mod rank;
mod take;
mod text;

use std::borrow::Cow;
use std::collections::BTreeMap;

use globset::GlobMatcher;

use crate::context::Context;
use crate::record::{Record, Value, is_usable_file_name};
use crate::template::Template;

pub use declare::DeclareError;
pub(crate) use take::{Earlier, EarlierFiles};

/// What a stage does with one record: pass it on, exclude it, or refuse it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outcome {
    Continue(Record),
    Excluded,
    Rejected(Rejection),
}

/// Why a stage refused a record, and which stage it was.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rejection {
    pub stage: String,
    pub reason: String,
}

impl Outcome {
    fn rejected(stage: &str, reason: impl Into<String>) -> Outcome {
        Outcome::Rejected(Rejection {
            stage: stage.to_string(),
            reason: reason.into(),
        })
    }
}

/// Writes a field of the record; a `name` or `ext` that would not make a usable file name is refused.
fn write_field(stage: &str, record: &mut Record, field: &str, value: Value) -> Result<(), Outcome> {
    let written = value.written();
    let usable = match (field, written.as_deref()) {
        ("name", Some(name)) => is_usable_file_name(name),
        ("ext", Some(ext)) => ext.is_empty() || !ext.contains(['/', '\0']),
        _ => true,
    };
    if !usable {
        let shown = written.unwrap_or_default();
        return Err(Outcome::rejected(
            stage,
            format!("`{shown}` is not usable as the {field}"),
        ));
    }
    record.set_field(field, value);
    Ok(())
}

/// The text of a field, numbers written out; a date or a missing field says why it has none.
fn field_text<'a>(record: &'a Record, name: &str) -> Result<Cow<'a, str>, String> {
    match record.field(name) {
        Some(Value::Text(text)) => Ok(Cow::Borrowed(text)),
        Some(Value::Number(number)) => Ok(Cow::Owned(number.to_string())),
        Some(Value::Date(_)) => Err(format!("the field `{name}` is a date")),
        None => Err(format!("the field `{name}` does not exist")),
    }
}

/// Fixed values written into records by `set` and `default`.
pub type Fields = BTreeMap<String, Value>;

/// One small function of a pipeline, declared with its parameters.
#[derive(Debug, Clone)]
pub enum Stage {
    Filter(Filter),
    Number(Number),
    Regex(Pattern),
    Set(Fields),
    Default(Fields),
    Replace(Replace),
    Case(Case),
    Strip(Strip),
    Format(Template),
    Folder(Template),
    Lift(Lift),
    Next(Next),
    Rank(Rank),
    Take(Take),
    Move(Move),
    Cleanup(Cleanup),
}

impl Stage {
    /// The name the stage is declared by.
    pub fn name(&self) -> &'static str {
        match self {
            Stage::Filter(_) => "filter",
            Stage::Number(_) => "number",
            Stage::Regex(_) => "regex",
            Stage::Set(_) => "set",
            Stage::Default(_) => "default",
            Stage::Replace(_) => "replace",
            Stage::Case(_) => "case",
            Stage::Strip(_) => "strip",
            Stage::Format(_) => "format",
            Stage::Folder(_) => "folder",
            Stage::Lift(_) => "lift",
            Stage::Next(_) => "next",
            Stage::Rank(_) => "rank",
            Stage::Take(_) => "take",
            Stage::Move(_) => "move",
            Stage::Cleanup(_) => "cleanup",
        }
    }

    /// Runs the stage on one record, with what the batch and the target let it know.
    pub fn apply(&self, record: Record, context: &mut Context) -> Outcome {
        match self {
            Stage::Filter(filter) => filter::apply(filter, record),
            Stage::Number(number) => number::apply(number, record),
            Stage::Regex(pattern) => pattern::apply(pattern, record),
            Stage::Set(fields) => text::set(fields, record),
            Stage::Default(fields) => text::default(fields, record),
            Stage::Replace(replace) => text::replace(replace, record),
            Stage::Case(case) => text::case(case, record),
            Stage::Strip(strip) => text::strip(strip, record),
            Stage::Format(template) => format::apply(template, record),
            Stage::Lift(lift) => path::lift(lift, record),
            Stage::Folder(template) => path::folder(template, record),
            Stage::Next(next) => next::apply(next, record, context),
            Stage::Rank(_) | Stage::Take(_) => {
                Outcome::rejected(self.name(), "works on a whole batch")
            }
            other => Outcome::rejected(other.name(), "is not available yet"),
        }
    }

    /// Numbers the records of a batch among themselves; only a `rank` stage does anything here.
    pub(crate) fn rank_batch(&self, records: Vec<Record>) -> Option<Vec<Outcome>> {
        match self {
            Stage::Rank(rank) => Some(rank::apply(rank, records)),
            _ => None,
        }
    }

    /// Takes fields from what earlier pipelines planned; any stage but `take` refuses.
    pub(crate) fn take_from(&self, record: Record, earlier: &EarlierFiles) -> Outcome {
        match self {
            Stage::Take(take) => take::apply(take, record, earlier),
            other => Outcome::rejected(other.name(), "does not take from other files"),
        }
    }

    /// Whether the stage touches the filesystem, and so must come after every stage that only rewrites the plan.
    pub fn is_effect(&self) -> bool {
        matches!(self, Stage::Move(_) | Stage::Cleanup(_))
    }

    /// Whether the stage rewrites the folders of the plan.
    pub fn is_path(&self) -> bool {
        matches!(self, Stage::Lift(_) | Stage::Folder(_))
    }
}

#[derive(Debug, Clone)]
pub struct Filter {
    /// Lower-cased extensions, written without the dot.
    pub ext: Vec<String>,
    pub glob: Option<GlobMatcher>,
    pub invert: bool,
}

#[derive(Debug, Clone)]
pub struct Number {
    pub from: String,
    pub into: String,
    pub nth: Option<i64>,
    pub prefix: Option<Prefix>,
    pub exclude: Vec<String>,
}

/// A word that the number to read must follow.
#[derive(Debug, Clone)]
pub struct Prefix {
    pub word: String,
    pattern: regex::Regex,
}

impl Prefix {
    pub(crate) fn new(word: &str) -> Result<Prefix, regex::Error> {
        let pattern = regex::Regex::new(&format!(
            r"(?i)\b{}[\s._-]*([0-9]{{1,3}})",
            regex::escape(word)
        ))?;
        Ok(Prefix {
            word: word.to_string(),
            pattern,
        })
    }
}

#[derive(Debug, Clone)]
pub struct Pattern {
    pub pattern: regex::Regex,
    pub from: String,
    pub into: Option<String>,
    pub replace: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Replace {
    pub find: String,
    pub with: String,
    pub field: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CaseKind {
    Lower,
    Upper,
    Title,
}

#[derive(Debug, Clone)]
pub struct Case {
    pub to: CaseKind,
    pub field: String,
}

#[derive(Debug, Clone)]
pub struct Strip {
    /// Opening and closing characters of each bracket group to remove.
    pub groups: Vec<(char, char)>,
    pub field: String,
}

#[derive(Debug, Clone)]
pub enum Lift {
    Levels(usize),
    To(GlobMatcher),
}

#[derive(Debug, Clone)]
pub struct Next {
    pub into: String,
    pub like: Template,
}

#[derive(Debug, Clone)]
pub struct Rank {
    pub into: String,
    pub by: Vec<String>,
    pub prefer: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct Take {
    pub fields: Vec<String>,
    pub from: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OnConflict {
    Reject,
    Suffix,
}

#[derive(Debug, Clone)]
pub struct Move {
    pub on_conflict: OnConflict,
    pub suffix: String,
}

#[derive(Debug, Clone)]
pub struct Cleanup {
    pub keep: Vec<GlobMatcher>,
}

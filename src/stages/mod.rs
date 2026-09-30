//! The stages a pipeline is stacked from, as declared in configuration.

mod batch;
mod declare;
mod effect;
mod filter;
mod format;
mod next;
mod number;
mod path;
mod pattern;
mod rank;
mod take;
mod text;

use std::collections::BTreeMap;

use globset::GlobMatcher;

use crate::context::Context;
use crate::record::{Record, Value, is_usable_file_name};
use crate::template::Template;

pub use batch::Batch;
pub use declare::DeclareError;
pub use effect::Effect;

/// What a stage answers for one record: the record to go on with, or what stops it.
pub type Flow = Result<Record, Stop>;

/// What ends the way of a record through its pipeline: a filter excluded it, or a stage refused it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Stop {
    Excluded,
    Rejected(Rejection),
}

/// Why a stage refused a record, and which stage it was.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rejection {
    pub stage: String,
    pub reason: String,
}

impl Stop {
    fn rejected(stage: &str, reason: impl Into<String>) -> Stop {
        Stop::Rejected(Rejection {
            stage: stage.to_string(),
            reason: reason.into(),
        })
    }
}

/// Writes a field of the record; a `name` or `ext` that would not make a usable file name is refused.
fn write_field(stage: &str, record: &mut Record, field: &str, value: Value) -> Result<(), Stop> {
    let written = value.written();
    let usable = match (field, written.as_deref()) {
        ("name", Some(name)) => is_usable_file_name(name),
        ("ext", Some(ext)) => ext.is_empty() || !ext.contains(['/', '\0']),
        _ => true,
    };
    if !usable {
        let shown = written.unwrap_or_default();
        return Err(Stop::rejected(
            stage,
            format!("`{shown}` is not usable as the {field}"),
        ));
    }
    record.set_field(field, value);
    Ok(())
}

/// The refusal of a stage that could not go on, for the reason it met.
fn refused_by<E: ToString>(stage: &str) -> impl Fn(E) -> Stop + '_ {
    move |reason| Stop::rejected(stage, reason.to_string())
}

/// The refusal of a stage whose field does not hold what the stage reads.
fn unfit(stage: &str, field: &str, problem: &str) -> Stop {
    Stop::rejected(stage, format!("the field `{field}` {problem}"))
}

/// What a field holds as it is written into a name, for a stage that reads text or a number.
fn written_field(stage: &str, record: &Record, name: &str) -> Result<String, Stop> {
    match record.field(name).map(Value::written) {
        Some(Some(written)) => Ok(written),
        Some(None) => Err(unfit(stage, name, "is a date")),
        None => Err(unfit(stage, name, "does not exist")),
    }
}

/// The text a field holds, for a stage that rewrites text.
fn text_field<'a>(stage: &str, record: &'a Record, name: &str) -> Result<&'a str, Stop> {
    match record.field(name) {
        Some(Value::Text(text)) => Ok(text),
        Some(_) => Err(unfit(stage, name, "is not text")),
        None => Err(unfit(stage, name, "does not exist")),
    }
}

/// What every stage does: one function over a batch, whatever the stage is.
pub trait Run {
    /// The name the stage is declared by.
    fn name(&self) -> &'static str;

    /// Runs the stage over the files the running pipeline still holds, with what the batch and the
    /// target let it know.
    fn run(&self, batch: &mut Batch, context: &mut Context);
}

/// Fixed values written into records by `set` and `default`.
pub type Fields = BTreeMap<String, Value>;

/// One small function of a pipeline, declared with its parameters.
#[derive(Debug, Clone)]
pub enum Stage {
    Filter(Filter),
    Number(Number),
    Regex(Pattern),
    Set(SetFields),
    Default(DefaultFields),
    Replace(Replace),
    Case(Case),
    Strip(Strip),
    Format(Format),
    Folder(Folder),
    Lift(Lift),
    Next(Next),
    Rank(Rank),
    Take(Take),
    Move(Move),
    Cleanup(Cleanup),
}

impl Stage {
    /// What the stage does, whichever stage it was declared as.
    fn as_run(&self) -> &dyn Run {
        match self {
            Stage::Filter(stage) => stage,
            Stage::Number(stage) => stage,
            Stage::Regex(stage) => stage,
            Stage::Set(stage) => stage,
            Stage::Default(stage) => stage,
            Stage::Replace(stage) => stage,
            Stage::Case(stage) => stage,
            Stage::Strip(stage) => stage,
            Stage::Format(stage) => stage,
            Stage::Folder(stage) => stage,
            Stage::Lift(stage) => stage,
            Stage::Next(stage) => stage,
            Stage::Rank(stage) => stage,
            Stage::Take(stage) => stage,
            Stage::Move(stage) => stage,
            Stage::Cleanup(stage) => stage,
        }
    }

    /// The name the stage is declared by.
    pub fn name(&self) -> &'static str {
        self.as_run().name()
    }

    /// Runs the stage over the files the running pipeline still holds.
    pub(crate) fn run(&self, batch: &mut Batch, context: &mut Context) {
        self.as_run().run(batch, context);
    }

    /// Runs the stage on one record, which is a batch of one, with what the target lets it know.
    pub fn apply(&self, record: Record, context: &mut Context) -> Flow {
        let mut batch = Batch::new(vec![record]);
        batch.claim("", |_| true);
        self.run(&mut batch, context);
        batch
            .into_files()
            .pop()
            .map(|file| file.flow)
            .expect("a batch of one holds its file")
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

/// The fixed values `set` writes, replacing what a field held.
#[derive(Debug, Clone)]
pub struct SetFields(pub Fields);

/// The fixed values `default` writes into the fields that hold nothing yet.
#[derive(Debug, Clone)]
pub struct DefaultFields(pub Fields);

/// The template `format` writes the name from.
#[derive(Debug, Clone)]
pub struct Format(pub Template);

/// The template `folder` puts the file under.
#[derive(Debug, Clone)]
pub struct Folder(pub Template);

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

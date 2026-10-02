//! The stages a pipeline is stacked from, as declared in configuration.

mod batch;
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

use std::collections::BTreeMap;

use globset::GlobMatcher;

use crate::context::Context;
use crate::record::{Record, Value, is_usable_file_name};
use crate::template::Template;

pub use batch::Batch;
pub use declare::{Declaration, DeclareError, Parameter, ParameterKind};

/// What a stage answers for one record: the record to go on with, or what stops it.
pub type Flow = Result<Record, Stop>;

/// Why a stage refused a record, and which stage it was. A filter that turns a record away refuses it
/// too, so every stop is a refusal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Stop {
    pub stage: String,
    pub reason: String,
}

impl Stop {
    pub(crate) fn rejected(stage: &str, reason: impl Into<String>) -> Stop {
        Stop {
            stage: stage.to_string(),
            reason: reason.into(),
        }
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

/// Writes a field a stage detected or a pipeline fixed, unless `vars` answered it: an answer is what the
/// user gave for what detection cannot find, so it stands.
fn fill_field(stage: &str, record: &mut Record, field: &str, value: Value) -> Result<(), Stop> {
    if record.is_answer(field) {
        return Ok(());
    }
    write_field(stage, record, field, value)
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

/// One small function over a batch, which every stage there is implements.
pub trait Stage {
    /// The name the stage is declared by.
    fn name(&self) -> &'static str;

    /// Runs the stage over the batch its pipeline claimed, with what the target and the pipelines before
    /// let it know, and hands on the batch it made.
    fn run(&self, batch: Batch, context: &mut Context) -> Batch;
}

/// Fixed values written into records by `set` and `default`.
pub type Fields = BTreeMap<String, Value>;

/// One stage as a pipeline declares it, with its parameters: the closed list of the stages there are.
#[derive(Debug, Clone)]
pub enum Declared {
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
}

impl Declared {
    /// What the stage does, whichever stage it was declared as.
    fn as_stage(&self) -> &dyn Stage {
        match self {
            Declared::Filter(stage) => stage,
            Declared::Number(stage) => stage,
            Declared::Regex(stage) => stage,
            Declared::Set(stage) => stage,
            Declared::Default(stage) => stage,
            Declared::Replace(stage) => stage,
            Declared::Case(stage) => stage,
            Declared::Strip(stage) => stage,
            Declared::Format(stage) => stage,
            Declared::Folder(stage) => stage,
            Declared::Lift(stage) => stage,
            Declared::Next(stage) => stage,
            Declared::Rank(stage) => stage,
            Declared::Take(stage) => stage,
        }
    }

    /// The name the stage is declared by.
    pub fn name(&self) -> &'static str {
        self.as_stage().name()
    }

    /// Runs the stage over the batch its pipeline claimed.
    pub(crate) fn run(&self, batch: Batch, context: &mut Context) -> Batch {
        self.as_stage().run(batch, context)
    }

    /// Runs the stage on one record, which is a batch of one, with what the target lets it know.
    pub fn apply(&self, record: Record, context: &mut Context) -> Flow {
        self.run(Batch::new(vec![record]), context)
            .into_files()
            .pop()
            .map(|file| file.flow)
            .expect("a batch of one holds its file")
    }

    /// Whether the stage rewrites the folders of the plan.
    pub fn is_path(&self) -> bool {
        matches!(self, Declared::Lift(_) | Declared::Folder(_))
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
    Keep(usize),
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

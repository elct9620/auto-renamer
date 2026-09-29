//! The stages a pipeline is stacked from, as declared in configuration.

mod declare;

use std::collections::BTreeMap;

use globset::GlobMatcher;

use crate::record::Value;
use crate::template::Template;

pub use declare::DeclareError;

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
    pub prefix: Option<String>,
    pub exclude: Vec<String>,
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

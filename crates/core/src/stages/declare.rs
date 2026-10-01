use std::fmt;

use globset::{Glob, GlobMatcher};
use regex::RegexBuilder;
use toml::Value as Toml;

use super::{
    Case, CaseKind, Cleanup, Declared, DefaultFields, Fields, Filter, Folder, Format, Lift, Move,
    Next, Number, OnConflict, Pattern, Prefix, Rank, Replace, SetFields, Strip, Take,
};
use crate::reader::{Reader, Scope};
use crate::template::Template;

/// The most a compiled regular expression may take, so a declaration cannot exhaust memory.
const REGEX_SIZE_LIMIT: usize = 1 << 20;

const DEFAULT_FIELD: &str = "name";
const DEFAULT_SUFFIX: &str = "_v2";

/// Why one stage declaration was refused.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeclareError {
    NotAStage,
    NotOneKey,
    UnknownStage(String),
    NeedsParameters(String),
    Invalid {
        stage: String,
        parameter: Option<String>,
        reason: String,
    },
}

impl fmt::Display for DeclareError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DeclareError::NotAStage => write!(f, "a stage is a name or a table with one key"),
            DeclareError::NotOneKey => {
                write!(f, "a stage table needs exactly one key, the stage name")
            }
            DeclareError::UnknownStage(name) => write!(f, "unknown stage `{name}`"),
            DeclareError::NeedsParameters(name) => write!(f, "the stage `{name}` needs parameters"),
            DeclareError::Invalid {
                stage,
                parameter: Some(parameter),
                reason,
            } => write!(f, "stage `{stage}`, parameter `{parameter}`: {reason}"),
            DeclareError::Invalid {
                stage,
                parameter: None,
                reason,
            } => write!(f, "stage `{stage}`: {reason}"),
        }
    }
}

impl std::error::Error for DeclareError {}

impl Declared {
    /// Reads one stage from its declaration, refusing a mistaken one.
    pub fn read(value: &Toml) -> Result<Declared, DeclareError> {
        match value {
            Toml::String(name) => declare_named(name, None),
            Toml::Table(table) => {
                let mut entries = table.iter();
                match (entries.next(), entries.next()) {
                    (Some((name, parameters)), None) => declare_named(name, Some(parameters)),
                    _ => Err(DeclareError::NotOneKey),
                }
            }
            _ => Err(DeclareError::NotAStage),
        }
    }
}

fn declare_named(name: &str, value: Option<&Toml>) -> Result<Declared, DeclareError> {
    match name {
        "filter" => filter(value),
        "number" => number(value),
        "regex" => regex(value),
        "set" => fields("set", value).map(|fields| Declared::Set(SetFields(fields))),
        "default" => {
            fields("default", value).map(|fields| Declared::Default(DefaultFields(fields)))
        }
        "replace" => replace(value),
        "case" => case(value),
        "strip" => strip(value),
        "format" => template("format", value).map(|template| Declared::Format(Format(template))),
        "folder" => template("folder", value).map(|template| Declared::Folder(Folder(template))),
        "lift" => lift(value),
        "next" => next(value),
        "rank" => rank(value),
        "take" => take(value),
        "move" => move_stage(value),
        "cleanup" => cleanup(value),
        other => Err(DeclareError::UnknownStage(other.to_string())),
    }
}

/// The parameters of one stage, read with the stage's name as their scope.
type Args = Reader<&'static str>;

impl Scope for &'static str {
    type Error = DeclareError;

    fn invalid(&self, parameter: &str, reason: String) -> DeclareError {
        invalid(self, Some(parameter), reason)
    }

    fn unknown(&self, parameter: &str) -> DeclareError {
        invalid(self, Some(parameter), "is not a parameter of this stage")
    }
}

/// Parameters a stage cannot do without; a bare name is refused.
fn required(stage: &'static str, value: Option<&Toml>) -> Result<Args, DeclareError> {
    match value {
        None => Err(DeclareError::NeedsParameters(stage.to_string())),
        Some(value) => parameters(stage, value),
    }
}

/// Parameters a stage has defaults for; a bare name declares all of them at once.
fn optional(stage: &'static str, value: Option<&Toml>) -> Result<Args, DeclareError> {
    match value {
        None => Ok(Reader::new(stage, toml::Table::new())),
        Some(value) => parameters(stage, value),
    }
}

fn parameters(stage: &'static str, value: &Toml) -> Result<Args, DeclareError> {
    match value {
        Toml::Table(table) => Ok(Reader::new(stage, table.clone())),
        _ => Err(invalid(stage, None, "expects a table of parameters")),
    }
}

/// A list of field names the stage cannot do without.
fn required_fields(args: &mut Args, key: &str) -> Result<Vec<String>, DeclareError> {
    let fields = args
        .strings(key)?
        .ok_or_else(|| args.invalid(key, "is required"))?;
    if fields.is_empty() {
        return Err(args.invalid(key, "needs at least one field"));
    }
    Ok(fields)
}

fn glob(args: &Args, key: &str, source: &str) -> Result<GlobMatcher, DeclareError> {
    Glob::new(source)
        .map(|glob| glob.compile_matcher())
        .map_err(|error| args.invalid(key, error.to_string()))
}

fn invalid(stage: &str, parameter: Option<&str>, reason: impl Into<String>) -> DeclareError {
    DeclareError::Invalid {
        stage: stage.to_string(),
        parameter: parameter.map(str::to_string),
        reason: reason.into(),
    }
}

fn field_or_default(args: &mut Args, key: &str) -> Result<String, DeclareError> {
    Ok(args
        .string(key)?
        .unwrap_or_else(|| DEFAULT_FIELD.to_string()))
}

fn filter(value: Option<&Toml>) -> Result<Declared, DeclareError> {
    let mut args = required("filter", value)?;
    let ext: Vec<String> = args
        .strings("ext")?
        .unwrap_or_default()
        .iter()
        .map(|ext| ext.to_lowercase())
        .collect();
    let glob = match args.string("glob")? {
        Some(source) => Some(glob(&args, "glob", &source)?),
        None => None,
    };
    let invert = args.boolean("invert")?.unwrap_or(false);
    if ext.is_empty() && glob.is_none() {
        return Err(args.invalid("ext", "needs `ext` or `glob` to say which files it means"));
    }
    args.finish()?;
    Ok(Declared::Filter(Filter { ext, glob, invert }))
}

fn number(value: Option<&Toml>) -> Result<Declared, DeclareError> {
    let mut args = required("number", value)?;
    let from = field_or_default(&mut args, "from")?;
    let into = args.required_string("into")?;
    let nth = args.integer("nth")?;
    if nth == Some(0) {
        return Err(args.invalid("nth", "counts from 1, or from the end when negative"));
    }
    let prefix = match args.string("prefix")? {
        Some(word) => {
            Some(Prefix::new(&word).map_err(|error| args.invalid("prefix", error.to_string()))?)
        }
        None => None,
    };
    let exclude = args.strings("exclude")?.unwrap_or_default();
    args.finish()?;
    Ok(Declared::Number(Number {
        from,
        into,
        nth,
        prefix,
        exclude,
    }))
}

fn regex(value: Option<&Toml>) -> Result<Declared, DeclareError> {
    let mut args = required("regex", value)?;
    let source = args.required_string("pattern")?;
    let from = field_or_default(&mut args, "from")?;
    let into = args.string("into")?;
    let replace = args.string("replace")?;
    if into.is_some() && replace.is_some() {
        return Err(args.invalid("replace", "cannot be combined with `into`"));
    }
    let pattern = RegexBuilder::new(&source)
        .size_limit(REGEX_SIZE_LIMIT)
        .build()
        .map_err(|error| args.invalid("pattern", error.to_string()))?;
    args.finish()?;
    Ok(Declared::Regex(Pattern {
        pattern,
        from,
        into,
        replace,
    }))
}

fn fields(stage: &'static str, value: Option<&Toml>) -> Result<Fields, DeclareError> {
    required(stage, value)?.into_values()
}

fn replace(value: Option<&Toml>) -> Result<Declared, DeclareError> {
    let mut args = required("replace", value)?;
    let find = args.required_string("find")?;
    if find.is_empty() {
        return Err(args.invalid("find", "needs something to look for"));
    }
    let with = args.required_string("with")?;
    let field = field_or_default(&mut args, "field")?;
    args.finish()?;
    Ok(Declared::Replace(Replace { find, with, field }))
}

fn case(value: Option<&Toml>) -> Result<Declared, DeclareError> {
    let mut args = required("case", value)?;
    let to = match args.required_string("to")?.as_str() {
        "lower" => CaseKind::Lower,
        "upper" => CaseKind::Upper,
        "title" => CaseKind::Title,
        _ => return Err(args.invalid("to", "must be lower, upper or title")),
    };
    let field = field_or_default(&mut args, "field")?;
    args.finish()?;
    Ok(Declared::Case(Case { to, field }))
}

fn strip(value: Option<&Toml>) -> Result<Declared, DeclareError> {
    let mut args = optional("strip", value)?;
    let sources = args
        .strings("groups")?
        .unwrap_or_else(|| vec!["[]".to_string()]);
    let mut groups = Vec::new();
    for source in &sources {
        let mut chars = source.chars();
        match (chars.next(), chars.next(), chars.next()) {
            (Some(open), Some(close), None) => groups.push((open, close)),
            _ => {
                return Err(
                    args.invalid("groups", "each group is an opening and a closing character")
                );
            }
        }
    }
    let field = field_or_default(&mut args, "field")?;
    args.finish()?;
    Ok(Declared::Strip(Strip { groups, field }))
}

fn template(stage: &'static str, value: Option<&Toml>) -> Result<Template, DeclareError> {
    match value {
        None => Err(DeclareError::NeedsParameters(stage.to_string())),
        Some(Toml::String(source)) => {
            Template::parse(source).map_err(|error| invalid(stage, None, error.to_string()))
        }
        Some(_) => Err(invalid(stage, None, "expects a template written as text")),
    }
}

fn lift(value: Option<&Toml>) -> Result<Declared, DeclareError> {
    match value {
        None => Err(DeclareError::NeedsParameters("lift".to_string())),
        Some(Toml::Integer(levels)) if *levels >= 1 => {
            Ok(Declared::Lift(Lift::Levels(*levels as usize)))
        }
        Some(Toml::Integer(_)) => Err(invalid("lift", None, "lifts at least one level")),
        Some(other) => {
            let mut args = parameters("lift", other)?;
            let source = args.required_string("to")?;
            let matcher = glob(&args, "to", &source)?;
            args.finish()?;
            Ok(Declared::Lift(Lift::To(matcher)))
        }
    }
}

fn next(value: Option<&Toml>) -> Result<Declared, DeclareError> {
    let mut args = required("next", value)?;
    let into = args.required_string("into")?;
    let source = args.required_string("like")?;
    let like = Template::parse(&source).map_err(|error| args.invalid("like", error.to_string()))?;
    if !like.has_field(&into) {
        return Err(args.invalid("like", "must mention the field it fills"));
    }
    args.finish()?;
    Ok(Declared::Next(Next { into, like }))
}

fn rank(value: Option<&Toml>) -> Result<Declared, DeclareError> {
    let mut args = required("rank", value)?;
    let into = args.required_string("into")?;
    let by = required_fields(&mut args, "by")?;
    let prefer = args.strings("prefer")?.unwrap_or_default();
    args.finish()?;
    Ok(Declared::Rank(Rank { into, by, prefer }))
}

fn take(value: Option<&Toml>) -> Result<Declared, DeclareError> {
    let mut args = required("take", value)?;
    let fields = required_fields(&mut args, "fields")?;
    let from = args.string("from")?;
    args.finish()?;
    Ok(Declared::Take(Take { fields, from }))
}

fn move_stage(value: Option<&Toml>) -> Result<Declared, DeclareError> {
    let mut args = optional("move", value)?;
    let on_conflict = match args.string("on_conflict")?.as_deref() {
        None | Some("reject") => OnConflict::Reject,
        Some("suffix") => OnConflict::Suffix,
        Some(_) => return Err(args.invalid("on_conflict", "must be reject or suffix")),
    };
    let suffix = args
        .string("suffix")?
        .unwrap_or_else(|| DEFAULT_SUFFIX.to_string());
    if suffix.is_empty() || suffix.contains('/') || suffix.contains('\0') {
        return Err(args.invalid(
            "suffix",
            "is added to a file name, so it cannot be empty or hold a slash",
        ));
    }
    args.finish()?;
    Ok(Declared::Move(Move {
        on_conflict,
        suffix,
    }))
}

fn cleanup(value: Option<&Toml>) -> Result<Declared, DeclareError> {
    let mut args = optional("cleanup", value)?;
    let mut keep = Vec::new();
    for source in args.strings("keep")?.unwrap_or_default() {
        keep.push(glob(&args, "keep", &source)?);
    }
    args.finish()?;
    Ok(Declared::Cleanup(Cleanup { keep }))
}

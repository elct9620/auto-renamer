use std::fmt;

use globset::{Glob, GlobMatcher};
use regex::RegexBuilder;
use toml::Value as Toml;

use super::{
    Case, CaseKind, Declared, DefaultFields, Fields, Filter, Folder, Format, Lift, Next, Number,
    Pattern, Prefix, Rank, Replace, SetFields, Strip, Take,
};
use crate::reader::{Reader, Scope};
use crate::template::Template;

/// The most a compiled regular expression may take, so a declaration cannot exhaust memory.
const REGEX_SIZE_LIMIT: usize = 1 << 20;

const DEFAULT_FIELD: &str = "name";

/// How one stage is declared, for a form to be made from, and the reading of its declaration.
#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Declaration {
    pub name: &'static str,
    /// The name alone declares the stage.
    pub bare: bool,
    /// The kind of the single value the stage may be declared with, as `{ format = "..." }`.
    pub value: Option<ParameterKind>,
    /// The parameters the stage's table may hold.
    pub parameters: &'static [Parameter],
    /// The table holds fixed values under names of the user's choosing.
    pub values: bool,
    /// A declaration the stage accepts, written as its inline TOML value; empty for the bare name.
    pub example: &'static str,
    #[cfg_attr(feature = "serde", serde(skip))]
    read: fn(Option<&Toml>) -> Result<Declared, DeclareError>,
}

/// One named parameter of a stage.
#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Parameter {
    pub name: &'static str,
    pub kind: ParameterKind,
    pub required: bool,
    /// The only values the parameter takes; empty when it is not limited.
    pub choices: &'static [&'static str],
}

/// The kind of value a parameter takes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "lowercase"))]
pub enum ParameterKind {
    Text,
    Texts,
    Integer,
    Boolean,
}

const fn may(name: &'static str, kind: ParameterKind) -> Parameter {
    Parameter {
        name,
        kind,
        required: false,
        choices: &[],
    }
}

const fn must(name: &'static str, kind: ParameterKind) -> Parameter {
    Parameter {
        name,
        kind,
        required: true,
        choices: &[],
    }
}

const fn stage(
    name: &'static str,
    parameters: &'static [Parameter],
    example: &'static str,
    read: fn(Option<&Toml>) -> Result<Declared, DeclareError>,
) -> Declaration {
    Declaration {
        name,
        bare: false,
        value: None,
        parameters,
        values: false,
        example,
        read,
    }
}

use ParameterKind::{Boolean, Integer, Text, Texts};

/// Every stage there is; a declaration is read by this list.
static DECLARATIONS: &[Declaration] = &[
    stage(
        "filter",
        &[may("ext", Texts), may("glob", Text), may("invert", Boolean)],
        r#"{ ext = ["mkv", "mp4"] }"#,
        filter,
    ),
    stage(
        "number",
        &[
            may("from", Text),
            must("into", Text),
            may("nth", Integer),
            may("prefix", Text),
            may("exclude", Texts),
        ],
        r#"{ from = "path", into = "season", prefix = "Season" }"#,
        number,
    ),
    stage(
        "regex",
        &[
            must("pattern", Text),
            may("from", Text),
            may("into", Text),
            may("replace", Text),
        ],
        r#"{ pattern = '(\d+)', into = "episode" }"#,
        regex,
    ),
    Declaration {
        values: true,
        ..stage("set", &[], r#"{ kind = "video" }"#, set)
    },
    Declaration {
        values: true,
        ..stage("default", &[], "{ season = 1 }", default)
    },
    stage(
        "replace",
        &[must("find", Text), must("with", Text), may("field", Text)],
        r#"{ find = "_", with = " " }"#,
        replace,
    ),
    stage(
        "case",
        &[
            Parameter {
                choices: &["lower", "upper", "title"],
                ..must("to", Text)
            },
            may("field", Text),
        ],
        r#"{ to = "lower" }"#,
        case,
    ),
    Declaration {
        bare: true,
        ..stage(
            "strip",
            &[may("groups", Texts), may("field", Text)],
            "",
            strip,
        )
    },
    Declaration {
        value: Some(Text),
        ..stage(
            "format",
            &[],
            r#""{show} s{season:02}e{episode:02}""#,
            format,
        )
    },
    Declaration {
        value: Some(Text),
        ..stage("folder", &[], r#""{show}""#, folder)
    },
    Declaration {
        value: Some(Integer),
        ..stage("lift", &[may("to", Text), may("keep", Integer)], "1", lift)
    },
    stage(
        "next",
        &[must("into", Text), must("like", Text)],
        r#"{ into = "episode", like = "{show} s{season:02}e{episode:02}" }"#,
        next,
    ),
    stage(
        "rank",
        &[must("into", Text), must("by", Texts), may("prefer", Texts)],
        r#"{ into = "episode", by = ["season"] }"#,
        rank,
    ),
    stage(
        "take",
        &[must("fields", Texts), may("from", Text)],
        r#"{ fields = ["episode"] }"#,
        take,
    ),
];

fn declaration(name: &str) -> Option<&'static Declaration> {
    DECLARATIONS
        .iter()
        .find(|declaration| declaration.name == name)
}

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
    /// How each stage there is is declared, from the list a declaration is read by.
    pub fn declarations() -> &'static [Declaration] {
        DECLARATIONS
    }

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
    match declaration(name) {
        Some(declaration) => (declaration.read)(value),
        None => Err(DeclareError::UnknownStage(name.to_string())),
    }
}

fn set(value: Option<&Toml>) -> Result<Declared, DeclareError> {
    fields("set", value).map(|fields| Declared::Set(SetFields(fields)))
}

fn default(value: Option<&Toml>) -> Result<Declared, DeclareError> {
    fields("default", value).map(|fields| Declared::Default(DefaultFields(fields)))
}

fn format(value: Option<&Toml>) -> Result<Declared, DeclareError> {
    template("format", value).map(|template| Declared::Format(Format(template)))
}

fn folder(value: Option<&Toml>) -> Result<Declared, DeclareError> {
    template("folder", value).map(|template| Declared::Folder(Folder(template)))
}

/// The parameters of one stage, read with the stage's name as their scope.
type Args = Reader<&'static Declaration>;

impl Scope for &'static Declaration {
    type Error = DeclareError;

    fn invalid(&self, parameter: &str, reason: String) -> DeclareError {
        invalid(self.name, Some(parameter), reason)
    }

    fn unknown(&self, parameter: &str) -> DeclareError {
        invalid(
            self.name,
            Some(parameter),
            "is not a parameter of this stage",
        )
    }

    fn describes(&self, parameter: &str) -> bool {
        self.parameters
            .iter()
            .any(|described| described.name == parameter)
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
        None => Ok(Reader::new(described(stage), toml::Table::new())),
        Some(value) => parameters(stage, value),
    }
}

fn described(stage: &str) -> &'static Declaration {
    declaration(stage).expect("every stage read is in the list of declarations")
}

fn parameters(stage: &'static str, value: &Toml) -> Result<Args, DeclareError> {
    match value {
        Toml::Table(table) => Ok(Reader::new(described(stage), table.clone())),
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
            let to = args.string("to")?;
            let keep = args.integer("keep")?;
            let lift = match (to, keep) {
                (Some(source), None) => Lift::To(glob(&args, "to", &source)?),
                (None, Some(keep)) => Lift::Keep(
                    usize::try_from(keep)
                        .map_err(|_| args.invalid("keep", "counts folders from 0"))?,
                ),
                _ => return Err(invalid("lift", None, "takes either `to` or `keep`")),
            };
            args.finish()?;
            Ok(Declared::Lift(lift))
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

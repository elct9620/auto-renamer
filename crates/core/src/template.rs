use std::fmt;
use std::iter::Peekable;
use std::str::Chars;

use chrono::format::{Item, StrftimeItems};
use regex::{Regex, RegexBuilder};

use crate::record::{Record, Value};

/// A `u64` has at most 20 digits, so a wider padding only spends memory.
const MAX_PAD_WIDTH: usize = 20;

/// The most a matcher built from a template may take to compile.
const MATCHER_SIZE_LIMIT: usize = 1 << 20;

/// A name written out of a record's fields.
#[derive(Debug, Clone)]
pub struct Template {
    segments: Vec<Segment>,
}

#[derive(Debug, Clone)]
enum Segment {
    Literal(String),
    Field { name: String, spec: Option<Spec> },
    Optional(Vec<Segment>),
}

#[derive(Debug, Clone)]
enum Spec {
    Pad {
        width: usize,
        raw: String,
    },
    Date {
        items: Vec<Item<'static>>,
        raw: String,
    },
}

impl Spec {
    fn raw(&self) -> &str {
        match self {
            Spec::Pad { raw, .. } | Spec::Date { raw, .. } => raw,
        }
    }
}

/// Why a template was refused when it was read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TemplateError {
    UnclosedField,
    UnclosedOptional,
    NestedOptional,
    StrayClose(char),
    EmptyField,
    InvalidSpec(String),
    InvalidDateFormat(String),
}

/// Why a template was refused for one record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RenderError {
    MissingField(String),
    SpecMismatch { field: String, spec: String },
    NeedsFormat(String),
    TooLarge,
}

impl fmt::Display for TemplateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TemplateError::UnclosedField => write!(f, "a field is missing its closing brace"),
            TemplateError::UnclosedOptional => {
                write!(f, "an optional part is missing its closing bracket")
            }
            TemplateError::NestedOptional => write!(f, "optional parts cannot be nested"),
            TemplateError::StrayClose(ch) => {
                write!(f, "unexpected `{ch}`; write it twice for a literal")
            }
            TemplateError::EmptyField => write!(f, "a field has no name"),
            TemplateError::InvalidSpec(spec) => {
                write!(f, "`{spec}` is neither a zero padding nor a date format")
            }
            TemplateError::InvalidDateFormat(spec) => write!(f, "`{spec}` is not a date format"),
        }
    }
}

impl fmt::Display for RenderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RenderError::MissingField(field) => write!(f, "the field `{field}` has no value"),
            RenderError::SpecMismatch { field, spec } => {
                write!(f, "the format `{spec}` does not suit the field `{field}`")
            }
            RenderError::NeedsFormat(field) => write!(f, "the date field `{field}` needs a format"),
            RenderError::TooLarge => write!(f, "the names to match are too long to match"),
        }
    }
}

impl std::error::Error for TemplateError {}
impl std::error::Error for RenderError {}

impl Template {
    /// Reads a template, refusing one that is malformed.
    pub fn parse(source: &str) -> Result<Template, TemplateError> {
        let mut chars = source.chars().peekable();
        let segments = parse_segments(&mut chars, false)?;
        Ok(Template { segments })
    }

    /// Whether a field of this name is written anywhere in the template.
    pub fn has_field(&self, name: &str) -> bool {
        fn any(segments: &[Segment], name: &str) -> bool {
            segments.iter().any(|segment| match segment {
                Segment::Literal(_) => false,
                Segment::Field { name: field, .. } => field == name,
                Segment::Optional(inner) => any(inner, name),
            })
        }
        any(&self.segments, name)
    }

    /// A pattern for the names this template writes, capturing the number of the field `capture`
    /// and taking every other field at its value in `record`.
    pub(crate) fn matcher(&self, record: &Record, capture: &str) -> Result<Regex, RenderError> {
        let mut source = String::from("^");
        matcher_segments(&self.segments, record, capture, &mut source)?;
        source.push('$');
        RegexBuilder::new(&source)
            .size_limit(MATCHER_SIZE_LIMIT)
            .build()
            .map_err(|_| RenderError::TooLarge)
    }

    /// Writes the template out for one record, refusing when a required field is missing or misused.
    pub fn render(&self, record: &Record) -> Result<String, RenderError> {
        let mut out = String::new();
        render_segments(&self.segments, record, &mut out)?;
        Ok(out)
    }
}

fn parse_segments(
    chars: &mut Peekable<Chars<'_>>,
    in_optional: bool,
) -> Result<Vec<Segment>, TemplateError> {
    let mut segments = Vec::new();
    let mut literal = String::new();
    let flush = |literal: &mut String, segments: &mut Vec<Segment>| {
        if !literal.is_empty() {
            segments.push(Segment::Literal(std::mem::take(literal)));
        }
    };

    while let Some(ch) = chars.next() {
        match ch {
            '{' if chars.next_if_eq(&'{').is_some() => literal.push('{'),
            '}' if chars.next_if_eq(&'}').is_some() => literal.push('}'),
            '[' if chars.next_if_eq(&'[').is_some() => literal.push('['),
            '{' => {
                flush(&mut literal, &mut segments);
                segments.push(parse_field(chars)?);
            }
            '[' if in_optional => return Err(TemplateError::NestedOptional),
            '[' => {
                flush(&mut literal, &mut segments);
                segments.push(Segment::Optional(parse_segments(chars, true)?));
            }
            ']' if !in_optional && chars.next_if_eq(&']').is_some() => literal.push(']'),
            ']' if in_optional => {
                flush(&mut literal, &mut segments);
                return Ok(segments);
            }
            '}' | ']' => return Err(TemplateError::StrayClose(ch)),
            other => literal.push(other),
        }
    }

    if in_optional {
        return Err(TemplateError::UnclosedOptional);
    }
    flush(&mut literal, &mut segments);
    Ok(segments)
}

fn parse_field(chars: &mut Peekable<Chars<'_>>) -> Result<Segment, TemplateError> {
    let mut body = String::new();
    loop {
        match chars.next() {
            Some('}') => break,
            Some(ch) => body.push(ch),
            None => return Err(TemplateError::UnclosedField),
        }
    }

    let (name, spec) = match body.split_once(':') {
        Some((name, spec)) => (name, Some(parse_spec(spec)?)),
        None => (body.as_str(), None),
    };
    if name.is_empty() {
        return Err(TemplateError::EmptyField);
    }
    Ok(Segment::Field {
        name: name.to_string(),
        spec,
    })
}

fn parse_spec(raw: &str) -> Result<Spec, TemplateError> {
    if raw.starts_with('%') {
        let items: Vec<Item<'static>> = StrftimeItems::new(raw)
            .map(|item| item.to_owned())
            .collect();
        if items.iter().any(|item| matches!(item, Item::Error)) {
            return Err(TemplateError::InvalidDateFormat(raw.to_string()));
        }
        return Ok(Spec::Date {
            items,
            raw: raw.to_string(),
        });
    }

    match raw.strip_prefix('0').map(str::parse::<usize>) {
        Some(Ok(width)) if width <= MAX_PAD_WIDTH => Ok(Spec::Pad {
            width,
            raw: raw.to_string(),
        }),
        _ => Err(TemplateError::InvalidSpec(raw.to_string())),
    }
}

fn render_segments(
    segments: &[Segment],
    record: &Record,
    out: &mut String,
) -> Result<(), RenderError> {
    for segment in segments {
        match segment {
            Segment::Literal(text) => out.push_str(text),
            Segment::Field { name, spec } => {
                out.push_str(&render_field(name, spec.as_ref(), record)?)
            }
            Segment::Optional(inner) => {
                let mut part = String::new();
                match render_segments(inner, record, &mut part) {
                    Ok(()) => out.push_str(&part),
                    Err(RenderError::MissingField(_)) => {}
                    Err(other) => return Err(other),
                }
            }
        }
    }
    Ok(())
}

fn render_field(name: &str, spec: Option<&Spec>, record: &Record) -> Result<String, RenderError> {
    let value = record
        .field(name)
        .ok_or_else(|| RenderError::MissingField(name.to_string()))?;
    let mismatch = |spec: &Spec| RenderError::SpecMismatch {
        field: name.to_string(),
        spec: spec.raw().to_string(),
    };

    match (value, spec) {
        (Value::Text(text), None) => Ok(text.clone()),
        (Value::Number(number), None) => Ok(number.to_string()),
        (Value::Number(number), Some(Spec::Pad { width, .. })) => Ok(format!("{number:0width$}")),
        (Value::Date(date), Some(Spec::Date { items, .. })) => {
            Ok(date.format_with_items(items.iter()).to_string())
        }
        (Value::Date(_), None) => Err(RenderError::NeedsFormat(name.to_string())),
        (_, Some(spec)) => Err(mismatch(spec)),
    }
}

fn matcher_segments(
    segments: &[Segment],
    record: &Record,
    capture: &str,
    out: &mut String,
) -> Result<(), RenderError> {
    for segment in segments {
        match segment {
            Segment::Literal(text) => out.push_str(&regex::escape(text)),
            Segment::Field { name, spec } if name == capture => match spec {
                Some(Spec::Pad { width, .. }) => {
                    out.push_str(&format!("([0-9]{{{},}})", (*width).max(1)))
                }
                _ => out.push_str("([0-9]+)"),
            },
            Segment::Field { name, spec } => {
                out.push_str(&regex::escape(&render_field(name, spec.as_ref(), record)?))
            }
            Segment::Optional(inner) => {
                let mut part = String::new();
                match matcher_segments(inner, record, capture, &mut part) {
                    Ok(()) => {
                        out.push_str("(?:");
                        out.push_str(&part);
                        out.push_str(")?");
                    }
                    Err(RenderError::MissingField(_)) => {}
                    Err(other) => return Err(other),
                }
            }
        }
    }
    Ok(())
}

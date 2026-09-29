use regex::Captures;

use super::{Outcome, Pattern, field_text};
use crate::record::{Record, Value};

pub(super) fn apply(pattern: &Pattern, mut record: Record) -> Outcome {
    let text = match field_text(&record, &pattern.from) {
        Ok(text) => text.into_owned(),
        Err(reason) => return Outcome::rejected("regex", reason),
    };

    match (&pattern.replace, &pattern.into) {
        (Some(replacement), _) => rewrite(pattern, replacement, &text, &mut record),
        (None, Some(into)) => extract_into(pattern, into, &text, &mut record),
        (None, None) => extract_named(pattern, &text, &mut record),
    }
    Outcome::Continue(record)
}

fn rewrite(pattern: &Pattern, replacement: &str, text: &str, record: &mut Record) {
    let rewritten = pattern.pattern.replace_all(text, replacement).into_owned();
    record.set_field(&pattern.from, Value::Text(rewritten));
}

/// The first group, or the whole match when the pattern has none.
fn extract_into(pattern: &Pattern, into: &str, text: &str, record: &mut Record) {
    let Some(captures) = pattern.pattern.captures(text) else {
        return;
    };
    if let Some(found) = captures.get(1).or_else(|| captures.get(0)) {
        record.set_field(into, typed(found.as_str()));
    }
}

fn extract_named(pattern: &Pattern, text: &str, record: &mut Record) {
    let Some(captures): Option<Captures<'_>> = pattern.pattern.captures(text) else {
        return;
    };
    for name in pattern.pattern.capture_names().flatten() {
        if let Some(found) = captures.name(name) {
            record.set_field(name, typed(found.as_str()));
        }
    }
}

/// Digits alone are a number, so a template can pad them; anything else stays text.
fn typed(found: &str) -> Value {
    let digits = (1..=18).contains(&found.len()) && found.bytes().all(|byte| byte.is_ascii_digit());
    match found.parse::<u64>() {
        Ok(number) if digits => Value::Number(number),
        _ => Value::Text(found.to_string()),
    }
}

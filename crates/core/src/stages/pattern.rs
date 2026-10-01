use super::{Batch, Flow, Pattern, Stage, Stop, fill_field, write_field, written_field};
use crate::context::Context;
use crate::record::{Record, Value};

impl Stage for Pattern {
    fn name(&self) -> &'static str {
        "regex"
    }

    fn run(&self, batch: Batch, _: &mut Context) -> Batch {
        batch.each(|record| apply(self, record))
    }
}

fn apply(pattern: &Pattern, mut record: Record) -> Flow {
    let text = written_field("regex", &record, &pattern.from)?;

    match (&pattern.replace, &pattern.into) {
        (Some(replacement), _) => rewrite(pattern, replacement, &text, &mut record),
        (None, Some(into)) => extract_into(pattern, into, &text, &mut record),
        (None, None) => extract_named(pattern, &text, &mut record),
    }?;
    Ok(record)
}

fn rewrite(
    pattern: &Pattern,
    replacement: &str,
    text: &str,
    record: &mut Record,
) -> Result<(), Stop> {
    let rewritten = pattern.pattern.replace_all(text, replacement).into_owned();
    write_field("regex", record, &pattern.from, Value::Text(rewritten))
}

/// The first group, or the whole match when the pattern has none.
fn extract_into(
    pattern: &Pattern,
    into: &str,
    text: &str,
    record: &mut Record,
) -> Result<(), Stop> {
    let Some(captures) = pattern.pattern.captures(text) else {
        return Ok(());
    };
    match captures.get(1).or_else(|| captures.get(0)) {
        Some(found) => fill_field("regex", record, into, typed(found.as_str())),
        None => Ok(()),
    }
}

fn extract_named(pattern: &Pattern, text: &str, record: &mut Record) -> Result<(), Stop> {
    let Some(captures) = pattern.pattern.captures(text) else {
        return Ok(());
    };
    for name in pattern.pattern.capture_names().flatten() {
        if let Some(found) = captures.name(name) {
            fill_field("regex", record, name, typed(found.as_str()))?;
        }
    }
    Ok(())
}

/// Digits alone are a number, so a template can pad them; anything else stays text.
fn typed(found: &str) -> Value {
    let digits = (1..=18).contains(&found.len()) && found.bytes().all(|byte| byte.is_ascii_digit());
    match found.parse::<u64>() {
        Ok(number) if digits => Value::Number(number),
        _ => Value::Text(found.to_string()),
    }
}

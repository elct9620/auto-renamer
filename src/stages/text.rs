use super::{Case, CaseKind, Fields, Flow, Replace, Strip, text_field, write_field};
use crate::record::{Record, Value};

pub(super) fn set(fields: &Fields, record: Record) -> Flow {
    write_all("set", fields, record, |_, _| true)
}

pub(super) fn default(fields: &Fields, record: Record) -> Flow {
    write_all("default", fields, record, |record, name| {
        record.field(name).is_none()
    })
}

fn write_all(
    stage: &str,
    fields: &Fields,
    mut record: Record,
    wanted: impl Fn(&Record, &str) -> bool,
) -> Flow {
    for (name, value) in fields {
        if wanted(&record, name) {
            write_field(stage, &mut record, name, value.clone())?;
        }
    }
    Ok(record)
}

pub(super) fn replace(replace: &Replace, record: Record) -> Flow {
    rewrite("replace", &replace.field, record, |text| {
        text.replace(&replace.find, &replace.with)
    })
}

pub(super) fn case(case: &Case, record: Record) -> Flow {
    rewrite("case", &case.field, record, |text| match case.to {
        CaseKind::Lower => text.to_lowercase(),
        CaseKind::Upper => text.to_uppercase(),
        CaseKind::Title => title_case(text),
    })
}

pub(super) fn strip(strip: &Strip, record: Record) -> Flow {
    rewrite("strip", &strip.field, record, |text| {
        let stripped = strip
            .groups
            .iter()
            .fold(text.to_string(), |text, &(open, close)| {
                remove_group(&text, open, close)
            });
        stripped.split_whitespace().collect::<Vec<_>>().join(" ")
    })
}

/// Rewrites a text field; a field that is missing or is not text is refused.
fn rewrite(
    stage: &str,
    field: &str,
    mut record: Record,
    change: impl FnOnce(&str) -> String,
) -> Flow {
    let rewritten = change(text_field(stage, &record, field)?);
    write_field(stage, &mut record, field, Value::Text(rewritten))?;
    Ok(record)
}

fn title_case(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut word_start = true;
    for ch in text.chars() {
        if ch.is_whitespace() {
            word_start = true;
            out.push(ch);
        } else if word_start {
            out.extend(ch.to_uppercase());
            word_start = false;
        } else {
            out.extend(ch.to_lowercase());
        }
    }
    out
}

/// Removes every outermost group from its opening to its closing character; a group never closed stays.
fn remove_group(text: &str, open: char, close: char) -> String {
    let mut removed: Vec<(usize, usize)> = Vec::new();
    let mut depth = 0usize;
    let mut start = 0usize;
    for (index, ch) in text.char_indices() {
        if ch == close && depth > 0 {
            depth -= 1;
            if depth == 0 {
                removed.push((start, index + ch.len_utf8()));
            }
        } else if ch == open {
            if depth == 0 {
                start = index;
            }
            depth += 1;
        }
    }

    let mut out = String::with_capacity(text.len());
    let mut cursor = 0;
    for (from, to) in removed {
        out.push_str(&text[cursor..from]);
        cursor = to;
    }
    out.push_str(&text[cursor..]);
    out
}

use super::{
    Batch, Case, CaseKind, DefaultFields, Fields, Flow, Replace, Run, SetFields, Strip, text_field,
    write_field,
};
use crate::context::Context;
use crate::record::{Record, Value};

impl Run for SetFields {
    fn name(&self) -> &'static str {
        "set"
    }

    fn run(&self, batch: &mut Batch, _: &mut Context) {
        batch.each(|record| write_all("set", &self.0, record, |_, _| true));
    }
}

impl Run for DefaultFields {
    fn name(&self) -> &'static str {
        "default"
    }

    fn run(&self, batch: &mut Batch, _: &mut Context) {
        batch.each(|record| {
            write_all("default", &self.0, record, |record, name| {
                record.field(name).is_none()
            })
        });
    }
}

impl Run for Replace {
    fn name(&self) -> &'static str {
        "replace"
    }

    fn run(&self, batch: &mut Batch, _: &mut Context) {
        batch.each(|record| {
            rewrite("replace", &self.field, record, |text| {
                text.replace(&self.find, &self.with)
            })
        });
    }
}

impl Run for Case {
    fn name(&self) -> &'static str {
        "case"
    }

    fn run(&self, batch: &mut Batch, _: &mut Context) {
        batch.each(|record| {
            rewrite("case", &self.field, record, |text| match self.to {
                CaseKind::Lower => text.to_lowercase(),
                CaseKind::Upper => text.to_uppercase(),
                CaseKind::Title => title_case(text),
            })
        });
    }
}

impl Run for Strip {
    fn name(&self) -> &'static str {
        "strip"
    }

    fn run(&self, batch: &mut Batch, _: &mut Context) {
        batch.each(|record| {
            rewrite("strip", &self.field, record, |text| {
                let stripped = self
                    .groups
                    .iter()
                    .fold(text.to_string(), |text, &(open, close)| {
                        remove_group(&text, open, close)
                    });
                stripped.split_whitespace().collect::<Vec<_>>().join(" ")
            })
        });
    }
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

use super::Outcome;
use crate::record::{Record, Value};
use crate::template::Template;

pub(super) fn apply(template: &Template, mut record: Record) -> Outcome {
    let name = match template.render(&record) {
        Ok(name) => name,
        Err(error) => return Outcome::rejected("format", error.to_string()),
    };
    if name.is_empty() || name == "." || name == ".." || name.contains(['/', '\0']) {
        return Outcome::rejected("format", format!("`{name}` is not a usable file name"));
    }

    let file_name = match record.field("ext") {
        Some(Value::Text(ext)) if !ext.is_empty() => format!("{name}.{ext}"),
        _ => name,
    };
    let plan = record.plan().with_file_name(file_name);
    record.set_plan(plan);
    Outcome::Continue(record)
}

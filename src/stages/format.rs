use super::{Outcome, write_field};
use crate::record::{Record, Value};
use crate::template::Template;

pub(super) fn apply(template: &Template, mut record: Record) -> Outcome {
    let name = match template.render(&record) {
        Ok(name) => name,
        Err(error) => return Outcome::rejected("format", error.to_string()),
    };
    match write_field("format", &mut record, "name", Value::Text(name)) {
        Ok(()) => Outcome::Continue(record),
        Err(refused) => refused,
    }
}

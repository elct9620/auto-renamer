use super::{Flow, Stop, write_field};
use crate::record::{Record, Value};
use crate::template::Template;

pub(super) fn apply(template: &Template, mut record: Record) -> Flow {
    let name = template
        .render(&record)
        .map_err(|error| Stop::rejected("format", error.to_string()))?;
    write_field("format", &mut record, "name", Value::Text(name))?;
    Ok(record)
}

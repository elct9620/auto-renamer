use super::{Flow, refused_by, write_field};
use crate::record::{Record, Value};
use crate::template::Template;

pub(super) fn apply(template: &Template, mut record: Record) -> Flow {
    let name = template.render(&record).map_err(refused_by("format"))?;
    write_field("format", &mut record, "name", Value::Text(name))?;
    Ok(record)
}

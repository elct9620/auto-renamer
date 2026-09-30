use super::{Filter, Flow, Stop};
use crate::record::{Record, Value};

pub(super) fn apply(filter: &Filter, record: Record) -> Flow {
    let name = text(&record, "name");
    let ext = text(&record, "ext");
    let file_name = if ext.is_empty() {
        name
    } else {
        format!("{name}.{ext}")
    };

    let ext_matches = filter.ext.is_empty() || filter.ext.contains(&ext.to_lowercase());
    let glob_matches = filter
        .glob
        .as_ref()
        .is_none_or(|glob| glob.is_match(&file_name));

    if (ext_matches && glob_matches) != filter.invert {
        Ok(record)
    } else {
        Err(Stop::Excluded)
    }
}

fn text(record: &Record, field: &str) -> String {
    match record.field(field) {
        Some(Value::Text(text)) => text.clone(),
        _ => String::new(),
    }
}

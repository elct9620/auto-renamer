use super::{Batch, Filter, Stage, Stop};
use crate::context::Context;
use crate::record::{Record, Value};

impl Stage for Filter {
    fn name(&self) -> &'static str {
        "filter"
    }

    fn run(&self, batch: &mut Batch, _: &mut Context) {
        batch.each(|record| {
            if self.accepts(&record) {
                Ok(record)
            } else {
                Err(Stop::Excluded)
            }
        });
    }
}

impl Filter {
    /// Whether the filter lets the file through.
    pub(crate) fn accepts(&self, record: &Record) -> bool {
        let name = text(record, "name");
        let ext = text(record, "ext");
        let file_name = if ext.is_empty() {
            name
        } else {
            format!("{name}.{ext}")
        };

        let ext_matches = self.ext.is_empty() || self.ext.contains(&ext.to_lowercase());
        let glob_matches = self
            .glob
            .as_ref()
            .is_none_or(|glob| glob.is_match(&file_name));

        (ext_matches && glob_matches) != self.invert
    }
}

fn text(record: &Record, field: &str) -> String {
    match record.field(field) {
        Some(Value::Text(text)) => text.clone(),
        _ => String::new(),
    }
}

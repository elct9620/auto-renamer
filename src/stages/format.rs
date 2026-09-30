use super::{Batch, Format, Run, refused_by, write_field};
use crate::context::Context;
use crate::record::Value;

impl Run for Format {
    fn name(&self) -> &'static str {
        "format"
    }

    fn run(&self, batch: &mut Batch, _: &mut Context) {
        batch.each(|mut record| {
            let name = self.0.render(&record).map_err(refused_by("format"))?;
            write_field("format", &mut record, "name", Value::Text(name))?;
            Ok(record)
        });
    }
}

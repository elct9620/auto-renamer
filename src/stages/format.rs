use super::{Batch, Format, Stage, refused_by, write_field};
use crate::context::Context;
use crate::record::Value;

impl Stage for Format {
    fn name(&self) -> &'static str {
        "format"
    }

    fn run(&self, batch: Batch, _: &mut Context) -> Batch {
        batch.each(|mut record| {
            let name = self.0.render(&record).map_err(refused_by("format"))?;
            write_field("format", &mut record, "name", Value::Text(name))?;
            Ok(record)
        })
    }
}

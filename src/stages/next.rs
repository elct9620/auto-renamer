use super::{Next, Outcome};
use crate::record::Record;

pub(super) fn apply(next: &Next, record: Record) -> Outcome {
    if record.field(&next.into).is_some() {
        return Outcome::Continue(record);
    }
    Outcome::rejected(
        "next",
        "reading the target for the next number is not available yet",
    )
}

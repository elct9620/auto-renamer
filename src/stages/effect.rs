//! The stages that touch the filesystem, as far as planning is concerned.

use super::{Batch, Cleanup, Move, Run};
use crate::context::Context;

impl Run for Move {
    fn name(&self) -> &'static str {
        "move"
    }

    /// An effect does not rewrite the plan, so planning passes every record on.
    fn run(&self, _: &mut Batch, _: &mut Context) {}
}

impl Run for Cleanup {
    fn name(&self) -> &'static str {
        "cleanup"
    }

    /// An effect does not rewrite the plan, so planning passes every record on.
    fn run(&self, _: &mut Batch, _: &mut Context) {}
}

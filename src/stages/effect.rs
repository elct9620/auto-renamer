//! The stages that touch the filesystem, as far as planning is concerned: each only notes what it asks for.

use super::{Batch, Cleanup, Move, Run};
use crate::context::Context;

/// What an effect stage asks to be done to a planned file, as the stage was declared.
#[derive(Debug, Clone, Copy)]
pub enum Effect<'p> {
    Move(&'p Move),
    Cleanup(&'p Cleanup),
}

impl Run for Move {
    fn name(&self) -> &'static str {
        "move"
    }

    fn run<'p>(&'p self, batch: &mut Batch<'p>, _: &mut Context) {
        batch.schedule(Effect::Move(self));
    }
}

impl Run for Cleanup {
    fn name(&self) -> &'static str {
        "cleanup"
    }

    fn run<'p>(&'p self, batch: &mut Batch<'p>, _: &mut Context) {
        batch.schedule(Effect::Cleanup(self));
    }
}

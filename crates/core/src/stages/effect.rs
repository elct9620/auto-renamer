//! The stages that touch the filesystem, as far as planning is concerned: each only notes what it asks for.

use super::{Batch, Cleanup, Move, Stage};
use crate::context::Context;

/// What an effect stage asks to be done to a planned file, with the settings the stage was declared with.
#[derive(Debug, Clone)]
pub enum Effect {
    Move(Move),
    Cleanup(Cleanup),
}

impl Stage for Move {
    fn name(&self) -> &'static str {
        "move"
    }

    fn run(&self, batch: Batch, _: &mut Context) -> Batch {
        batch.schedule(Effect::Move(self.clone()))
    }
}

impl Stage for Cleanup {
    fn name(&self) -> &'static str {
        "cleanup"
    }

    fn run(&self, batch: Batch, _: &mut Context) -> Batch {
        batch.schedule(Effect::Cleanup(self.clone()))
    }
}

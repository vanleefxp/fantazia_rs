use std::cmp::Ordering;
use malachite_base::num::arithmetic::traits::Sign;

use crate::edo12::Pitch;

impl Sign for Pitch {
    fn sign(&self) -> Ordering {
        self.step.0.sign().then_with(|| self.tone.sign())
    }
}
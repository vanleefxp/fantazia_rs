use std::cmp::Ordering;
use malachite_base::num::arithmetic::traits::Sign;

use super::super::{Pitch, Interval, IntervalQual};

impl Sign for Pitch {
    fn sign(&self) -> Ordering {
        self.step.0.sign().then_with(|| self.tone.sign())
    }
}

impl Sign for Interval {
    fn sign(&self) -> Ordering {
        self.deg.0.sign().then_with(|| self.qual.partial_cmp(&IntervalQual::Perfect).unwrap())
    }
}
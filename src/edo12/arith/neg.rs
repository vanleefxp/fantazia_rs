use std::ops::Neg;

use malachite_base::num::arithmetic::traits::NegAssign;

use super::super::{Acci, OPitch, Step};

impl Neg for Step {
    type Output = Self;

    fn neg(self) -> Self::Output {
        (7 - self as u8).try_into().unwrap()
    }
}

impl NegAssign for Acci {
    fn neg_assign(&mut self) {
        self.0.neg_assign();
    }
}

#[inline]
fn get_negated_step_and_tone(p: &OPitch) -> (Step, i8) {
    (-p.step, 12 - p.tone)
}

impl Neg for OPitch {
    type Output = Self;

    fn neg(self) -> Self::Output {
        let (step, tone) = get_negated_step_and_tone(&self);
        OPitch { step, tone }
    }
}

impl NegAssign for OPitch {
    fn neg_assign(&mut self) {
        let (step, tone) = get_negated_step_and_tone(self);
        self.step = step;
        self.tone = tone;
    }
}

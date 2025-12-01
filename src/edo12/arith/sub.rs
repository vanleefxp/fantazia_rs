use std::ops::Sub;

use super::super::{OPitch, SimpleInterval, OStep};
use crate::impl_sub_assign_by_sub;

impl Sub for OStep {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        (self as u8 + (7 - rhs as u8)).try_into().unwrap()
    }
}

#[inline]
fn get_subtracted_step_and_tone(p1: &OPitch, p2: &OPitch) -> (OStep, i8) {
    if p1.step >= p2.step {
        let step = (p1.step as u8 - p2.step as u8).try_into().unwrap();
        let tone = p1.tone - p2.tone;
        (step, tone)
    } else {
        let step = (p1.step as u8 + (7 - p2.step as u8)).try_into().unwrap();
        let tone = p1.tone - p2.tone + 12;
        (step, tone)
    }
}

impl Sub for OPitch {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        let (step, tone) = get_subtracted_step_and_tone(&self, &rhs);
        OPitch { step, tone }
    }
}

impl Sub for SimpleInterval {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        let this_pitch = OPitch::from(self);
        let rhs_pitch = OPitch::from(rhs);
        (this_pitch - rhs_pitch).into()
    }
}

impl_sub_assign_by_sub!(OPitch, SimpleInterval);

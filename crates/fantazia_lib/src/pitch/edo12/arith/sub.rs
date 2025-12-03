use std::ops::Sub;

use super::super::{OPitch, OStep, OInterval};
use crate::impl_sub_assign_by_sub;

impl Sub for OStep {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        (self as u8 + (7 - rhs as u8)).try_into().unwrap()
    }
}

impl Sub for OPitch {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        if self.step >= rhs.step {
            let step = (self.step as u8 - rhs.step as u8).try_into().unwrap();
            let tone = self.tone - rhs.tone;
            OPitch { step, tone }
        } else {
            let step = (self.step as u8 + (7 - rhs.step as u8)).try_into().unwrap();
            let tone = self.tone - rhs.tone + 12;
            OPitch { step, tone }
        }
    }
}

impl Sub for OInterval {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        let this_pitch = OPitch::from(self);
        let rhs_pitch = OPitch::from(rhs);
        (this_pitch - rhs_pitch).into()
    }
}

impl_sub_assign_by_sub!(OPitch, OInterval);

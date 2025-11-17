use std::ops::{Add, AddAssign};

use super::super::{OPitch, Step};
use crate::impl_sum_bisect;

impl Add for Step {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        ((self as u8 + rhs as u8) % 7).try_into().unwrap()
    }
}

#[inline]
fn get_added_step_and_tone(p1: &OPitch, p2: &OPitch) -> (Step, i8) {
    let mut step = p1.step as u8 + p2.step as u8;
    let mut tone = p1.tone + p2.tone;
    if step > 7 {
        tone -= 12;
        step -= 7;
    }
    let step = step.try_into().unwrap();
    (step, tone)
}

impl Add for OPitch {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        let (step, tone) = get_added_step_and_tone(&self, &rhs);
        OPitch { step, tone }
    }
}

impl Add<&Self> for OPitch {
    type Output = Self;

    fn add(self, rhs: &Self) -> Self::Output {
        let (step, tone) = get_added_step_and_tone(&self, rhs);
        OPitch { step, tone }
    }
}

impl Add<OPitch> for &OPitch {
    type Output = OPitch;

    fn add(self, rhs: OPitch) -> Self::Output {
        let (step, tone) = get_added_step_and_tone(self, &rhs);
        OPitch { step, tone }
    }
}

impl Add for &OPitch {
    type Output = OPitch;

    fn add(self, rhs: Self) -> Self::Output {
        let (step, tone) = get_added_step_and_tone(self, rhs);
        OPitch { step, tone }
    }
}

impl AddAssign for OPitch {
    fn add_assign(&mut self, rhs: Self) {
        let (step, tone) = get_added_step_and_tone(self, &rhs);
        self.step = step;
        self.tone = tone;
    }
}

impl AddAssign<&Self> for OPitch {
    fn add_assign(&mut self, rhs: &Self) {
        let (step, tone) = get_added_step_and_tone(self, rhs);
        self.step = step;
        self.tone = tone;
    }
}

impl_sum_bisect!(OPitch, OPitch::C);

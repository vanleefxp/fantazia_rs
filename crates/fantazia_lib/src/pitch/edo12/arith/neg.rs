use std::ops::Neg;

use malachite_base::num::arithmetic::traits::NegAssign;

use crate::pitch::edo12::OIntervalDeg;

use super::super::{Acci, IntervalDeg, OPitch, OStep, OInterval, Step, Interval, IntervalQual, Pitch};

macro_rules! derive_neg_assign_from_neg {
    ($($t:ty),*$(,)?) => {
        $(
            impl NegAssign for $t {
                fn neg_assign(&mut self) {
                    *self = -*self;
                }
            }
        )*
    };
}

macro_rules! derive_neg_assign_from_field {
    ($($t:ty),*$(,)?) => {
        $(
            impl NegAssign for $t {
                fn neg_assign(&mut self) {
                    self.0.neg_assign();
                }
            }
        )*
    };
}

impl Neg for OStep {
    type Output = Self;

    fn neg(self) -> Self::Output {
        use OStep::*;
        match self {
            C => self,
            step => (7 - step as u8).try_into().unwrap(),
        }
    }
}

impl Neg for OIntervalDeg {
    type Output = Self;

    fn neg(self) -> Self::Output {
        OStep::from(self).neg().into()
    }
}

impl Neg for OPitch {
    type Output = Self;

    fn neg(self) -> Self::Output {
        use OStep::*;
        match self.step {
            C => OPitch {
                step: C,
                tone: -self.tone,
            },
            step => OPitch {
                step: (7 - step as u8).try_into().unwrap(),
                tone: 12 - self.tone,
            },
        }
    }
}

impl Neg for Interval {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Pitch::from(self).neg().into()
    }
}

impl Neg for IntervalQual {
    type Output = Self;

    fn neg(self) -> Self::Output {
        use IntervalQual::*;
        match self {
            Augmented(n) => Diminished(n),
            Major => Minor,
            Perfect => Perfect,
            Minor => Major,
            Diminished(n) => Augmented(n),
        }
    }
}

impl NegAssign for Pitch {
    fn neg_assign(&mut self) {
        self.step.neg_assign();
        self.tone.neg_assign();
    }
}

derive_neg_assign_from_neg!(OPitch, OInterval, Interval, IntervalQual);
derive_neg_assign_from_field!(Step, Acci, IntervalDeg);

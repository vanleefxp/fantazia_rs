use std::ops::Neg;

use malachite_base::num::arithmetic::traits::NegAssign;

use crate::edo12::{Interval, Pitch};

use super::super::{Acci, IntervalDeg, OPitch, OStep, SimpleInterval, Step};

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

macro_rules! derive_neg_assign {
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
        OPitch::from(self).neg().into()
    }
}

impl Neg for SimpleInterval {
    type Output = Self;

    fn neg(self) -> Self::Output {
        OPitch::from(self).neg().into()
    }
}

impl NegAssign for Pitch {
    fn neg_assign(&mut self) {
        self.step.neg_assign();
        self.tone.neg_assign();
    }
}

derive_neg_assign_from_neg!(OPitch, SimpleInterval, Interval);
derive_neg_assign!(Step, Acci, IntervalDeg);

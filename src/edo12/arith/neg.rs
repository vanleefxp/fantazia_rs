use std::ops::Neg;

use malachite_base::num::arithmetic::traits::NegAssign;

use super::super::{Acci, OPitch, SimpleInterval, Step};

impl Neg for Step {
    type Output = Self;

    fn neg(self) -> Self::Output {
        use Step::*;
        match self {
            C => self,
            step => (7 - step as u8).try_into().unwrap(),
        }
    }
}

impl NegAssign for Acci {
    fn neg_assign(&mut self) {
        self.0.neg_assign();
    }
}

#[inline]
fn opitch_neg(p: OPitch) -> OPitch {
    use Step::*;
    match p.step {
        C => OPitch {
            step: C,
            tone: -p.tone,
        },
        step => OPitch {
            step: (7 - step as u8).try_into().unwrap(),
            tone: 12 - p.tone,
        },
    }
}

impl Neg for OPitch {
    type Output = Self;

    fn neg(self) -> Self::Output {
        opitch_neg(self)
    }
}

impl NegAssign for OPitch {
    fn neg_assign(&mut self) {
        *self = opitch_neg(*self);
    }
}

impl Neg for SimpleInterval {
    type Output = Self;

    fn neg(self) -> Self::Output {
        OPitch::from(self).neg().into()
    }
}

impl NegAssign for SimpleInterval {
    fn neg_assign(&mut self) {
        *self = -*self
    }
}

use std::{iter::Sum, ops::Add};

use malachite_base::num::{
    arithmetic::traits::{CheckedAdd, ModAdd},
    basic::traits::Zero as _,
};

use super::super::{OInterval, OIntervalDeg, OPitch, OStep};
use crate::{impl_add_assign_by_add, impl_sum_bisect, pitch::edo12::Acci};

macro_rules! impl_add_by_mod {
    ($modulus: expr, $repr_t:ty; $($t:ty),* $(,)?) => {
        $(
            impl Add<$t> for $repr_t {
                type Output = Self;
                fn add(self, rhs: $t) -> Self::Output {
                    ((self as $repr_t).mod_add(rhs as $repr_t, $modulus)).try_into().unwrap()
                }
            }
        )*
    };
}

impl_add_by_mod!(7, u8; OStep, OIntervalDeg);

impl Add for OPitch {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        let mut step = self.step as u8 + rhs.step as u8;
        let mut tone = self.tone + rhs.tone;
        if step > 7 {
            tone -= 12;
            step -= 7;
        }
        let step = step.try_into().unwrap();
        OPitch { step, tone }
    }
}

impl Add<OStep> for OPitch {
    type Output = Self;
    fn add(self, rhs: OStep) -> Self::Output {
        let mut result = self;
        match self.step as u8 + rhs {
            n @ 7.. => {
                result.step = (n - 7).try_into().unwrap();
                result.tone -= 12;
            }
            n => {
                result.step = n.try_into().unwrap();
            }
        }
        result
    }
}

impl Add<Acci> for OPitch {
    type Output = Self;

    fn add(self, rhs: Acci) -> Self::Output {
        let mut result = self;
        result.tone += rhs.0;
        result
    }
}

impl Add for OInterval {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        (OPitch::from(self) + OPitch::from(rhs)).into()
    }
}

impl Add<OStep> for OInterval {
    type Output = Self;

    fn add(self, rhs: OStep) -> Self::Output {
        (OPitch::from(self) + rhs).into()
    }
}

impl Add<Acci> for OInterval {
    type Output = Self;

    fn add(self, rhs: Acci) -> Self::Output {
        (OPitch::from(self) + rhs).into()
    }
}

impl CheckedAdd for OPitch {
    type Output = Self;

    fn checked_add(self, rhs: Self) -> Option<Self::Output> {
        let mut step = self.step as u8 + rhs.step as u8;
        let mut tone = self.tone.checked_add(rhs.tone)?;
        if step > 7 {
            tone = tone.checked_sub(12)?;
            step -= 7;
        }
        let step = step.try_into().unwrap();
        Some(OPitch { step, tone })
    }
}

impl CheckedAdd<OStep> for OPitch {
    type Output = Self;

    fn checked_add(self, other: OStep) -> Option<Self::Output> {
        let mut result = self;
        match self.step as u8 + other {
            n @ 7.. => {
                result.step = (n - 7).try_into().unwrap();
                result.tone = result.tone.checked_sub(12)?;
            }
            n => {
                result.step = n.try_into().unwrap();
            }
        }
        Some(result)
    }
}

impl CheckedAdd<Acci> for OPitch {
    type Output = Self;

    fn checked_add(self, other: Acci) -> Option<Self::Output> {
        let mut result = self;
        result.tone = result.tone.checked_add(other.0)?;
        Some(result)
    }
}

impl CheckedAdd for OInterval {
    type Output = Self;

    fn checked_add(self, other: Self) -> Option<Self::Output> {
        OPitch::from(self)
            .checked_add(OPitch::from(other))
            .map(Into::into)
    }
}

impl CheckedAdd<Acci> for OInterval {
    type Output = Self;

    fn checked_add(self, other: Acci) -> Option<Self::Output> {
        OPitch::from(self).checked_add(other).map(Into::into)
    }
}

impl Sum for OInterval {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.map(OPitch::from).sum::<OPitch>().into()
    }
}

impl_add_assign_by_add!(OPitch, OInterval);
impl_sum_bisect!(OPitch, OPitch::ZERO);

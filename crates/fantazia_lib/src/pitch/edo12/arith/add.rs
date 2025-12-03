use std::{iter::Sum, ops::Add};

use malachite_base::num::{
    arithmetic::traits::{CheckedAdd, ModAdd},
    basic::traits::Zero as _,
};

use super::super::{OInterval, OIntervalDeg, OPitch, OStep};
use crate::{impl_add_assign_by_add, impl_add_by_conversion, impl_sum_bisect};

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

impl_add_by_conversion!(OInterval, OPitch);

impl Sum for OInterval {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.map(OPitch::from).sum::<OPitch>().into()
    }
}

impl_add_assign_by_add!(OPitch, OInterval);
impl_sum_bisect!(OPitch, OPitch::ZERO);

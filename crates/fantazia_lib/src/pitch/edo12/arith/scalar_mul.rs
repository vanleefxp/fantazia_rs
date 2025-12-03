
use std::{ops::{Mul, MulAssign}, cmp::Ordering};

use malachite_base::num::arithmetic::traits::{DivMod, ModMul as _, Sign, CheckedMul};

use crate::pitch::edo12::{Acci, OPitch, Step, OStep, Pitch};

impl Mul<i8> for OStep {
    type Output = Self;

    fn mul(self, rhs: i8) -> Self::Output {
        use Ordering::*;
        match rhs.sign() {
            Greater => (self as u8).mod_mul(rhs as u8, 7).try_into().unwrap(),
            Equal => OStep::C,
            Less => -(OStep::try_from((self as u8).mod_mul((-rhs) as u8, 7)).unwrap()),
        }
    }
}

impl Mul<u8> for OStep {
    type Output = Self;

    fn mul(self, rhs: u8) -> Self::Output {
        (self as u8).mod_mul(rhs, 7).try_into().unwrap()
    }
}

macro_rules! derive_scalar_mul {
    ($($t:ty, $scalar_t:ty);*$(;)?) => {
        $(
            impl Mul<$scalar_t> for $t {
                type Output = Self;
                fn mul(self, rhs: $scalar_t) -> Self::Output {
                    Self::from(self.0 * rhs)
                }
            }

            impl CheckedMul<$scalar_t> for $t {
                type Output = Self;
                fn checked_mul(self, rhs: $scalar_t) -> Option<Self::Output> {
                    Some(Self::from(self.0.checked_mul(rhs)?))
                }
            }

            impl MulAssign<$scalar_t> for $t {
                fn mul_assign(&mut self, rhs: $scalar_t) {
                    self.0 *= rhs;
                }
            }
        )*
    };
}

macro_rules! derive_mul_assign_from_mul {
    ($($t:ty, $scalar_t:ty);*$(;)?) => {
        $(
            impl MulAssign<$scalar_t> for $t {
                fn mul_assign(&mut self, rhs: $scalar_t) {
                    *self = *self * rhs;
                }
            }
        )*
    };
}

derive_scalar_mul!(Acci, i8; Step, i8);

impl Mul<i8> for OPitch {
    type Output = Self;
    fn mul(self, rhs: i8) -> Self::Output {
        let step = self.step as i16 * rhs as i16;
        let (octave, step) = step.div_mod(7);
        let step: OStep = (step as u8).try_into().unwrap();
        let tone = (self.tone as i16 * rhs as i16 - octave * 12) as i8;
        OPitch { step, tone }
    }
}

impl CheckedMul<i8> for OPitch {
    type Output = Self;
    fn checked_mul(self, rhs: i8) -> Option<Self::Output> {
        let step = self.step as i16 * rhs as i16;
        let (octave, step) = step.div_mod(7);
        let step: OStep = (step as u8).try_into().unwrap();
        let tone = i8::try_from((self.tone as i16 * rhs as i16).checked_sub(octave.checked_mul(12)?)?).ok()?;
        Some(OPitch { step, tone })
    }
}

derive_mul_assign_from_mul!(OPitch, i8; OStep, i8; OStep, u8);

impl Mul<i8> for Pitch {
    type Output = Self;

    fn mul(self, rhs: i8) -> Self::Output {
        Pitch {
            step: self.step * rhs,
            tone: self.tone * rhs,
        }
    }
}

impl MulAssign<i8> for Pitch {
    fn mul_assign(&mut self, rhs: i8) {
        self.step *= rhs;
        self.tone *= rhs;
    }
}

impl CheckedMul<i8> for Pitch {
    type Output = Self;

    fn checked_mul(self, other: i8) -> Option<Self::Output> {
        Some(Pitch {
            step: self.step.checked_mul(other)?,
            tone: self.tone.checked_mul(other)?,
        })
    }

}
use std::{i8, ops::Sub};

use malachite_base::num::arithmetic::traits::CheckedSub;

use super::super::{Acci, OInterval, OPitch, OStep};
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

impl Sub<OStep> for OPitch {
    type Output = Self;

    fn sub(self, rhs: OStep) -> Self::Output {
        let mut result = self;
        if result.step >= rhs {
            result.step = (result.step as u8 - rhs as u8).try_into().unwrap();
        } else {
            result.step = (result.step as u8 + (7 - rhs as u8)).try_into().unwrap();
            result.tone += 12;
        }
        result
    }
}

impl Sub<Acci> for OPitch {
    type Output = Self;
    fn sub(self, rhs: Acci) -> Self::Output {
        let mut result = self;
        result.tone -= rhs.0;
        result
    }
}

impl Sub for OInterval {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        (OPitch::from(self) - OPitch::from(rhs)).into()
    }
}

impl Sub<OStep> for OInterval {
    type Output = Self;
    fn sub(self, rhs: OStep) -> Self::Output {
        (OPitch::from(self) - rhs).into()
    }
}

impl Sub<Acci> for OInterval {
    type Output = Self;
    fn sub(self, rhs: Acci) -> Self::Output {
        (OPitch::from(self) - rhs).into()
    }
}

impl CheckedSub for OPitch {
    type Output = Self;
    fn checked_sub(self, rhs: Self) -> Option<Self::Output> {
        if self.step >= rhs.step {
            let step = (self.step as u8 - rhs.step as u8).try_into().unwrap();
            let tone = self.tone.checked_sub(rhs.tone)?;
            Some(OPitch { step, tone })
        } else {
            let step = (self.step as u8 + (7 - rhs.step as u8)).try_into().unwrap();
            let tone = if self.tone < i8::MAX - 12 {
                (self.tone + 12).checked_sub(rhs.tone)?
            } else {
                self.tone.checked_sub(rhs.tone)?.checked_add(12)?
            };
            Some(OPitch { step, tone })
        }
    }
}

impl CheckedSub<OStep> for OPitch {
    type Output = Self;
    fn checked_sub(self, rhs: OStep) -> Option<Self::Output> {
        let mut result = self;
        if result.step >= rhs {
            result.step = (result.step as u8 - rhs as u8).try_into().unwrap();
        } else {
            result.step = (result.step as u8 + (7 - rhs as u8)).try_into().unwrap();
            result.tone = result.tone.checked_add(12)?;
        }
        Some(result)
    }
}

impl CheckedSub<Acci> for OPitch {
    type Output = Self;
    fn checked_sub(self, rhs: Acci) -> Option<Self::Output> {
        let mut result = self;
        result.tone = result.tone.checked_sub(rhs.0)?;
        Some(result)
    }
}

impl CheckedSub for OInterval {
    type Output = Self;
    fn checked_sub(self, rhs: Self) -> Option<Self::Output> {
        OPitch::from(self)
            .checked_sub(OPitch::from(rhs))
            .map(Into::into)
    }
}

impl CheckedSub<OStep> for OInterval {
    type Output = Self;
    fn checked_sub(self, rhs: OStep) -> Option<Self::Output> {
        OPitch::from(self).checked_sub(rhs).map(Into::into)
    }
}

impl CheckedSub<Acci> for OInterval {
    type Output = Self;
    fn checked_sub(self, rhs: Acci) -> Option<Self::Output> {
        OPitch::from(self).checked_sub(rhs).map(Into::into)
    }
}

impl_sub_assign_by_sub!(OPitch, OInterval);

use malachite_base::num::arithmetic::traits::AbsDiff;

use super::super::OPitch;

impl AbsDiff for OPitch {
    type Output = Self;

    fn abs_diff(self, other: Self) -> Self::Output {
        use std::cmp::Ordering::*;
        match self.cmp(&other) {
            Less => other - self,
            Greater => self - other,
            Equal => Self::C,
        }
    }
}

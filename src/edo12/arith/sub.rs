use std::ops::Sub;

use super::super::Step;

impl Sub for Step {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        (self as u8 + (7 - rhs as u8)).try_into().unwrap()
    }
}

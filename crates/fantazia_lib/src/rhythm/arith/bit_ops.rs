use std::ops::{Shl, ShlAssign, Shr, ShrAssign};

use malachite_base::num::arithmetic::traits::{ArithmeticCheckedShl, ArithmeticCheckedShr};

use super::super::BinaryDuration;

impl ShlAssign<i8> for BinaryDuration {
    fn shl_assign(&mut self, rhs: i8) {
        self.kind += rhs;
    }
}

impl ShrAssign<i8> for BinaryDuration {
    fn shr_assign(&mut self, rhs: i8) {
        self.kind -= rhs;
    }
}

impl Shl<i8> for BinaryDuration {
    type Output = Self;

    fn shl(self, rhs: i8) -> Self::Output {
        let mut result = self;
        result <<= rhs;
        result
    }
}

impl Shr<i8> for BinaryDuration {
    type Output = Self;

    fn shr(self, rhs: i8) -> Self::Output {
        let mut result = self;
        result >>= rhs;
        result
    }
}

impl ArithmeticCheckedShl<i8> for BinaryDuration {
    type Output = Self;

    fn arithmetic_checked_shl(self, rhs: i8) -> Option<Self::Output> {
        let mut result = self;
        result.kind = result.kind.checked_add(rhs)?;
        Some(result)
    }
}

impl ArithmeticCheckedShr<i8> for BinaryDuration {
    type Output = Self;

    fn arithmetic_checked_shr(self, rhs: i8) -> Option<Self::Output> {
        let mut result = self;
        result.kind = result.kind.checked_sub(rhs)?;
        Some(result)
    }
}

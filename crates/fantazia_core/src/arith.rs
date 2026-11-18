//! Additional arithmetic traits.

// Taken from `malachite_base`
// below is the license of the original code

// Copyright © 2025 Mikhail Hogrefe
//
// This file is part of Malachite.
//
// Malachite is free software: you can redistribute it and/or modify it under the terms of the GNU
// Lesser General Public License (LGPL) as published by the Free Software Foundation; either version
// 3 of the License, or (at your option) any later version. See <https://www.gnu.org/licenses/>.

use std::cmp::Ordering::{self, *};

/// Replaces a number with its negative. Assumes the result is representable.
pub const trait NegAssign {
    /// Performs the negate-assign operation.
    fn neg_assign(&mut self);
}

/// Takes the absolute value of a number. Assumes that the number has a representable absolute
/// number.
pub const trait Abs {
    /// The resulting type after applying the `.abs` operation.
    type Output;
    /// Performs the `.abs` operation.
    fn abs(self) -> Self::Output;
}

/// Replaces a number with its absolute value. Assumes that the number has a representable absolute
/// number.
pub const trait AbsAssign {
    /// Performs the abs-assign operation.
    fn abs_assign(&mut self);
}

/// Returns `Greater`, `Equal`, or `Less`, depending on whether a number is positive, zero, or
/// negative, respectively.
pub const trait Sign {
    /// Returns the sign of the value.
    fn sign(&self) -> Ordering;
}

/// Adds two numbers, returning `None` if the result is not representable.
pub const trait CheckedAdd<RHS = Self> {
    /// The resulting type after applying the `+` operator.
    type Output;
    /// Performs the checked `+` operation.
    fn checked_add(self, other: RHS) -> Option<Self::Output>;
}

/// Multiplies two numbers, returning `None` if the result is not representable.
pub const trait CheckedMul<RHS = Self> {
    /// The resulting type after applying the `*` operator.
    type Output;
    /// Performs the checked `*` operation.
    fn checked_mul(self, other: RHS) -> Option<Self::Output>;
}

/// Negates a number, returning `None` if the result is not representable.
pub const trait CheckedNeg {
    /// The resulting type after applying the unary `-` operator.
    type Output;
    /// Performs the checked unary `-` operation.
    fn checked_neg(self) -> Option<Self::Output>;
}

/// Subtracts two numbers, returning `None` if the result is not representable.
pub const trait CheckedSub<RHS = Self> {
    /// The resulting type after applying the `-` operator.
    type Output;
    /// Performs the checked `-` operation.
    fn checked_sub(self, other: RHS) -> Option<Self::Output>;
}

// MARK: Impl

macro_rules! apply_to_all {
    ($m:ident; $($t:ident),*$(,)?) => {
        $($m!($t);)*
    };
}
macro_rules! apply_to_primitive_ints {
    ($m:ident) => {
        apply_to_all!($m; u8, i8, u16, i16, u32, i32, u64, i64, u128, i128);
    }
}
macro_rules! apply_to_signed {
    ($m:ident) => {
        apply_to_all!($m; i8, i16, i32, i64, i128);
    }
}
macro_rules! apply_to_primitive_floats {
    ($m:ident) => {
        apply_to_all!($m; f16, f32, f64, f128);
    }
}

macro_rules! impl_sign_primitive_int {
    ($t:ident) => {
        const impl Sign for $t {
            /// Compares a number to zero.
            ///
            /// Returns `Greater`, `Equal`, or `Less`, depending on whether the number is positive,
            /// zero, or negative, respectively.
            ///
            /// # Worst-case complexity
            /// Constant time and additional memory.
            ///
            /// # Examples
            /// See [here](super::sign#sign).
            #[inline]
            fn sign(&self) -> Ordering {
                self.cmp(&0)
            }
        }
    };
}
apply_to_primitive_ints!(impl_sign_primitive_int);

macro_rules! impl_sign_primitive_float {
    ($t:ident) => {
        const impl Sign for $t {
            /// Compares a number to zero.
            ///
            /// - Positive finite numbers, positive zero, and positive infinity have sign `Greater`.
            /// - Negative finite numbers, negative zero, and negative infinity have sign `Less`.
            /// - `NaN` has sign `Equal`.
            ///
            /// # Worst-case complexity
            /// Constant time and additional memory.
            ///
            /// # Examples
            /// See [here](super::sign#sign).
            #[inline]
            fn sign(&self) -> Ordering {
                if self.is_nan() {
                    Equal
                } else if self.is_sign_positive() {
                    Greater
                } else {
                    Less
                }
            }
        }
    };
}
apply_to_primitive_floats!(impl_sign_primitive_float);

macro_rules! impl_abs_primitive_int {
    ($s:ident) => {
        const impl Abs for $s {
            type Output = $s;

            /// This is a wrapper over the `abs` functions in the standard library, for example
            /// [this one](i32::abs).
            #[inline]
            fn abs(self) -> $s {
                $s::abs(self)
            }
        }

        const impl AbsAssign for $s {
            /// Replaces a number with its absolute value.
            ///
            /// $x \gets |x|$.
            ///
            /// # Worst-case complexity
            /// Constant time and additional memory.
            ///
            /// # Examples
            /// See [here](super::abs#abs_assign).
            #[inline]
            fn abs_assign(&mut self) {
                *self = self.abs();
            }
        }
    };
}
apply_to_signed!(impl_abs_primitive_int);

macro_rules! impl_abs_primitive_float {
    ($f:ident) => {
        const impl Abs for $f {
            type Output = $f;
            #[inline]
            fn abs(self) -> $f {
                $f::abs(self)
            }
        }

        const impl AbsAssign for $f {
            /// Replaces a number with its absolute value.
            ///
            /// $x \gets |x|$.
            ///
            /// # Worst-case complexity
            /// Constant time and additional memory.
            ///
            /// # Examples
            /// See [here](super::abs#abs_assign).
            #[inline]
            fn abs_assign(&mut self) {
                *self = $f::abs(*self);
            }
        }
    };
}
apply_to_primitive_floats!(impl_abs_primitive_float);

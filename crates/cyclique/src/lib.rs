#![feature(
    const_trait_impl,
    const_ops,
    const_cmp,
    const_clone,
    const_default,
    derive_const
)]
#![cfg_attr(not(test), deny(clippy::unwrap_used))]

//! Congruence classes of the natural numbers.
//!
//! The `new_modulus_type!` macro declares a type whose values are integers
//! modulo a compile-time constant. The smallest unsigned integer type able to
//! represent every residue is used as the internal representation, and the
//! generated arithmetic is reduced modulo that constant.
//!
//! Modulus types support addition, subtraction, multiplication, and negation
//! both with themselves and with every built-in signed and unsigned integer
//! type. The right-hand integer is interpreted as an integer and reduced
//! modulo the divisor before the operation, so `1 - 2` on a modulus type with
//! divisor `10` is the class of `9` and `3 * -1` is the class of `7`.
//!
//! # Optional serialization support
//!
//! The `serde` feature implements `serde::Serialize` and `serde::Deserialize`
//! for every type declared by [`new_modulus_type!`]. The `rkyv` feature
//! implements `rkyv::Archive`, `rkyv::Serialize` and `rkyv::Deserialize` for
//! the same types.
//!
//! In both cases the value is stored as its underlying representation integer,
//! including for enum declarations, and deserialization rejects values outside
//! the modulus. A modulus type with a divisor of one is zero-sized and
//! serializes as the integer zero. Because pattern types do not implement
//! these traits, the implementations are generated manually rather than
//! derived.

use std::{
    fmt,
    ops::{Add, AddAssign, Mul, MulAssign, Neg, Not, Sub, SubAssign},
};

use paste::paste;

pub use cyclique_macro::{Debug, new_modulus_type};

#[doc(hidden)]
pub mod __private {
    pub use core;

    /// Re-export used by the generated code when the `serde` feature is enabled.
    #[cfg(feature = "serde")]
    pub use serde;

    /// Re-export used by the generated code when the `rkyv` feature is enabled.
    #[cfg(feature = "rkyv")]
    pub use rkyv;

    /// Error returned when a serialized value is not a valid residue of its
    /// modulus type.
    #[cfg(feature = "rkyv")]
    #[derive(Debug)]
    pub struct ResidueOutOfRange {
        type_name: &'static str,
    }

    #[cfg(feature = "rkyv")]
    impl ResidueOutOfRange {
        /// Creates a new error for the named modulus type.
        #[inline(always)]
        pub const fn new(type_name: &'static str) -> Self {
            Self { type_name }
        }
    }

    #[cfg(feature = "rkyv")]
    impl std::fmt::Display for ResidueOutOfRange {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(
                f,
                "value out of range for modulus type `{}`",
                self.type_name
            )
        }
    }

    #[cfg(feature = "rkyv")]
    impl std::error::Error for ResidueOutOfRange {}

    /// Multiply two 128-bit integers into a full 256-bit product.
    ///
    /// The returned pair is `(low, high)`.
    #[inline(always)]
    pub const fn wide_mul(a: u128, b: u128) -> (u128, u128) {
        let a_low = a as u64 as u128;
        let a_high = a >> 64;
        let b_low = b as u64 as u128;
        let b_high = b >> 64;

        let low_low = a_low * b_low;
        let low_high = a_low * b_high;
        let high_low = a_high * b_low;
        let high_high = a_high * b_high;

        let middle = (low_low >> 64) + (low_high as u64 as u128) + (high_low as u64 as u128);
        let low = ((middle as u64 as u128) << 64) | (low_low as u64 as u128);
        let high = high_high + (low_high >> 64) + (high_low >> 64) + (middle >> 64);
        (low, high)
    }

    /// Montgomery reduction of the 256-bit value `high * 2^128 + low` modulo
    /// the odd modulus `n`.
    ///
    /// `nprime` must satisfy `nprime = -n^-1 (mod 2^128)`.
    #[inline(always)]
    pub const fn redc(low: u128, high: u128, n: u128, nprime: u128) -> u128 {
        let factor = low.wrapping_mul(nprime);
        let (factor_low, factor_high) = wide_mul(factor, n);
        let (_, carry_low) = low.overflowing_add(factor_low);
        let (sum, carry_high) = high.overflowing_add(factor_high);
        let (sum, carry_extra) = sum.overflowing_add(carry_low as u128);
        let carry = carry_high | carry_extra;
        let reduced = sum.wrapping_sub(n);
        if carry || sum >= n { reduced } else { sum }
    }

    /// Multiply `a` and `b` and Montgomery-reduce the product modulo the odd
    /// modulus `n`.
    #[inline(always)]
    pub const fn redc_mul(a: u128, b: u128, n: u128, nprime: u128) -> u128 {
        let (low, high) = wide_mul(a, b);
        redc(low, high, n, nprime)
    }
}

pub trait ModulusType {
    /// The internal representation type.
    type Repr;
    /// The divisor of the modulus type.
    ///
    /// Zero encodes `2^BITS`, where `BITS` is the width of [`Self::Repr`]. In
    /// that case every value of the representation is a valid residue and the
    /// arithmetic wraps around.
    const DIVISOR: Self::Repr;
}

#[allow(non_camel_case_types)]
#[derive(Copy, Hash)]
#[derive_const(Clone, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct u0;

impl u0 {
    pub const MAX: Self = u0;
    pub const MIN: Self = u0;
    pub const BITS: u32 = 0;
}

impl fmt::Debug for u0 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fmt::Debug::fmt(&0u8, f)
    }
}

impl fmt::Display for u0 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fmt::Display::fmt(&0u8, f)
    }
}

macro_rules! impl_bin_op {
    () => {};
    ($($name:ident),+$(,)?) => {
        $(paste! {
            const impl [< $name:camel >] for u0 {
                type Output = u0;
                fn [< $name:snake >](self, _rhs: Self) -> Self::Output {
                    u0
                }
            }

            const impl [< $name:camel Assign >] for u0 {
                fn [< $name:snake _assign >](&mut self, _rhs: Self) {} // does nothing
            }
        })+
    };
}

macro_rules! impl_un_op {
    () => {};
    ($($name:ident),+$(,)?) => {
        $(paste! {
            const impl [< $name:camel >] for u0 {
                type Output = u0;
                fn [< $name:snake >](self) -> Self::Output {
                    u0
                }
            }
        })+
    };
}

impl_bin_op!(Add, Sub, Mul);
impl_un_op!(Neg, Not);

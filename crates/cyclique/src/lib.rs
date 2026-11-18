#![feature(
    const_trait_impl,
    const_ops,
    const_cmp,
    const_clone,
    const_default,
    derive_const
)]

use std::{
    fmt,
    ops::{Add, AddAssign, Mul, MulAssign, Neg, Not, Sub, SubAssign},
};

use paste::paste;

pub use cyclique_macro::{Debug, new_modulus_type};

#[doc(hidden)]
pub mod __private {
    pub use core;
}

pub trait ModulusType {
    /// The internal representation type
    type Repr;
    /// The divisor of the modulus type. None means 1 plus the maximum of representation type.
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

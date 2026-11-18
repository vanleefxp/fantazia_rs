//! Musical **accidental** representation.

use std::{
    fmt::{Display, Write},
    num::ParseIntError,
    ops::{Add, AddAssign, Mul, MulAssign, Sub, SubAssign},
    str::FromStr,
};

use derive_more::{Add, AddAssign, Debug, Deref, DerefMut, From, Hash, Into, Neg, Sub, SubAssign};
use fantazia_core::arith::{Abs, AbsAssign, NegAssign, Sign};
#[cfg(feature = "macro")]
use proc_macro2::TokenStream;
#[cfg(feature = "macro")]
use quote::{ToTokens, quote};
use thiserror::Error;

use crate::tone::Tone;

/// Helper function for creating an [`Acci`] object
#[inline(always)]
pub const fn acci(value: impl [const] Into<Acci>) -> Acci {
    value.into()
}

/// A musical **accidental** denoted by the number of semitones it alters the pitch.
#[derive(Debug, Copy, Hash)]
#[derive_const(
    Clone, Default, PartialEq, Eq, PartialOrd, Ord, Add, AddAssign, Sub, SubAssign, Neg, From,
    Into, Deref, DerefMut
)]
#[const_impl]
#[cfg_attr(
    feature = "rkyv",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(transparent)]
pub struct Acci(pub(crate) i8);

impl Acci {
    /// triple sharp
    pub const TRIPLE_SHARP: Self = Self(3);
    /// double sharp
    pub const DOUBLE_SHARP: Self = Self(2);
    /// sharp
    pub const SHARP: Self = Self(1);
    /// natural (no accidental)
    pub const NATURAL: Self = Self(0);
    /// flat
    pub const FLAT: Self = Self(-1);
    /// double flat
    pub const DOUBLE_FLAT: Self = Self(-2);
    /// triple flat
    pub const TRIPLE_FLAT: Self = Self(-3);

    /// Creates a new [`Acci`] object.
    pub const fn new(value: i8) -> Self {
        Self(value)
    }

    /// Represents the [`Acci`] object as primitive type [`i8`].
    pub const fn as_i8(&self) -> i8 {
        self.0
    }
}

const impl From<Acci> for Tone {
    fn from(value: Acci) -> Self {
        Self(value.0)
    }
}

const impl From<Tone> for Acci {
    fn from(value: Tone) -> Self {
        Self(value.0)
    }
}

const impl Add<Acci> for Tone {
    type Output = Tone;

    fn add(self, rhs: Acci) -> Self::Output {
        self + Self(rhs.0)
    }
}

const impl AddAssign<Acci> for Tone {
    fn add_assign(&mut self, rhs: Acci) {
        *self += Self(rhs.0);
    }
}

const impl Sub<Acci> for Tone {
    type Output = Self;

    fn sub(self, rhs: Acci) -> Self::Output {
        self - Self(rhs.0)
    }
}

const impl SubAssign<Acci> for Tone {
    fn sub_assign(&mut self, rhs: Acci) {
        *self -= Self(rhs.0);
    }
}

const impl Mul<i8> for Acci {
    type Output = Self;

    fn mul(mut self, rhs: i8) -> Self::Output {
        self *= rhs;
        self
    }
}

const impl Mul<Acci> for i8 {
    type Output = Acci;
    fn mul(self, rhs: Acci) -> Self::Output {
        rhs * self
    }
}

const impl MulAssign<i8> for Acci {
    fn mul_assign(&mut self, rhs: i8) {
        self.0 *= rhs;
    }
}

const impl NegAssign for Acci {
    fn neg_assign(&mut self) {
        *self = -*self;
    }
}

const impl Abs for Acci {
    type Output = Self;

    fn abs(self) -> Self::Output {
        Self(self.0.abs())
    }
}

const impl AbsAssign for Acci {
    fn abs_assign(&mut self) {
        *self = self.abs();
    }
}

const impl Sign for Acci {
    fn sign(&self) -> std::cmp::Ordering {
        self.0.cmp(&0)
    }
}

impl Display for Acci {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            v @ ..-3 => write!(f, "[{v}]")?,
            v @ -3..=-1 => {
                for _ in 0..(-v) {
                    f.write_char('-')?
                }
            }
            0 => f.write_char('=')?,
            v @ 1..=3 => {
                for _ in 0..v {
                    f.write_char('+')?
                }
            }
            v => write!(f, "[{v:+}]")?,
        }
        Ok(())
    }
}

/// Error type returned when parsing an [`Acci`].
#[derive(Debug, Clone, Error)]
pub enum InvalidAcci {
    /// Invalid accidental number.
    #[error("invalid accidental number - {0}")]
    InvalidNumber(#[from] ParseIntError),
    /// Invalid character in accidental string.
    #[error(
        "`{1}` (at index {0}) is not a valid character for accidental (expect `'+'`, `'-'` or `'='`)"
    )]
    InvalidChar(usize, char),
}

impl FromStr for Acci {
    type Err = InvalidAcci;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.starts_with('[') && s.ends_with(']') {
            let s = &s[1..(s.len() - 1)];
            Ok(Self(s.parse()?))
        } else {
            let mut value = 0i8;
            for (idx, ch) in s.char_indices() {
                match ch {
                    '+' => value += 1,
                    '-' => value -= 1,
                    '=' => (),
                    ch => return Err(InvalidAcci::InvalidChar(idx, ch)),
                }
            }
            Ok(Self(value))
        }
    }
}

#[cfg(feature = "macro")]
impl ToTokens for Acci {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let value = self.0;
        tokens.extend(quote! {
            ::fantazia::edo12::acci::Acci::new(#value)
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_display_and_parse() {
        let expected = [
            (acci(4), "[+4]"),
            (acci(3), "+++"),
            (acci(2), "++"),
            (acci(1), "+"),
            (acci(0), "="),
            (acci(-1), "-"),
            (acci(-2), "--"),
            (acci(-3), "---"),
            (acci(-4), "[-4]"),
        ];
        for (input, expected) in expected {
            assert_eq!(input.to_string(), expected);
            assert_eq!(expected.parse::<Acci>().unwrap(), input);
        }
    }
}

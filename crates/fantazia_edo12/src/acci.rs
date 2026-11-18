//! Musical **accidentals**.
//!
//! An accidental is the number of semitones by which a pitch is altered relative to the diatonic
//! tone of its step: `+1` is a sharp, `-1` a flat, `0` is natural, and values beyond that denote
//! double, triple, ... accidentals. [`Acci`] is a transparent wrapper around [`i8`], so every
//! integer is a valid accidental.
//!
//! [`Acci`] renders as a sequence of `+` / `-` / `=` characters and can be parsed back from the
//! same notation, including the bracketed form used for wide accidentals such as `[+4]`.
//!
//! The `acci!` literal macro (provided by `fantazia_edo12_macro` and re-exported by the `fantazia`
//! facade) constructs an accidental from a literal such as `acci!(++)`.

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

/// Formats the accidental with one `+` or `-` per semitone, `=` for a natural, or as a bracketed
/// signed integer (e.g. `[+4]`) when it would require more than three signs.
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

    /// Parses an accidental from a sequence of `+`, `-` and `=` characters, or from a bracketed
    /// decimal integer such as `[+4]`.
    ///
    /// Every `+` raises the accidental by one semitone and every `-` lowers it by one, while `=`
    /// stands for a natural and leaves the value unchanged. The empty string parses as the natural
    /// accidental.
    ///
    /// ```
    /// use fantazia_edo12::acci::Acci;
    ///
    /// assert_eq!("+".parse::<Acci>().ok(), Some(Acci::SHARP));
    /// assert_eq!("--".parse::<Acci>().ok(), Some(Acci::DOUBLE_FLAT));
    /// assert_eq!("=".parse::<Acci>().ok(), Some(Acci::NATURAL));
    /// assert_eq!("[+4]".parse::<Acci>().ok(), Some(Acci::new(4)));
    /// assert_eq!("x".parse::<Acci>().ok(), None);
    /// ```
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
    use std::cmp::Ordering;

    use fantazia_core::arith::{Abs, NegAssign, Sign};

    use super::*;
    use crate::tone::Tone;

    #[test]
    fn test_constants() {
        assert_eq!(Acci::TRIPLE_SHARP.as_i8(), 3);
        assert_eq!(Acci::DOUBLE_SHARP.as_i8(), 2);
        assert_eq!(Acci::SHARP.as_i8(), 1);
        assert_eq!(Acci::NATURAL.as_i8(), 0);
        assert_eq!(Acci::FLAT.as_i8(), -1);
        assert_eq!(Acci::DOUBLE_FLAT.as_i8(), -2);
        assert_eq!(Acci::TRIPLE_FLAT.as_i8(), -3);

        assert_eq!(Acci::new(2), Acci::DOUBLE_SHARP);
        assert_eq!(Acci::new(2), Acci::DOUBLE_SHARP);
        assert_eq!(Acci::default(), Acci::NATURAL);
        // `Acci` dereferences to its `i8` representation.
        assert_eq!(*Acci::FLAT, -1);
    }

    #[test]
    fn test_display_and_parse() {
        let expected = [
            (Acci::new(4), "[+4]"),
            (Acci::new(3), "+++"),
            (Acci::new(2), "++"),
            (Acci::new(1), "+"),
            (Acci::new(0), "="),
            (Acci::new(-1), "-"),
            (Acci::new(-2), "--"),
            (Acci::new(-3), "---"),
            (Acci::new(-4), "[-4]"),
        ];
        for (input, expected) in expected {
            assert_eq!(input.to_string(), expected);
            assert_eq!(expected.parse::<Acci>().unwrap(), input);
        }

        // Every accidental round-trips through its display representation.
        for value in -10..=10 {
            let acci = Acci::new(value);
            let src = acci.to_string();
            assert_eq!(src.parse::<Acci>().unwrap(), acci, "parse `{src}`");
        }
        assert_eq!(Acci::new(4).to_string(), "[+4]");
        assert_eq!(Acci::new(-6).to_string(), "[-6]");
    }

    #[test]
    fn test_parse_forms() {
        assert_eq!("".parse::<Acci>().unwrap(), Acci::NATURAL);
        assert_eq!("=".parse::<Acci>().unwrap(), Acci::NATURAL);
        assert_eq!("+-".parse::<Acci>().unwrap(), Acci::NATURAL);
        assert_eq!("=+=".parse::<Acci>().unwrap(), Acci::SHARP);
        assert_eq!("[4]".parse::<Acci>().unwrap(), Acci::new(4));
        assert_eq!("[+4]".parse::<Acci>().unwrap(), Acci::new(4));
        assert_eq!("[-4]".parse::<Acci>().unwrap(), Acci::new(-4));

        assert!(matches!(
            "C".parse::<Acci>(),
            Err(InvalidAcci::InvalidChar(0, 'C'))
        ));
        assert!(matches!(
            "+x+".parse::<Acci>(),
            Err(InvalidAcci::InvalidChar(1, 'x'))
        ));
        assert!(matches!(
            "[x]".parse::<Acci>(),
            Err(InvalidAcci::InvalidNumber(_))
        ));
        assert!(matches!(
            "[999]".parse::<Acci>(),
            Err(InvalidAcci::InvalidNumber(_))
        ));
        assert_eq!(
            "x".parse::<Acci>().unwrap_err().to_string(),
            "`x` (at index 0) is not a valid character for accidental (expect `'+'`, `'-'` or `'='`)"
        );
    }

    #[test]
    fn test_arithmetic() {
        assert_eq!(Acci::SHARP + Acci::FLAT, Acci::NATURAL);
        assert_eq!(Acci::SHARP - Acci::SHARP, Acci::NATURAL);
        assert_eq!(Acci::DOUBLE_SHARP * 2, Acci::new(4));
        assert_eq!(2 * Acci::DOUBLE_SHARP, Acci::new(4));
        assert_eq!(-Acci::SHARP, Acci::FLAT);
        assert_eq!(-Acci::NATURAL, Acci::NATURAL);
        assert_eq!(Acci::new(-3).abs(), Acci::TRIPLE_SHARP);

        assert_eq!(Acci::FLAT.sign(), Ordering::Less);
        assert_eq!(Acci::NATURAL.sign(), Ordering::Equal);
        assert_eq!(Acci::SHARP.sign(), Ordering::Greater);

        let mut acci = Acci::SHARP;
        acci += Acci::SHARP;
        assert_eq!(acci, Acci::DOUBLE_SHARP);
        acci -= Acci::SHARP;
        assert_eq!(acci, Acci::SHARP);
        acci *= 3;
        assert_eq!(acci, Acci::TRIPLE_SHARP);
        acci.neg_assign();
        assert_eq!(acci, Acci::TRIPLE_FLAT);
        acci.abs_assign();
        assert_eq!(acci, Acci::TRIPLE_SHARP);
    }

    #[test]
    fn test_tone_interop() {
        assert_eq!(Tone::from(Acci::SHARP), Tone::new(1));
        assert_eq!(Acci::from(Tone::new(-1)), Acci::FLAT);

        assert_eq!(Tone::C_0 + Acci::SHARP, Tone::new(1));
        assert_eq!(Tone::new(1) - Acci::SHARP, Tone::C_0);

        let mut tone = Tone::C_0;
        tone += Acci::DOUBLE_SHARP;
        assert_eq!(tone, Tone::new(2));
        tone -= Acci::SHARP;
        assert_eq!(tone, Tone::new(1));
    }

    #[test]
    fn test_debug() {
        assert_eq!(format!("{:?}", Acci::SHARP), "Acci(1)");
        assert_eq!(format!("{:?}", Acci::TRIPLE_FLAT), "Acci(-3)");
    }
}

use std::{
    fmt::{Debug, Display},
    iter::Sum,
    mem,
    ops::{Add, AddAssign, Deref, Mul, MulAssign, Neg, Sub, SubAssign},
};

use derive_more::{
    Add, AddAssign, Deref, DerefMut, Display, From, Hash, Into, Mul, MulAssign, Neg, Sub, SubAssign,
};
#[cfg(feature = "macro")]
use quote::{ToTokens, format_ident, quote};

use fantazia_core::{
    arith::{Abs, AbsAssign, NegAssign},
    pitch::{Frequency, GeneralPitch, SpecificPitch, SpecificPitchIndeterministic, ToFrequency},
};
use fantazia_internal_macro::{gen_otone_consts, gen_tone_consts};

use crate::acci::Acci;

/// Helper function for creating a [`Tone`].
/// See [`Tone`] for details.
///
/// # Examples
///
/// ```
/// use fantazia_edo12::tone::{tone, Tone};
///
/// assert_eq!(Tone::F_SHARP_0, tone(6));
/// assert_eq!(Tone::E_1, tone(16));
/// assert_eq!(Tone::B_FLAT_M1, tone(-2));
/// ```
#[inline(always)]
pub const fn tone(value: impl [const] Into<Tone>) -> Tone {
    value.into()
}

/// Helper function for creating an [`OTone`] that never panics.
/// It first converts the input into a [`Tone`] and then puts it in octave.
///
/// # Examples
///
/// ```
/// use fantazia_edo12::tone::{otone, OTone};
///
/// assert_eq!(OTone::F_SHARP, otone(6));
/// assert_eq!(OTone::E, otone(16));
/// assert_eq!(OTone::B_FLAT, otone(-2));
/// ```
#[inline(always)]
pub const fn otone(value: impl [const] Into<Tone>) -> OTone {
    tone(value).generalize()
}

// MARK: Octave Chromatic Tone

/// A chromatic tone within an octave notated by the distance from the C key in number of semitones.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "rkyv",
    derive(rkyv::Serialize, rkyv::Deserialize, rkyv::Archive)
)]
#[derive(Copy, Hash)]
#[derive_const(Clone, Default, PartialEq, Eq, PartialOrd, Ord)]
pub enum OTone {
    #[default]
    #[doc(hidden)]
    _0,
    #[doc(hidden)]
    _1,
    #[doc(hidden)]
    _2,
    #[doc(hidden)]
    _3,
    #[doc(hidden)]
    _4,
    #[doc(hidden)]
    _5,
    #[doc(hidden)]
    _6,
    #[doc(hidden)]
    _7,
    #[doc(hidden)]
    _8,
    #[doc(hidden)]
    _9,
    #[doc(hidden)]
    _10,
    #[doc(hidden)]
    _11,
}

const impl Deref for OTone {
    type Target = i8;
    fn deref(&self) -> &Self::Target {
        unsafe { mem::transmute(self) }
    }
}

impl Debug for OTone {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("OTone").field(&**self).finish()
    }
}

impl Display for OTone {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", **self)
    }
}

gen_otone_consts!();

impl OTone {
    /// Creates a new [`OTone`] object.
    pub const fn new(value: i8) -> Option<Self> {
        value.try_into().ok()
    }

    /// Creates an [`OTone`] object directly from integer representation without checking whether the
    /// value falls in the range `0..12`.
    ///
    /// # Safety
    /// Caller must ensure that `value` falls in the range `0..12`. Otherwise it is undefined behavior.
    #[inline(always)]
    pub const unsafe fn new_unchecked(value: i8) -> Self {
        debug_assert!(
            matches!(value, 0..12),
            "`value` does not fall in `0..12`, will cause undefined behavior"
        );
        unsafe { mem::transmute(value) }
    }
}

const impl TryFrom<i8> for OTone {
    type Error = ();

    fn try_from(value: i8) -> Result<Self, Self::Error> {
        match value {
            0..12 => unsafe { Ok(Self::new_unchecked(value)) },
            _ => Err(()),
        }
    }
}

const impl From<OTone> for i8 {
    fn from(value: OTone) -> Self {
        value as i8
    }
}

const impl GeneralPitch<Tone> for OTone {
    fn put_in_octave(self, octave: i8) -> Tone {
        Tone(octave * 12 + self as i8)
    }
}

impl ToFrequency for OTone {
    fn to_frequency(&self) -> Frequency {
        let tone = *self as i8;
        // SAFETY: `tone` is in range `0..12` so this will not result in infinity, negative or NaN values
        unsafe { Frequency::new_unchecked((tone as f64 / 12.).exp2()) }
    }
}

const impl Add<OTone> for OTone {
    type Output = Self;

    fn add(self, rhs: OTone) -> Self::Output {
        unsafe { Self::new_unchecked((self as i8 + rhs as i8).rem_euclid(12)) }
    }
}

const impl AddAssign<OTone> for OTone {
    fn add_assign(&mut self, rhs: OTone) {
        *self = *self + rhs;
    }
}

const impl Sub<OTone> for OTone {
    type Output = Self;

    fn sub(self, rhs: OTone) -> Self::Output {
        unsafe { Self::new_unchecked((self as i8 - rhs as i8).rem_euclid(12)) }
    }
}

const impl SubAssign<OTone> for OTone {
    fn sub_assign(&mut self, rhs: OTone) {
        *self = *self - rhs;
    }
}

const impl Mul<i8> for OTone {
    type Output = Self;
    fn mul(self, rhs: i8) -> Self::Output {
        unsafe { Self::new_unchecked((self as i16 * rhs as i16).rem_euclid(12) as i8) }
    }
}

const impl MulAssign<i8> for OTone {
    fn mul_assign(&mut self, rhs: i8) {
        *self = *self * rhs;
    }
}

const impl Neg for OTone {
    type Output = Self;
    fn neg(self) -> Self::Output {
        unsafe {
            Self::new_unchecked(match self as i8 {
                0 => 0,
                n => 12 - n,
            })
        }
    }
}

const impl NegAssign for OTone {
    fn neg_assign(&mut self) {
        *self = -*self;
    }
}

// MARK: OPitch Tone

/// The tone value for an [`OPitch`] object.
///
/// Note that different from [`OTone`], the internal value of `OPitchTone` may exceed the range 0..12.
/// For example [`OPitch`] object *C flat* has tone -1 and *B sharp* has tone 12.
#[derive(Debug, Display, Copy, Hash)]
#[derive_const(
    Clone, Default, PartialEq, Eq, PartialOrd, Ord, Deref, DerefMut, From, Into
)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "rkyv",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
#[const_impl]
pub struct OPitchTone(pub(crate) i8);

impl OPitchTone {
    /// Creates a new [`OPitchTone`] object.
    #[inline(always)]
    pub const fn new(value: impl [const] Into<Self>) -> Self {
        value.into()
    }

    /// Represents the [`OPitchTone`] object as primitive type [`i8`].
    #[inline(always)]
    pub const fn as_i8(&self) -> i8 {
        self.0
    }
}

impl ToFrequency for OPitchTone {
    fn to_frequency(&self) -> Frequency {
        unsafe { Frequency::new_unchecked((self.0 as f64 / 12.0).exp2()) }
    }
}

const impl SpecificPitch<OTone> for OPitchTone {
    fn octave(&self) -> i8 {
        self.0.div_euclid(12)
    }
    fn generalize(self) -> OTone {
        unsafe { OTone::new_unchecked(self.0.rem_euclid(12)) }
    }
}

const impl GeneralPitch<Tone> for OPitchTone {
    fn put_in_octave(self, octave: i8) -> Tone {
        Tone(self.0 + octave * 12)
    }
}

const impl Add<Acci> for OPitchTone {
    type Output = Self;
    fn add(self, rhs: Acci) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

const impl AddAssign<Acci> for OPitchTone {
    fn add_assign(&mut self, rhs: Acci) {
        self.0 += rhs.0
    }
}

const impl Sub<Acci> for OPitchTone {
    type Output = Self;
    fn sub(self, rhs: Acci) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}

const impl SubAssign<Acci> for OPitchTone {
    fn sub_assign(&mut self, rhs: Acci) {
        self.0 -= rhs.0
    }
}

// MARK: Chromatic Tone

/// A chromatic tone notated by the distance from middle C in number of semitones.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "rkyv",
    derive(rkyv::Serialize, rkyv::Deserialize, rkyv::Archive)
)]
#[derive(Debug, Display, Copy, Hash)]
#[derive_const(
    Clone, Default, PartialEq, Eq, PartialOrd, Ord, Add, AddAssign, Sub, SubAssign, Mul, MulAssign,
    Neg, Deref, DerefMut, From, Into
)]
#[const_impl]
pub struct Tone(pub(crate) i8);

gen_tone_consts!();

impl Tone {
    /// Creates a new [`Tone`] object.
    #[inline(always)]
    pub const fn new(value: impl [const] Into<Self>) -> Self {
        value.into()
    }

    /// Reoresents the [`Tone`] object as primitive type [`i8`].
    #[inline(always)]
    pub const fn as_i8(&self) -> i8 {
        self.0
    }

    // pub const fn checked_add(&self, rhs: Self) -> Option<Self> {
    //     self.0.checked_add(rhs.0).map(Self)
    // }

    // pub const fn checked_sub(&self, rhs: Self) -> Option<Self> {
    //     self.0.checked_sub(rhs.0).map(Self)
    // }

    // pub const fn checked_mul(&self, rhs: i8) -> Option<Self> {
    //     self.0.checked_mul(rhs).map(Self)
    // }
}

const impl SpecificPitch<OTone> for Tone {
    fn octave(&self) -> i8 {
        self.0.div_euclid(12)
    }
    fn generalize(self) -> OTone {
        unsafe { OTone::new_unchecked(self.0.rem_euclid(12)) }
    }
}

const impl SpecificPitchIndeterministic<OPitchTone> for Tone {
    fn generalize_with_octave(self, octave: i8) -> OPitchTone {
        OPitchTone(self.0 - 12 * octave)
    }
}

impl ToFrequency for Tone {
    fn to_frequency(&self) -> Frequency {
        let tone = self.0;
        // SAFETY: `tone` is `i8` so this will not result in infinity, negative or NaN values
        unsafe { Frequency::new_unchecked((tone as f64 / 12.).exp2()) }
    }
}

impl Sum for Tone {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.map(|v| v.as_i8()).sum::<i8>().into()
    }
}

const impl NegAssign for Tone {
    fn neg_assign(&mut self) {
        *self = -*self;
    }
}

const impl Abs for Tone {
    type Output = Self;
    fn abs(self) -> Self::Output {
        Self(self.0.abs())
    }
}

const impl AbsAssign for Tone {
    fn abs_assign(&mut self) {
        *self = self.abs()
    }
}

// MARK: Tests

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chromatic_tone() {
        let middle_c = tone(0);
        let high_a = tone(17);
        println!("{:?}", middle_c);
        println!("{:?}", high_a);

        assert_eq!(middle_c.generalize(), OTone::try_from(0).unwrap());
        assert_eq!(high_a.generalize(), OTone::try_from(5).unwrap());
    }

    #[test]
    fn test_const_gen() {
        // Test whether proc-macros are generating the correct constants
        let _otone_consts = [
            OTone::C,
            OTone::C_SHARP,
            OTone::D_FLAT,
            OTone::D,
            OTone::D_SHARP,
            OTone::E_FLAT,
            OTone::F,
            OTone::F_SHARP,
            OTone::G_FLAT,
            OTone::G,
            OTone::G_SHARP,
            OTone::A_FLAT,
            OTone::A,
            OTone::A_SHARP,
            OTone::B_FLAT,
            OTone::B,
        ];
        let _tone_consts = [
            Tone::C_M1,
            Tone::C_0,
            Tone::C_1,
            Tone::C_SHARP_M1,
            Tone::C_SHARP_0,
            Tone::C_SHARP_1,
        ];
    }

    #[test]
    fn test_derived_traits() {
        // no compiler error -> it works

        // Clone
        let _ = OTone::C.clone();
        // const Clone
        let _ = const { OTone::C.clone() };

        // Default
        let _ = OTone::default();
        // const Default
        let _ = const { OTone::default() };

        // PartialEq
        let _ = OTone::C == OTone::C;
        // const PartialEq
        let _ = const { OTone::C == OTone::C };

        // PartialOrd and Ord
        let _ = OTone::C.partial_cmp(&OTone::C);
        let _ = OTone::C.cmp(&OTone::C);

        // const PartialOrd and const Ord
        let _ = const { OTone::C.partial_cmp(&OTone::C) };
        let _ = const { OTone::C.cmp(&OTone::C) };

        let _ = const { OTone::C + OTone::D };
    }
}

#[cfg(feature = "macro")]
impl ToTokens for OTone {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let ident = format_ident!("_{}", (*self as i8).to_string());
        tokens.extend(quote! {
            ::fantazia::edo12::tone::OTone::#ident
        });
    }
}

#[cfg(feature = "macro")]
impl ToTokens for OPitchTone {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let value = self.0;
        tokens.extend(quote! {
            ::fantazia::edo12::tone::OPitchTone::new(#value)
        });
    }
}

#[cfg(feature = "macro")]
impl ToTokens for Tone {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let value = self.as_i8();
        tokens.extend(quote! {
            ::fantazia::edo12::tone::Tone::new(#value)
        });
    }
}

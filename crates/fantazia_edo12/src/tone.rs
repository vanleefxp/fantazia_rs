//! Chromatic **tones**, i.e. pitches measured in semitones above middle C.
//!
//! A tone is the *sounding* component of a named pitch: it does not carry the note letter, so it
//! cannot distinguish enharmonic spellings. This module provides:
//!
//! - [`OTone`]: an octave-unspecific tone, i.e. one of the twelve pitch classes;
//! - [`Tone`]: a tone with an octave specification;
//! - [`OPitchTone`]: the tone component of a named pitch (`OPitch`). Accidentals can move it
//!   outside `0..12`, so it is not restricted to the twelve pitch classes.
//!
//! [`tone`] and [`otone`] are shorthand constructors for [`Tone`] and [`OTone`].

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

/// The tone component of a named pitch (`OPitch`).
///
/// Note that different from [`OTone`], the internal value of `OPitchTone` may exceed the range 0..12.
/// For example the named pitch *C flat* has tone -1 and *B sharp* has tone 12.
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

    /// Represents the [`Tone`] object as primitive type [`i8`].
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

#[cfg(test)]
mod tests {
    use std::cmp::Ordering;

    use fantazia_core::{
        arith::{Abs, NegAssign},
        pitch::{GeneralPitch, SpecificPitch, SpecificPitchIndeterministic, ToFrequency},
    };

    use super::*;

    /// Frequency ratio of `semitones` semitones, i.e. the expected result of `ToFrequency` for a
    /// tone measured in semitones.
    fn ratio(semitones: f64) -> f64 {
        (semitones / 12.0).exp2()
    }

    fn assert_frequency(actual: Frequency, expected: f64) {
        assert!(
            (actual.get() - expected).abs() < 1e-12,
            "expected {expected}, got {}",
            actual.get()
        );
    }

    #[test]
    fn test_const_gen() {
        // Octave-unspecific tones generated from the pitch names.
        let expected = [
            (OTone::C, 0),
            (OTone::C_SHARP, 1),
            (OTone::D_FLAT, 1),
            (OTone::D, 2),
            (OTone::D_SHARP, 3),
            (OTone::E_FLAT, 3),
            (OTone::E, 4),
            (OTone::F, 5),
            (OTone::F_SHARP, 6),
            (OTone::G_FLAT, 6),
            (OTone::G, 7),
            (OTone::G_SHARP, 8),
            (OTone::A_FLAT, 8),
            (OTone::A, 9),
            (OTone::A_SHARP, 10),
            (OTone::B_FLAT, 10),
            (OTone::B, 11),
        ];
        for (tone, semitones) in expected {
            assert_eq!(i8::from(tone), semitones, "{tone:?}");
        }

        // Octave-specific tones.
        assert_eq!(Tone::C_M2.as_i8(), -24);
        assert_eq!(Tone::C_M1.as_i8(), -12);
        assert_eq!(Tone::C_0.as_i8(), 0);
        assert_eq!(Tone::C_1.as_i8(), 12);
        assert_eq!(Tone::C_2.as_i8(), 24);
        assert_eq!(Tone::C_SHARP_1.as_i8(), 13);
        assert_eq!(Tone::B_M1.as_i8(), -1);
        assert_eq!(Tone::B_FLAT_0.as_i8(), 10);
    }

    #[test]
    fn test_otone_integer_representation() {
        for value in 0i8..12 {
            let tone = OTone::new(value).unwrap();
            assert_eq!(i8::from(tone), value);
            assert_eq!(*tone, value);
            assert_eq!(OTone::try_from(value), Ok(tone));
            // SAFETY: `value` is in `0..12`.
            assert_eq!(unsafe { OTone::new_unchecked(value) }, tone);
        }
        assert_eq!(OTone::new(12), None);
        assert_eq!(OTone::new(-1), None);
        assert_eq!(OTone::try_from(12), Err(()));
        assert_eq!(OTone::try_from(-1), Err(()));
        assert_eq!(OTone::default(), OTone::C);
    }

    #[test]
    fn test_otone_arithmetic() {
        // Addition wraps around modulo twelve.
        assert_eq!(OTone::C + OTone::C_SHARP, OTone::C_SHARP);
        assert_eq!(OTone::B + OTone::C_SHARP, OTone::C);
        assert_eq!(OTone::A + OTone::E, OTone::C_SHARP);
        assert_eq!(OTone::C - OTone::C_SHARP, OTone::B);
        assert_eq!(OTone::C_SHARP - OTone::C_SHARP, OTone::C);

        assert_eq!(OTone::C_SHARP * 12, OTone::C);
        assert_eq!(OTone::C_SHARP * -1, OTone::B);
        assert_eq!(OTone::E * 3, OTone::C);

        assert_eq!(-OTone::C, OTone::C);
        assert_eq!(-OTone::C_SHARP, OTone::B);
        assert_eq!(-OTone::G, OTone::F);
        for value in 0..12 {
            let tone = OTone::new(value).unwrap();
            assert_eq!(tone + -tone, OTone::C);
        }
    }

    #[test]
    fn test_otone_assign_arithmetic() {
        let mut tone = OTone::C;
        tone += OTone::B;
        assert_eq!(tone, OTone::B);
        tone += OTone::C_SHARP;
        assert_eq!(tone, OTone::C);
        tone -= OTone::C_SHARP;
        assert_eq!(tone, OTone::B);
        tone *= 2;
        assert_eq!(tone, OTone::A_SHARP);

        let mut tone = OTone::C_SHARP;
        tone.neg_assign();
        assert_eq!(tone, OTone::B);
    }

    #[test]
    fn test_otone_display_and_debug() {
        assert_eq!(OTone::C.to_string(), "0");
        assert_eq!(OTone::C_SHARP.to_string(), "1");
        assert_eq!(OTone::B.to_string(), "11");
        assert_eq!(format!("{:?}", OTone::C), "OTone(0)");
        assert_eq!(format!("{:?}", OTone::A), "OTone(9)");
    }

    #[test]
    fn test_otone_to_frequency() {
        for semitones in 0i8..12 {
            let tone = OTone::new(semitones).unwrap();
            assert_frequency(tone.to_frequency(), ratio(semitones as f64));
        }
        assert_frequency(OTone::C.to_frequency(), 1.0);
        assert_frequency(OTone::A.to_frequency(), ratio(9.0));
    }

    #[test]
    fn test_tone_octave_and_generalization() {
        for value in -30..=30 {
            let tone = Tone::new(value);
            assert_eq!(tone.as_i8(), value);
            assert_eq!(tone.octave(), value.div_euclid(12));
            assert_eq!(i8::from(tone.generalize()), value.rem_euclid(12));
        }

        assert_eq!(Tone::C_0.octave(), 0);
        assert_eq!(Tone::C_1.octave(), 1);
        assert_eq!(Tone::B_M1.octave(), -1);
        assert_eq!(Tone::C_M1.generalize(), OTone::C);
        assert_eq!(Tone::new(13).generalize(), OTone::C_SHARP);
        assert_eq!(Tone::C_1.generalize_with_octave(1), OPitchTone::new(0));
        assert_eq!(Tone::new(13).generalize_with_octave(1), OPitchTone::new(1));
        assert_eq!(Tone::new(13).generalize_with_octave(0), OPitchTone::new(13));
        assert_eq!(OTone::C.put_in_octave(1), Tone::C_1);
        assert_eq!(OTone::C_SHARP.put_in_octave(-1), Tone::C_SHARP_M1);
    }

    #[test]
    fn test_tone_arithmetic() {
        assert_eq!(Tone::C_0 + Tone::C_1, Tone::C_1);
        assert_eq!(Tone::C_1 - Tone::C_0, Tone::C_1);
        assert_eq!(Tone::C_2 - Tone::C_1, Tone::C_1);
        assert_eq!(Tone::C_1 * 2, Tone::C_2);
        assert_eq!(-Tone::C_1, Tone::C_M1);
        assert_eq!(-Tone::C_0, Tone::C_0);
        assert_eq!(Tone::new(-3).abs(), Tone::new(3));
        assert_eq!(Tone::default(), Tone::C_0);

        let mut tone = Tone::C_0;
        tone += Tone::C_1;
        assert_eq!(tone, Tone::C_1);
        tone -= Tone::C_0;
        assert_eq!(tone, Tone::C_1);
        tone *= 2;
        assert_eq!(tone, Tone::C_2);
        tone.neg_assign();
        assert_eq!(tone, Tone::C_M2);
    }

    #[test]
    fn test_tone_sum() {
        assert_eq!([Tone::C_0, Tone::C_1].into_iter().sum::<Tone>(), Tone::C_1);
        assert_eq!(std::iter::empty::<Tone>().sum::<Tone>(), Tone::C_0);
    }

    #[test]
    fn test_tone_to_frequency() {
        assert_frequency(Tone::C_M1.to_frequency(), 0.5);
        assert_frequency(Tone::C_0.to_frequency(), 1.0);
        assert_frequency(Tone::C_1.to_frequency(), 2.0);
        assert_frequency(Tone::C_2.to_frequency(), 4.0);
        assert_frequency(Tone::A_0.to_frequency(), ratio(9.0));
        assert_frequency(Tone::new(-1).to_frequency(), ratio(-1.0));
        // An octave is a factor of two, whatever the starting point is.
        for semitones in -12i8..=12 {
            assert_frequency(
                Tone::new(semitones + 12).to_frequency(),
                Tone::new(semitones).to_frequency().get() * 2.0,
            );
        }
    }

    #[test]
    fn test_tone_display_and_debug() {
        assert_eq!(Tone::C_0.to_string(), "0");
        assert_eq!(Tone::C_1.to_string(), "12");
        assert_eq!(Tone::new(-1).to_string(), "-1");
        assert_eq!(format!("{:?}", Tone::C_1), "Tone(12)");
    }

    #[test]
    fn test_opitch_tone() {
        let tone = OPitchTone::new(13);
        assert_eq!(tone.as_i8(), 13);
        assert_eq!(tone.octave(), 1);
        assert_eq!(tone.generalize(), OTone::C_SHARP);
        assert_eq!(OPitchTone::new(-1).octave(), -1);
        assert_eq!(OPitchTone::new(-1).generalize(), OTone::B);
        assert_eq!(OPitchTone::new(0), OPitchTone::default());

        assert_eq!(tone + Acci::FLAT, OPitchTone::new(12));
        assert_eq!(tone - Acci::DOUBLE_SHARP, OPitchTone::new(11));
        assert_eq!(OPitchTone::new(1).put_in_octave(1), Tone::C_SHARP_1);
        assert_eq!(OPitchTone::new(1).put_in_octave(0), Tone::C_SHARP_0);

        let mut tone = OPitchTone::new(1);
        tone += Acci::SHARP;
        assert_eq!(tone, OPitchTone::new(2));
        tone -= Acci::TRIPLE_FLAT;
        assert_eq!(tone, OPitchTone::new(5));

        assert_frequency(OPitchTone::new(12).to_frequency(), 2.0);
        assert_frequency(OPitchTone::new(-1).to_frequency(), ratio(-1.0));
        assert_eq!(OPitchTone::new(13).to_string(), "13");
    }

    /// Generic helper so that the derived `Clone` implementation can be exercised without
    /// triggering `clippy::clone_on_copy`.
    const fn clone_const<T: [const] Clone>(value: &T) -> T {
        value.clone()
    }

    #[test]
    fn test_derived_traits() {
        // The derived `Clone` implementation is usable at runtime and in `const` contexts.
        assert_eq!(clone_const(&OTone::C), OTone::C);
        assert_eq!(const { clone_const(&OTone::C) }, OTone::C);

        assert_eq!(OTone::default(), OTone::C);
        assert_eq!(const { OTone::default() }, OTone::C);

        assert!(OTone::C == OTone::C);
        const {
            assert!(OTone::C == OTone::C);
        }

        assert_eq!(OTone::C.partial_cmp(&OTone::C), Some(Ordering::Equal));
        assert_eq!(OTone::C.cmp(&OTone::C), Ordering::Equal);
        assert_eq!(const { OTone::C.cmp(&OTone::D) }, Ordering::Less);
        assert_eq!(const { OTone::C + OTone::D }, OTone::D);
    }

    #[test]
    fn test_chromatic_tone() {
        let middle_c = tone(0);
        let high_a = tone(17);

        assert_eq!(middle_c, Tone::C_0);
        assert_eq!(high_a, Tone::F_1);
        assert_eq!(middle_c.generalize(), OTone::C);
        assert_eq!(high_a.generalize(), OTone::F);
        assert_eq!(otone(17), OTone::F);
    }
}

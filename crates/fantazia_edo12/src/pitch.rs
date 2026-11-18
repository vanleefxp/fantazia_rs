//! Named pitch representation. In `fantazia` a named pitch is represented by its **step** and **tone**
//! value. Tone determines how the pitch sounds and step adds information about how the pitch is named.
//! This naming system distinguishes **enharmonics** effectively. For example, _F sharp_ and _G flat_
//! are enharmonics but are named differently.
//!
//! This module provides:
//! - the [`OPitch`] type for octave-unspecific pitch.
//! - the [`Pitch`] type for octave-specific pitch.
//!
//! The `opitch!` and `pitch!` literal macros (provided by `fantazia_edo12_macro` and re-exported by
//! the `fantazia` facade) construct pitches from literals such as `opitch!(F+)` and
//! `pitch!(G[+3]_2)`.

use std::{
    fmt::{self, Debug, Display, Formatter},
    iter::Sum,
    num::ParseIntError,
    ops::{Add, AddAssign, Mul, MulAssign, Neg, Sub, SubAssign},
    str::FromStr,
};

use crate::{
    acci::{Acci, InvalidAcci},
    step::{InvalidOStep, OStep, Step},
    tone::{OPitchTone, Tone},
};
use derive_more::{
    Add, AddAssign, Deref, DerefMut, From, Hash, Into, Mul, MulAssign, Neg, Sub, SubAssign, Sum,
};
use fantazia_core::{
    arith::{Abs, AbsAssign, Sign},
    pitch::{Frequency, GeneralPitch, GenericPitch, SpecificPitch, ToFrequency},
};
use fantazia_internal_macro::{gen_acci_ops_trait, gen_opitch_consts, gen_pitch_consts};
#[cfg(feature = "macro")]
use proc_macro2::TokenStream;
#[cfg(feature = "macro")]
use quote::{ToTokens, quote};
use thiserror::Error;

gen_acci_ops_trait!();

// MARK: OPitch

/// An octave-unspecific named pitch / interval.
#[derive(Copy, Hash)]
#[derive_const(
    Clone, Default, PartialEq, Eq, PartialOrd, Ord, Deref, DerefMut, From, Into
)]
#[const_impl]
#[cfg_attr(
    feature = "rkyv",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct OPitch(pub(crate) GenericPitch<OStep, OPitchTone>);

gen_opitch_consts!();

impl OPitch {
    /// Creates a new [`OPitch`] object from step and tone.
    #[inline(always)]
    pub const fn new(step: impl [const] Into<OStep>, tone: impl [const] Into<OPitchTone>) -> Self {
        Self(GenericPitch {
            step: step.into(),
            tone: tone.into(),
        })
    }

    /// Creates a new [`OPitch`] object from step. Uses the step's diatonic tone.
    ///
    /// See [`OStep::diatonic_tone`] for details.
    #[inline(always)]
    pub const fn diatonic(step: impl [const] Into<OStep>) -> Self {
        let step = step.into();
        Self(GenericPitch {
            step,
            tone: step.diatonic_tone(),
        })
    }

    /// Returns the `order`-th named pitch in **circle / chain of fifth**. The result is calculated by
    /// repeatedly raising up C by perfect fifths `order` times (or down if `order` is negative).
    pub const fn from_co5_order(order: i8) -> Self {
        let step = OStep::from_co5_order(order);
        let tone = OPitchTone(step.diatonic_tone().0 + (order + 1).div_euclid(7));
        Self(GenericPitch { step, tone })
    }

    /// Returns the pitch's index in **circle / chain of fifth**
    pub const fn co5_order(&self) -> i8 {
        self.step.co5_order() + self.acci().0 * 7
    }

    /// Returns the **accidental** of the pitch. i.e. the difference between the pitch's actual tone and
    /// the step's diatonic tone.
    pub const fn acci(&self) -> Acci {
        Acci(self.tone.0 - self.step.diatonic_tone().0)
    }

    /// Changes the pitch's step in place.
    #[inline(always)]
    pub const fn with_step(mut self, step: impl [const] Into<OStep>) -> Self {
        self.step = step.into();
        self
    }

    /// Changes the pitch's tone in place.
    #[inline(always)]
    pub const fn with_tone(mut self, tone: impl [const] Into<OPitchTone>) -> Self {
        self.tone = tone.into();
        self
    }

    /// Sets the pitch's accidental in place.
    ///
    /// **Note**: If you want to alter the pitch by an accidental
    pub const fn with_acci(self, acci: impl [const] Into<Acci>) -> Self {
        self.with_tone(self.step.diatonic_tone().0 + acci.into().0)
    }

    /// Returns `true` if the pitch has no accidental, i.e. if its tone is the diatonic tone of its
    /// step.
    ///
    /// ```
    /// use fantazia_edo12::pitch::OPitch;
    ///
    /// assert!(OPitch::C.is_diatonic());
    /// assert!(OPitch::E.is_diatonic());
    /// assert!(!OPitch::C_SHARP.is_diatonic());
    /// assert!(!OPitch::D_FLAT.is_diatonic());
    /// ```
    pub const fn is_diatonic(&self) -> bool {
        self.tone == self.step.diatonic_tone()
    }

    /// Returns `true` if this pitch and `other` sound the same, i.e. if they are **enharmonic**.
    ///
    /// Enharmonic pitches are spelled differently, so they are not [`PartialEq`] equal.
    ///
    /// ```
    /// use fantazia_edo12::pitch::OPitch;
    ///
    /// assert!(OPitch::C_SHARP.is_enharmonic(&OPitch::D_FLAT));
    /// assert!(!OPitch::C_SHARP.is_enharmonic(&OPitch::D));
    /// assert_ne!(OPitch::C_SHARP, OPitch::D_FLAT);
    /// ```
    pub const fn is_enharmonic(&self, other: &Self) -> bool {
        self.tone.generalize() == other.tone.generalize()
    }
}

const impl GeneralPitch<Pitch> for OPitch {
    fn put_in_octave(self, octave: i8) -> Pitch {
        self.0.put_in_octave(octave).into()
    }
}

impl ToFrequency for OPitch {
    fn to_frequency(&self) -> Frequency {
        self.0.to_frequency()
    }
}

const impl Add for OPitch {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        let mut step = self.step as i8 + rhs.step as i8;
        let mut tone = self.tone.0 + rhs.tone.0;
        if step >= 7 {
            tone -= 12;
            step -= 7;
        }
        let step = unsafe { OStep::new_unchecked(step) };
        let tone = OPitchTone(tone);
        Self(GenericPitch { step, tone })
    }
}

const impl AddAssign for OPitch {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

const impl Add<Step> for OPitch {
    type Output = Self;
    fn add(mut self, rhs: Step) -> Self::Output {
        self += rhs;
        self
    }
}

/// Transposes the pitch up by the diatonic interval from C to `rhs`.
///
/// The note letter is advanced by the step's letter, and the pitch's tone is advanced by the
/// diatonic tone of `rhs`; the octave of `rhs` is folded into the result, which therefore remains
/// octave-unspecific. For steps within one octave this is equivalent to adding
/// `OPitch::diatonic(rhs.generalize())`.
///
/// ```
/// use fantazia_edo12::{pitch::OPitch, step::Step};
///
/// assert_eq!(OPitch::C + Step::new(1), OPitch::D, "up a major second from C");
/// assert_eq!(
///     OPitch::C_SHARP + Step::new(1),
///     OPitch::D_SHARP,
///     "a major second above C sharp"
/// );
/// assert_eq!(OPitch::B + Step::new(1), OPitch::C_SHARP, "the octave wraps");
/// ```
const impl AddAssign<Step> for OPitch {
    fn add_assign(&mut self, rhs: Step) {
        let step = self.step as i16 + rhs.as_i8() as i16;
        let octave = step.div_euclid(7);
        self.step = unsafe { OStep::new_unchecked(step.rem_euclid(7) as i8) };
        self.tone.0 += rhs.diatonic_tone().0 - (octave * 12) as i8;
    }
}

const impl Add<Acci> for OPitch {
    type Output = Self;
    fn add(mut self, rhs: Acci) -> Self::Output {
        self += rhs;
        self
    }
}

const impl AddAssign<Acci> for OPitch {
    fn add_assign(&mut self, rhs: Acci) {
        self.tone += rhs;
    }
}

impl Sum for OPitch {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        let mut result = Self::default();
        for value in iter {
            result += value;
        }
        result
    }
}

const impl Sub for OPitch {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        if self.step >= rhs.step {
            let step = unsafe { OStep::new_unchecked(self.step as i8 - rhs.step as i8) };
            let tone = OPitchTone(self.tone.0 - rhs.tone.0);
            Self(GenericPitch { step, tone })
        } else {
            let step = unsafe { OStep::new_unchecked(self.step as i8 + (7 - rhs.step as i8)) };
            let tone = OPitchTone(self.tone.0 - rhs.tone.0 + 12);
            Self(GenericPitch { step, tone })
        }
    }
}

const impl SubAssign for OPitch {
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}

const impl Sub<Acci> for OPitch {
    type Output = Self;
    fn sub(mut self, rhs: Acci) -> Self::Output {
        self -= rhs;
        self
    }
}

const impl Sub<OStep> for OPitch {
    type Output = Self;
    fn sub(mut self, rhs: OStep) -> Self::Output {
        self -= rhs;
        self
    }
}

/// Transposes the pitch down by the diatonic interval from C to `rhs`.
///
/// This is the inverse operation of [`AddAssign<Step>`].
///
/// ```
/// use fantazia_edo12::{pitch::OPitch, step::OStep};
///
/// assert_eq!(OPitch::C - OStep::E, OPitch::A_FLAT, "down a major third from C");
/// assert_eq!(OPitch::C_SHARP - OStep::E, OPitch::A, "the sharp is kept");
/// ```
const impl SubAssign<OStep> for OPitch {
    fn sub_assign(&mut self, rhs: OStep) {
        let step = self.step as i8 - rhs as i8;
        let octave = step.div_euclid(7);
        self.step = unsafe { OStep::new_unchecked(step.rem_euclid(7)) };
        self.tone.0 += -rhs.diatonic_tone().0 - octave * 12;
    }
}

const impl SubAssign<Acci> for OPitch {
    fn sub_assign(&mut self, rhs: Acci) {
        self.tone -= rhs;
    }
}

const impl Mul<i8> for OPitch {
    type Output = Self;
    fn mul(self, rhs: i8) -> Self::Output {
        let step = self.step as i16 * rhs as i16;
        let octave = step.div_euclid(7);
        let step = unsafe { OStep::new_unchecked(step.rem_euclid(7) as i8) };
        let tone = OPitchTone((self.tone.0 as i16 * rhs as i16 - octave * 12) as i8);
        Self(GenericPitch { step, tone })
    }
}

const impl MulAssign<i8> for OPitch {
    fn mul_assign(&mut self, rhs: i8) {
        *self = *self * rhs;
    }
}

const impl Neg for OPitch {
    type Output = Self;
    fn neg(self) -> Self::Output {
        if self.step == OStep::C {
            Self(GenericPitch {
                step: OStep::C,
                tone: OPitchTone(-self.tone.0),
            })
        } else {
            Self(GenericPitch {
                step: unsafe { OStep::new_unchecked(7 - self.step as i8) },
                tone: OPitchTone(12 - self.tone.0),
            })
        }
    }
}

impl Debug for OPitch {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("OPitch")
            .field("step", &self.step)
            .field("tone", &self.tone)
            .finish()
    }
}

/// Formats the pitch as its step followed by its accidental, omitting the accidental when the
/// pitch is diatonic. The result can be parsed back with [`OPitch::from_str`].
///
/// ```
/// use fantazia_edo12::pitch::OPitch;
///
/// assert_eq!(OPitch::C.to_string(), "C");
/// assert_eq!(OPitch::C_SHARP.to_string(), "C+");
/// assert_eq!(OPitch::D_FLAT.to_string(), "D-");
/// assert_eq!(OPitch::C.with_acci(2).to_string(), "C++");
/// assert_eq!(OPitch::C.with_acci(4).to_string(), "C[+4]");
/// ```
impl Display for OPitch {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if self.is_diatonic() {
            write!(f, "{}", self.step)?;
        } else {
            write!(f, "{}{}", self.step, self.acci())?;
        }
        Ok(())
    }
}

/// Error type returned when parsing an [`OPitch`].
#[derive(Debug, Clone, Error)]
pub enum InvalidOPitch {
    /// The step part of the pitch is not a valid [`OStep`].
    #[error(transparent)]
    InvalidOStep(#[from] InvalidOStep),
    /// The accidental part of the pitch is not a valid [`Acci`].
    #[error(transparent)]
    InvalidAcci(#[from] InvalidAcci),
}

impl FromStr for OPitch {
    type Err = InvalidOPitch;

    /// Parses an [`OPitch`] from a step name optionally followed by an accidental.
    ///
    /// The step is parsed by [`OStep::from_str`] and the accidental by [`Acci::from_str`]. If no
    /// accidental is present the pitch is diatonic for its step.
    ///
    /// ```
    /// use fantazia_edo12::pitch::OPitch;
    ///
    /// assert_eq!("C".parse::<OPitch>().ok(), Some(OPitch::C));
    /// assert_eq!("c+".parse::<OPitch>().ok(), Some(OPitch::C_SHARP));
    /// assert_eq!("D-".parse::<OPitch>().ok(), Some(OPitch::D_FLAT));
    /// assert_eq!("sol".parse::<OPitch>().ok(), Some(OPitch::G));
    /// assert_eq!("C[+4]".parse::<OPitch>().ok(), Some(OPitch::C.with_acci(4)));
    /// assert_eq!("H".parse::<OPitch>().ok(), None);
    /// ```
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some(idx) = s.find(|ch: char| !ch.is_ascii_alphanumeric()) {
            let (ostep_src, acci_src) = s.split_at(idx);
            let ostep = ostep_src.parse::<OStep>()?;
            let acci = acci_src.parse::<Acci>()?;
            Ok(Self::diatonic(ostep).with_acci(acci))
        } else {
            Ok(Self::diatonic(s.parse::<OStep>()?))
        }
    }
}

// MARK: Pitch

/// An octave-specific named pitch.
///
/// A `Pitch` is the octave-specific counterpart of [`OPitch`]: it pairs a [`Step`] with a [`Tone`],
/// so it denotes a concrete key on the keyboard while preserving the spelling of the pitch.
///
/// Obtain one from an [`OPitch`] with [`fantazia_core::pitch::GeneralPitch::put_in_octave`], and go
/// back with [`fantazia_core::pitch::SpecificPitch::generalize`]. Like [`OPitch`], `Pitch` supports
/// arithmetic; the interval between two pitches is represented by [`crate::interval::Interval`].
#[derive(Copy)]
#[derive_const(
    Clone, Default, PartialEq, Eq, PartialOrd, Ord, Add, AddAssign, Sum, Sub, SubAssign, Mul,
    MulAssign, Neg, Deref, DerefMut, From, Into
)]
#[const_impl(
    Add, AddAssign, Sub, SubAssign, Mul, MulAssign, Neg, Deref, DerefMut, From, Into
)]
#[cfg_attr(
    feature = "rkyv",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Pitch(GenericPitch<Step, Tone>);
gen_pitch_consts!();

impl Pitch {
    /// Creates a new [`Pitch`] from an octave-specific step and tone.
    ///
    /// The two components are not checked for consistency. To spell a pitch according to its step,
    /// use [`Pitch::new_diatonic`] or [`Pitch::with_acci`] instead.
    #[inline(always)]
    pub const fn new(step: impl [const] Into<Step>, tone: impl [const] Into<Tone>) -> Self {
        Self(GenericPitch {
            step: step.into(),
            tone: tone.into(),
        })
    }

    /// Creates a new diatonic [`Pitch`] from its step, i.e. without any accidental.
    ///
    /// See [`Step::diatonic_tone`] for details.
    ///
    /// ```
    /// use fantazia_edo12::{pitch::Pitch, step::Step};
    ///
    /// assert_eq!(Pitch::new_diatonic(Step::C_0), Pitch::C_0);
    /// assert_eq!(Pitch::new_diatonic(Step::E_1), Pitch::E_1);
    /// ```
    #[inline(always)]
    pub const fn new_diatonic(step: impl [const] Into<Step>) -> Self {
        let step = step.into();
        Self(GenericPitch {
            step,
            tone: step.diatonic_tone(),
        })
    }

    /// Returns the pitch's index in **circle / chain of fifth**.
    ///
    /// This method is equivalent to `self.generalize().co5_order()`, so the octave specification
    /// does not influence the result.
    pub const fn co5_order(&self) -> i8 {
        self.generalize().co5_order()
    }

    /// Returns the **accidental** of the pitch, i.e. the difference between the pitch's tone and
    /// the diatonic tone of its step.
    ///
    /// ```
    /// use fantazia_edo12::pitch::Pitch;
    ///
    /// assert_eq!(Pitch::C_1.acci(), fantazia_edo12::acci::Acci::NATURAL);
    /// assert_eq!(Pitch::C_SHARP_1.acci(), fantazia_edo12::acci::Acci::SHARP);
    /// assert_eq!(Pitch::D_FLAT_1.acci(), fantazia_edo12::acci::Acci::FLAT);
    /// ```
    pub const fn acci(&self) -> Acci {
        (self.tone - self.step.diatonic_tone()).into()
    }

    /// Changes the pitch's step in place, keeping its tone.
    pub const fn with_step(mut self, step: impl [const] Into<Step>) -> Self {
        self.step = step.into();
        self
    }

    /// Changes the pitch's tone in place, keeping its step.
    pub const fn with_tone(mut self, tone: impl [const] Into<Tone>) -> Self {
        self.tone = tone.into();
        self
    }

    /// Sets the pitch's accidental in place, keeping its step and octave.
    ///
    /// **Note**: use the [`Add`] / [`Sub`] implementations with an [`Acci`] to *alter* the pitch
    /// rather than to respell it.
    pub const fn with_acci(mut self, acci: impl [const] Into<Acci>) -> Self {
        self.tone = self.step.diatonic_tone() + acci.into();
        self
    }

    /// Returns `true` if the pitch has no accidental.
    pub const fn is_diatonic(&self) -> bool {
        self.tone == self.step.diatonic_tone()
    }

    /// Returns `true` if this pitch and `other` sound the same, i.e. if they are **enharmonic**.
    ///
    /// Unlike [`OPitch::is_enharmonic`], the octave is significant here.
    pub const fn is_enharmonic(&self, other: &Self) -> bool {
        self.tone == other.tone
    }
}

impl Debug for Pitch {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("Pitch")
            .field("step", &self.step)
            .field("tone", &self.tone)
            .finish()
    }
}

/// Formats the pitch as its generalized form followed by `_` and its octave number, e.g. `C_1` or
/// `F+_-1`. The result can be parsed back with [`Pitch::from_str`].
///
/// ```
/// use fantazia_edo12::pitch::Pitch;
///
/// assert_eq!(Pitch::C_1.to_string(), "C_1");
/// assert_eq!(Pitch::C_SHARP_1.to_string(), "C+_1");
/// assert_eq!(Pitch::D_FLAT_M1.to_string(), "D-_-1");
/// ```
impl Display for Pitch {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}_{}", self.generalize(), self.octave())
    }
}

const impl SpecificPitch<OPitch> for Pitch {
    fn octave(&self) -> i8 {
        self.0.octave()
    }
    fn generalize(self) -> OPitch {
        self.0.generalize().into()
    }
}

impl ToFrequency for Pitch {
    fn to_frequency(&self) -> Frequency {
        self.0.to_frequency()
    }
}

/// Error type returned when parsing a [`Pitch`].
#[derive(Debug, Clone, Error)]
pub enum InvalidPitch {
    /// The pitch part of the input is not a valid [`OPitch`].
    #[error(transparent)]
    InvalidOPitch(#[from] InvalidOPitch),
    /// Missing octave specification.
    #[error("missing octave specification")]
    MissingOctave,
    /// Invalid octave number.
    #[error("invalid octave - {0}")]
    InvalidOctave(#[from] ParseIntError),
}

impl FromStr for Pitch {
    type Err = InvalidPitch;

    /// Parses a [`Pitch`] from an octave-unspecific pitch and an octave number separated by `_`,
    /// e.g. `C_0`, `F+_1` or `B-_-1`.
    ///
    /// The pitch is parsed by [`OPitch::from_str`] and is then put into the given octave.
    ///
    /// ```
    /// use fantazia_edo12::pitch::Pitch;
    ///
    /// assert_eq!("C_0".parse::<Pitch>().ok(), Some(Pitch::C_0));
    /// assert_eq!("F+_1".parse::<Pitch>().ok(), Some(Pitch::F_SHARP_1));
    /// assert_eq!("B-_-1".parse::<Pitch>().ok(), Some(Pitch::B_FLAT_M1));
    /// assert_eq!("C".parse::<Pitch>().ok(), None);
    /// ```
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some((opitch_src, octave_src)) = s.rsplit_once('_') {
            let octave = octave_src.parse()?;
            Ok(opitch_src.parse::<OPitch>()?.put_in_octave(octave))
        } else {
            Err(InvalidPitch::MissingOctave)
        }
    }
}

const impl Add<Acci> for Pitch {
    type Output = Self;

    fn add(mut self, rhs: Acci) -> Self::Output {
        self += rhs;
        self
    }
}

const impl AddAssign<Acci> for Pitch {
    fn add_assign(&mut self, rhs: Acci) {
        self.tone += rhs;
    }
}

const impl Sub<Acci> for Pitch {
    type Output = Self;

    fn sub(mut self, rhs: Acci) -> Self::Output {
        self -= rhs;
        self
    }
}

const impl SubAssign<Acci> for Pitch {
    fn sub_assign(&mut self, rhs: Acci) {
        self.tone -= rhs;
    }
}

const impl Abs for Pitch {
    type Output = Self;
    fn abs(mut self) -> Self::Output {
        self.abs_assign();
        self
    }
}

const impl AbsAssign for Pitch {
    fn abs_assign(&mut self) {
        if self.step.0 < 0 {
            self.step = -self.step;
            self.tone = -self.tone;
        }
    }
}

const impl Sign for Pitch {
    fn sign(&self) -> std::cmp::Ordering {
        self.cmp(&Self::C_0)
    }
}

#[cfg(feature = "macro")]
impl ToTokens for OPitch {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let value = self.0;
        tokens.extend(quote! {
            ::fantazia::edo12::pitch::OPitch::from(#value)
        });
    }
}

#[cfg(feature = "macro")]
impl ToTokens for Pitch {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let value = self.0;
        tokens.extend(quote! {
            ::fantazia::edo12::pitch::Pitch::from(#value)
        });
    }
}

#[cfg(test)]
mod tests {
    use std::cmp::Ordering;

    use fantazia_core::{
        arith::{Abs, Sign},
        pitch::{GeneralPitch, SpecificPitch, ToFrequency},
    };

    use super::*;

    fn assert_frequency(actual: Frequency, expected: f64) {
        assert!(
            (actual.get() - expected).abs() < 1e-12,
            "expected {expected}, got {}",
            actual.get()
        );
    }

    #[test]
    fn test_const_gen() {
        // Octave-unspecific pitches generated from the pitch names.
        let expected = [
            (OPitch::C, OStep::C, 0),
            (OPitch::C_SHARP, OStep::C, 1),
            (OPitch::D_FLAT, OStep::D, 1),
            (OPitch::D, OStep::D, 2),
            (OPitch::D_SHARP, OStep::D, 3),
            (OPitch::E_FLAT, OStep::E, 3),
            (OPitch::E, OStep::E, 4),
            (OPitch::F, OStep::F, 5),
            (OPitch::F_SHARP, OStep::F, 6),
            (OPitch::G_FLAT, OStep::G, 6),
            (OPitch::G, OStep::G, 7),
            (OPitch::G_SHARP, OStep::G, 8),
            (OPitch::A_FLAT, OStep::A, 8),
            (OPitch::A, OStep::A, 9),
            (OPitch::A_SHARP, OStep::A, 10),
            (OPitch::B_FLAT, OStep::B, 10),
            (OPitch::B, OStep::B, 11),
        ];
        for (opitch, step, tone) in expected {
            assert_eq!(opitch.step, step, "{opitch:?}");
            assert_eq!(opitch.tone.as_i8(), tone, "{opitch:?}");
        }

        // Octave-specific pitches generated from the pitch names.
        assert_eq!(Pitch::C_M2.step, Step::C_M2);
        assert_eq!(Pitch::C_M2.tone, Tone::C_M2);
        assert_eq!(Pitch::C_0.step, Step::C_0);
        assert_eq!(Pitch::C_0.tone, Tone::C_0);
        assert_eq!(Pitch::C_1.tone, Tone::C_1);
        assert_eq!(Pitch::C_SHARP_2.tone.as_i8(), 25);
        assert_eq!(Pitch::B_M1.tone.as_i8(), -1);
    }

    #[test]
    fn test_opitch_construction() {
        assert_eq!(OPitch::new(OStep::C, 0), OPitch::C);
        assert_eq!(OPitch::new(OStep::C, 1), OPitch::C_SHARP);
        assert_eq!(OPitch::diatonic(OStep::D).with_acci(-1), OPitch::D_FLAT);
        assert_eq!(OPitch::diatonic(OStep::E), OPitch::E);
        assert_eq!(OPitch::diatonic(OStep::E).tone.as_i8(), 4);
        assert_eq!(OPitch::default(), OPitch::C);

        assert_eq!(OPitch::C.acci(), Acci::NATURAL);
        assert_eq!(OPitch::C_SHARP.acci(), Acci::SHARP);
        assert_eq!(OPitch::D_FLAT.acci(), Acci::FLAT);
        assert_eq!(OPitch::C.double_sharp().acci(), Acci::DOUBLE_SHARP);
        assert_eq!(OPitch::B.flat().acci(), Acci::FLAT);

        // `with_step` keeps the tone, so it respells rather than transposes.
        assert_eq!(OPitch::D_FLAT.with_step(OStep::C), OPitch::C_SHARP);
        assert_eq!(OPitch::C.with_tone(1), OPitch::C_SHARP);
        assert_eq!(
            OPitch::C_SHARP.with_acci(Acci::FLAT),
            OPitch::C.with_acci(-1)
        );
        assert_eq!(OPitch::C.with_acci(2), OPitch::C.double_sharp());
    }

    #[test]
    fn test_opitch_predicates() {
        assert!(OPitch::C.is_diatonic());
        assert!(OPitch::E.is_diatonic());
        assert!(OPitch::B.is_diatonic());
        assert!(!OPitch::C_SHARP.is_diatonic());
        assert!(!OPitch::D_FLAT.is_diatonic());
        assert!(!OPitch::C.double_sharp().is_diatonic());

        assert!(OPitch::C_SHARP.is_enharmonic(&OPitch::D_FLAT));
        assert!(OPitch::C.is_enharmonic(&OPitch::B.with_acci(1)));
        assert!(!OPitch::C_SHARP.is_enharmonic(&OPitch::D));
        assert_ne!(OPitch::C_SHARP, OPitch::D_FLAT);
    }

    #[test]
    fn test_opitch_co5_order() {
        let chain = [
            (-6, OPitch::G_FLAT),
            (-5, OPitch::D_FLAT),
            (-4, OPitch::A_FLAT),
            (-3, OPitch::E_FLAT),
            (-2, OPitch::B_FLAT),
            (-1, OPitch::F),
            (0, OPitch::C),
            (1, OPitch::G),
            (2, OPitch::D),
            (3, OPitch::A),
            (4, OPitch::E),
            (5, OPitch::B),
            (6, OPitch::F_SHARP),
            (7, OPitch::C_SHARP),
        ];
        for (order, pitch) in chain {
            assert_eq!(OPitch::from_co5_order(order), pitch, "order {order}");
            assert_eq!(pitch.co5_order(), order, "{pitch}");
        }

        // The order and the pitch are inverse mappings over a wide range.
        for order in -30..=30 {
            assert_eq!(OPitch::from_co5_order(order).co5_order(), order);
        }

        // Accidentals move a pitch by a whole chain of fifths.
        assert_eq!(OPitch::C.double_sharp().co5_order(), 14);
        assert_eq!(OPitch::C.double_flat().co5_order(), -14);
    }

    #[test]
    fn test_opitch_display() {
        let expected = [
            (OPitch::C, "C"),
            (OPitch::C_SHARP, "C+"),
            (OPitch::D_FLAT, "D-"),
            (OPitch::D, "D"),
            (OPitch::D_SHARP, "D+"),
            (OPitch::E_FLAT, "E-"),
            (OPitch::E, "E"),
            (OPitch::F, "F"),
            (OPitch::F_SHARP, "F+"),
            (OPitch::G_FLAT, "G-"),
            (OPitch::G, "G"),
            (OPitch::G_SHARP, "G+"),
            (OPitch::A_FLAT, "A-"),
            (OPitch::A, "A"),
            (OPitch::A_SHARP, "A+"),
            (OPitch::B_FLAT, "B-"),
            (OPitch::B, "B"),
        ];
        for (opitch, display_str) in expected {
            assert_eq!(opitch.to_string(), display_str);
            assert_eq!(display_str.parse::<OPitch>().unwrap(), opitch);
        }

        // Wide accidentals use the bracketed representation.
        assert_eq!(OPitch::C.with_acci(4).to_string(), "C[+4]");
        assert_eq!(OPitch::C.with_acci(-4).to_string(), "C[-4]");
        assert_eq!(
            OPitch::C
                .with_acci(4)
                .to_string()
                .parse::<OPitch>()
                .unwrap(),
            OPitch::C.with_acci(4)
        );

        assert_eq!(
            format!("{:?}", OPitch::C_SHARP),
            "OPitch { step: C, tone: OPitchTone(1) }"
        );
    }

    #[test]
    fn test_opitch_parse_roundtrip() {
        // Every accidental in a representative range round-trips through `Display`.
        for step in [
            OStep::C,
            OStep::D,
            OStep::E,
            OStep::F,
            OStep::G,
            OStep::A,
            OStep::B,
        ] {
            for acci in -5..=5 {
                let opitch = OPitch::diatonic(step).with_acci(acci);
                let src = opitch.to_string();
                assert_eq!(src.parse::<OPitch>().unwrap(), opitch, "parse `{src}`");
            }
        }

        assert_eq!("c".parse::<OPitch>().unwrap(), OPitch::C);
        assert_eq!("do".parse::<OPitch>().unwrap(), OPitch::C);
        assert_eq!("sol-".parse::<OPitch>().unwrap(), OPitch::G_FLAT);
        assert_eq!("ti+".parse::<OPitch>().unwrap(), OPitch::B.with_acci(1));

        for invalid in ["", "H", "C#", "C++x", "0"] {
            assert!(invalid.parse::<OPitch>().is_err(), "`{invalid}`");
        }
    }

    #[test]
    fn test_opitch_arithmetic() {
        assert_eq!(OPitch::C + OPitch::E, OPitch::E);
        assert_eq!(OPitch::E + OPitch::A, OPitch::C_SHARP);
        assert_eq!(OPitch::B + OPitch::D, OPitch::C_SHARP);
        assert_eq!(OPitch::C - OPitch::C, OPitch::C);
        assert_eq!(OPitch::E - OPitch::C, OPitch::E);
        assert_eq!(OPitch::C - OPitch::E, OPitch::A_FLAT);
        assert_eq!(OPitch::G - OPitch::C, OPitch::G);
        assert_eq!(OPitch::G * 2, OPitch::D);
        let zero = 0;
        assert_eq!(OPitch::E * zero, OPitch::C);
        assert_eq!(-OPitch::C, OPitch::C);
        assert_eq!(-OPitch::G, OPitch::F);
        assert_eq!(-OPitch::C_SHARP, OPitch::C.with_acci(-1));
        assert_eq!(
            [OPitch::C, OPitch::E].into_iter().sum::<OPitch>(),
            OPitch::E
        );
        assert_eq!(std::iter::empty::<OPitch>().sum::<OPitch>(), OPitch::C);

        // Adding a pitch and subtracting it again is the identity.
        for pitch in [OPitch::C, OPitch::D_FLAT, OPitch::F_SHARP, OPitch::B] {
            assert_eq!(pitch + OPitch::G - OPitch::G, pitch);
        }
    }

    #[test]
    fn test_opitch_acci_arithmetic() {
        assert_eq!(OPitch::C + Acci::SHARP, OPitch::C_SHARP);
        assert_eq!(OPitch::C + Acci::FLAT, OPitch::C.with_acci(-1));
        assert_eq!(OPitch::C - Acci::SHARP, OPitch::C.with_acci(-1));
        assert_eq!(OPitch::C.double_sharp() - Acci::SHARP, OPitch::C_SHARP);

        let mut pitch = OPitch::C;
        pitch += Acci::DOUBLE_SHARP;
        assert_eq!(pitch.acci(), Acci::DOUBLE_SHARP);
        pitch -= Acci::SHARP;
        assert_eq!(pitch.acci(), Acci::SHARP);
    }

    #[test]
    fn test_opitch_step_transposition() {
        assert_eq!(OPitch::C + Step::new(1), OPitch::D);
        assert_eq!(OPitch::C + Step::new(2), OPitch::E);
        assert_eq!(OPitch::C + Step::new(4), OPitch::G);
        assert_eq!(OPitch::C_SHARP + Step::new(1), OPitch::D_SHARP);
        assert_eq!(OPitch::B + Step::new(1), OPitch::C_SHARP);
        assert_eq!(OPitch::C + Step::C_1, OPitch::C);
        assert_eq!(OPitch::C + Step::F_M1, OPitch::F);

        assert_eq!(OPitch::C - OStep::E, OPitch::A_FLAT);
        assert_eq!(OPitch::C_SHARP - OStep::E, OPitch::A);
        assert_eq!(OPitch::C - OStep::C, OPitch::C);
        assert_eq!(OPitch::B - OStep::C, OPitch::B);
        assert_eq!(OPitch::C - OStep::B, OPitch::D_FLAT);
    }

    #[test]
    fn test_pitch_construction() {
        assert_eq!(Pitch::new(Step::C_0, Tone::C_0), Pitch::C_0);
        assert_eq!(Pitch::new_diatonic(Step::C_0), Pitch::C_0);
        assert_eq!(Pitch::new_diatonic(Step::E_1), Pitch::E_1);
        assert_eq!(Pitch::new_diatonic(Step::B_M1), Pitch::B_M1);

        assert_eq!(Pitch::C_1.acci(), Acci::NATURAL);
        assert_eq!(Pitch::C_SHARP_1.acci(), Acci::SHARP);
        assert_eq!(Pitch::D_FLAT_1.acci(), Acci::FLAT);
        assert_eq!(Pitch::C_1.co5_order(), 0);
        assert_eq!(Pitch::C_SHARP_1.co5_order(), 7);
        assert_eq!(Pitch::F_M1.co5_order(), -1);

        // `with_step` keeps the tone, so it respells rather than transposes.
        assert_eq!(Pitch::D_FLAT_0.with_step(Step::C_0), Pitch::C_SHARP_0);
        assert_eq!(Pitch::C_0.with_tone(Tone::new(1)), Pitch::C_SHARP_0);
        assert_eq!(Pitch::C_0.with_acci(Acci::SHARP), Pitch::C_SHARP_0);
        assert_eq!(
            Pitch::new_diatonic(Step::C_1).with_acci(Acci::DOUBLE_SHARP),
            Pitch::C_1.with_tone(Tone::new(14))
        );
    }

    #[test]
    fn test_pitch_predicates() {
        assert!(Pitch::C_0.is_diatonic());
        assert!(Pitch::E_1.is_diatonic());
        assert!(!Pitch::C_SHARP_0.is_diatonic());
        assert!(Pitch::C_SHARP_0.is_enharmonic(&Pitch::D_FLAT_0));
        assert!(!Pitch::C_SHARP_0.is_enharmonic(&Pitch::D_FLAT_1));
    }

    #[test]
    fn test_pitch_octave() {
        assert_eq!(Pitch::C_0.octave(), 0);
        assert_eq!(Pitch::C_1.octave(), 1);
        assert_eq!(Pitch::B_M1.octave(), -1);
        assert_eq!(Pitch::C_1.generalize(), OPitch::C);
        assert_eq!(Pitch::C_SHARP_1.generalize(), OPitch::C_SHARP);
        assert_eq!(Pitch::C_1.with_octave(0), Pitch::C_0);
        assert_eq!(Pitch::C_0.with_octave(2), Pitch::C_2);
        assert_eq!(Pitch::C_1.shift_octave(-2), Pitch::C_M1);
    }

    #[test]
    fn test_pitch_arithmetic() {
        assert_eq!(Pitch::C_0 + Pitch::C_1, Pitch::C_1);
        assert_eq!(Pitch::C_1 - Pitch::C_0, Pitch::C_1);
        assert_eq!(Pitch::C_1 * 2, Pitch::C_2);
        assert_eq!(-Pitch::C_1, Pitch::C_M1);
        assert_eq!(
            [Pitch::C_0, Pitch::C_1].into_iter().sum::<Pitch>(),
            Pitch::C_1
        );

        let mut pitch = Pitch::C_0;
        pitch += Acci::SHARP;
        assert_eq!(pitch, Pitch::C_SHARP_0);
        pitch -= Acci::SHARP;
        assert_eq!(pitch, Pitch::C_0);

        assert_eq!(Pitch::C_M1.abs(), Pitch::C_1);
        assert_eq!(Pitch::C_M1.sign(), Ordering::Less);
        assert_eq!(Pitch::C_0.sign(), Ordering::Equal);
        assert_eq!(Pitch::C_1.sign(), Ordering::Greater);

        let mut pitch = Pitch::C_1;
        pitch = -pitch;
        assert_eq!(pitch, Pitch::C_M1);
        pitch.abs_assign();
        assert_eq!(pitch, Pitch::C_1);
    }

    #[test]
    fn test_pitch_display_and_parse() {
        assert_eq!(Pitch::C_0.to_string(), "C_0");
        assert_eq!(Pitch::C_1.to_string(), "C_1");
        assert_eq!(Pitch::C_SHARP_1.to_string(), "C+_1");
        assert_eq!(Pitch::D_FLAT_M1.to_string(), "D-_-1");
        assert_eq!(Pitch::B_M1.to_string(), "B_-1");

        for octave in -2..=2 {
            for opitch in [
                OPitch::C,
                OPitch::C_SHARP,
                OPitch::D_FLAT,
                OPitch::E,
                OPitch::F,
                OPitch::F_SHARP,
                OPitch::B,
            ] {
                let pitch = opitch.put_in_octave(octave);
                let src = pitch.to_string();
                assert_eq!(src.parse::<Pitch>().unwrap(), pitch, "parse `{src}`");
            }
        }

        assert_eq!("c_1".parse::<Pitch>().unwrap(), Pitch::C_1);
        assert_eq!("sol_-1".parse::<Pitch>().unwrap(), Pitch::G_M1);
        assert_eq!("C[+4]_0".parse::<Pitch>().unwrap(), Pitch::C_0.with_acci(4));
    }

    #[test]
    fn test_pitch_parse_errors() {
        assert!(matches!(
            "C".parse::<Pitch>(),
            Err(InvalidPitch::MissingOctave)
        ));
        assert!(matches!(
            "C_x".parse::<Pitch>(),
            Err(InvalidPitch::InvalidOctave(_))
        ));
        assert!(matches!(
            "H_0".parse::<Pitch>(),
            Err(InvalidPitch::InvalidOPitch(_))
        ));
        assert_eq!(
            "C".parse::<Pitch>().unwrap_err().to_string(),
            "missing octave specification"
        );
    }

    #[test]
    fn test_pitch_frequency() {
        assert_frequency(Pitch::C_0.to_frequency(), 1.0);
        assert_frequency(Pitch::C_1.to_frequency(), 2.0);
        assert_frequency(Pitch::C_M1.to_frequency(), 0.5);
        assert_frequency(Pitch::C_SHARP_1.to_frequency(), 2f64.powf(13.0 / 12.0));
        assert_frequency(Pitch::A_0.to_frequency(), 2f64.powf(9.0 / 12.0));
        assert_frequency(Pitch::A_1.to_frequency(), 2f64.powf(21.0 / 12.0));
        assert_frequency(OPitch::A.to_frequency(), 2f64.powf(9.0 / 12.0));
    }

    #[test]
    fn test_pitch_debug() {
        assert_eq!(
            format!("{:?}", Pitch::C_1),
            "Pitch { step: Step(7), tone: Tone(12) }"
        );
    }
}

//! Named pitch representation. In `fantazia` a named pitch is represented by its **step** and **tone**
//! value. Tone determines how the pitch sounds and step adds information about how the pitch is named.
//! This naming system distinguishes **enharmonics** effectively. For example, _F sharp_ and _G flat_
//! are enharmonics but are named differently.
//!
//! This module provides:
//! - the [`OPitch`] type for octave-unspecific pitch.
//! - the [`Pitch`] type for octave-specific pitch.

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

    pub const fn is_diatonic(&self) -> bool {
        self.tone == self.step.diatonic_tone()
    }

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

const impl AddAssign<Step> for OPitch {
    fn add_assign(&mut self, rhs: Step) {
        let mut step = self.step as i8 + rhs.as_i8();
        if step >= 7 {
            self.tone.0 -= 12;
            step -= 7;
        }
        self.step = unsafe { OStep::new_unchecked(step) };
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

const impl SubAssign<OStep> for OPitch {
    fn sub_assign(&mut self, rhs: OStep) {
        let mut step = self.step as i8 - rhs as i8;
        if step < 0 {
            self.tone.0 += 12;
            step += 7;
        }
        self.step = unsafe { OStep::new_unchecked(step) };
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

#[derive(Debug, Clone, Error)]
pub enum InvalidOPitch {
    #[error(transparent)]
    InvalidOStep(#[from] InvalidOStep),
    #[error(transparent)]
    InvalidAcci(#[from] InvalidAcci),
}

impl FromStr for OPitch {
    type Err = InvalidOPitch;
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

#[derive(Copy)]
#[derive_const(
    Clone, Default, PartialEq, Eq, PartialOrd, Ord, Add, AddAssign, Sum, Sub, SubAssign, Mul,
    MulAssign, Neg, Deref, DerefMut, From, Into
)]
#[const_impl(
    Add, AddAssign, Sub, SubAssign, Mul, MulAssign, Neg, Deref, DerefMut, From, Into
)]
pub struct Pitch(GenericPitch<Step, Tone>);
gen_pitch_consts!();

impl Pitch {
    #[inline(always)]
    pub const fn new(step: impl [const] Into<Step>, tone: impl [const] Into<Tone>) -> Self {
        Self(GenericPitch {
            step: step.into(),
            tone: tone.into(),
        })
    }

    #[inline(always)]
    pub const fn new_diatonic(step: impl [const] Into<Step>) -> Self {
        let step = step.into();
        Self(GenericPitch {
            step,
            tone: step.diatonic_tone(),
        })
    }

    pub const fn co5_order(&self) -> i8 {
        self.generalize().co5_order()
    }

    pub const fn acci(&self) -> Acci {
        (self.tone - self.step.diatonic_tone()).into()
    }

    pub const fn with_step(mut self, step: impl [const] Into<Step>) -> Self {
        self.step = step.into();
        self
    }

    pub const fn with_tone(mut self, tone: impl [const] Into<Tone>) -> Self {
        self.tone = tone.into();
        self
    }

    pub const fn with_acci(mut self, acci: impl [const] Into<Acci>) -> Self {
        self.tone = self.step.diatonic_tone() + acci.into();
        self
    }

    pub const fn is_diatonic(&self) -> bool {
        self.tone == self.step.diatonic_tone()
    }

    pub const fn is_enharmonic(&self, other: &Self) -> bool {
        self.tone == other.tone
    }
}

const impl SpecificPitch<OPitch> for Pitch {
    fn octave(&self) -> i8 {
        self.0.octave().into()
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

#[derive(Debug, Clone, Error)]
pub enum InvalidPitch {
    #[error(transparent)]
    InvalidOPitch(#[from] InvalidOPitch),
    #[error("missing octave specification")]
    MissingOctave,
    #[error("invalid octave - {0}")]
    InvalidOctave(#[from] ParseIntError),
}

impl FromStr for Pitch {
    type Err = InvalidPitch;
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
    use super::*;

    #[test]
    fn test_const_gen() {
        // ensures that the proc-macro has created correct constants
        let _opitch_values = vec![
            OPitch::C,
            OPitch::C_SHARP,
            OPitch::D_FLAT,
            OPitch::D,
            OPitch::D_SHARP,
            OPitch::E_FLAT,
            OPitch::E,
            OPitch::F,
            OPitch::F_SHARP,
            OPitch::G_FLAT,
            OPitch::G,
            OPitch::G_SHARP,
            OPitch::A_FLAT,
            OPitch::A,
            OPitch::A_SHARP,
            OPitch::B_FLAT,
            OPitch::B,
        ];
        let _pitch_values = vec![
            Pitch::C_2,
            Pitch::C_1,
            Pitch::C_0,
            Pitch::C_M1,
            Pitch::C_M2,
            Pitch::C_SHARP_2,
            Pitch::C_SHARP_1,
            Pitch::C_SHARP_0,
            Pitch::C_SHARP_M1,
            Pitch::C_SHARP_M2,
        ];
    }

    #[test]
    fn test_opitch_diaplay() {
        let expected = vec![
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
    }

    #[test]
    fn test_acci_modify() {
        assert_eq!(OPitch::C.sharp(), OPitch::C_SHARP);
        assert_eq!(OPitch::C_SHARP.sharp(), OPitch::C.double_sharp());
        assert_eq!(OPitch::E.flat(), OPitch::E_FLAT);
    }
}

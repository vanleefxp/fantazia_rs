//! Pitch-related traits and types.

use std::{iter::Sum, marker::Destruct, ops::AddAssign};

use derive_more::{Add, AddAssign, Mul, MulAssign, Neg, Sub, SubAssign};
#[cfg(feature = "macro")]
use proc_macro2::TokenStream;
#[cfg(feature = "macro")]
use quote::{ToTokens, quote};

use typed_floats::{Powf, tf64};

/// Type alias used to represent frequency.
pub type Frequency = tf64::Positive;
/// Frequency ratio between two neighboring periods for periodic scales.
pub type FrequencyPeriod = tf64::StrictlyPositiveFinite;
/// Relative position of a tone in a frequency period
pub type RelativePos = tf64::NonNaN;

/// A pitch whose frequency can be determined by frequency period and its relative position in one period.
pub const trait PeriodicTone {
    #[inline(always)]
    /// Frequency ratio between two neighboring periods for periodic scales.
    fn frequency_period(&self) -> FrequencyPeriod {
        const { FrequencyPeriod::new(2.).ok().unwrap() }
    }
    /// Relative position of a tone in a frequency period
    fn relative_pos(&self) -> RelativePos;
}

/// Turns the pitch object into a frequency relative to middle C.
pub const trait ToFrequency {
    /// Performs the pitch-to-frequency conversion.
    fn to_frequency(&self) -> Frequency;
}

impl<T: PeriodicTone> ToFrequency for T {
    fn to_frequency(&self) -> Frequency {
        self.frequency_period().powf(self.relative_pos())
    }
}

/// An octave-specific pitch object, like a compound interval or a specific pitch on the piano keyboard.
pub const trait SpecificPitch<G> {
    /// Get the index of octave the pitch is in.
    fn octave(&self) -> i8;
    /// Returns the pitch object's octave-unspecific version.
    fn generalize(self) -> G;

    /// Puts the pitch object into the specified octave.
    fn with_octave(self, octave: i8) -> Self
    where
        Self: Sized,
        G: [const] GeneralPitch<Self>,
    {
        self.generalize().put_in_octave(octave)
    }

    /// Shifts up (or down if `octave` is negative) the pitch by `octave` octaves.
    fn shift_octave(self, octave: i8) -> Self
    where
        Self: Sized,
        G: [const] GeneralPitch<Self>,
    {
        let cur_octave = self.octave();
        self.with_octave(cur_octave + octave)
    }
}

/// Like [`SpecificPitch`], but more information is needed to determine which octave the pitch is in.
pub const trait SpecificPitchIndeterministic<G> {
    /// Returns the pitch object's octave-unspecific version, given which octave the pitch is in.
    fn generalize_with_octave(self, octave: i8) -> G;
}

/// An octave-unspecific pitch object, like a pitch class or a simple interval.
pub const trait GeneralPitch<S> {
    /// Puts the pitch object into the specified octave.
    fn put_in_octave(self, octave: i8) -> S;
}

/// A generic template for representing named pitches, either [general](`GeneralPitch`) or
/// [specific](`SpecificPitch`).
#[derive(Debug, Copy, Hash)]
#[derive_const(
    Clone, PartialEq, Eq, PartialOrd, Ord, Default, Add, AddAssign, Sub, SubAssign, Mul, MulAssign,
    Neg
)]
#[const_impl]
#[cfg_attr(
    feature = "rkyv",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct GenericPitch<S, T> {
    /// The nominal **step** of the pitch. Adds information about how the pitch is written.
    pub step: S,
    /// The **tone** of the pitch. Uniquely determines how the pitch sounds.
    pub tone: T,
}

const impl<S1, T1, S2, T2> GeneralPitch<GenericPitch<S2, T2>> for GenericPitch<S1, T1>
where
    S1: [const] GeneralPitch<S2>,
    T1: [const] GeneralPitch<T2>,
    Self: [const] Destruct,
{
    #[inline(always)]
    fn put_in_octave(self, octave: i8) -> GenericPitch<S2, T2> {
        GenericPitch {
            step: self.step.put_in_octave(octave),
            tone: self.tone.put_in_octave(octave),
        }
    }
}

const impl<S1, T1, S2, T2> SpecificPitch<GenericPitch<S2, T2>> for GenericPitch<S1, T1>
where
    S1: [const] SpecificPitch<S2>,
    T1: [const] SpecificPitchIndeterministic<T2>,
    Self: [const] Destruct,
{
    #[inline(always)]
    fn octave(&self) -> i8 {
        self.step.octave()
    }

    fn generalize(self) -> GenericPitch<S2, T2> {
        let octave = self.octave();
        GenericPitch {
            step: self.step.generalize(),
            tone: self.tone.generalize_with_octave(octave),
        }
    }
}

const impl<S, T> ToFrequency for GenericPitch<S, T>
where
    T: [const] ToFrequency,
{
    fn to_frequency(&self) -> Frequency {
        self.tone.to_frequency()
    }
}

impl<S, T> Sum for GenericPitch<S, T>
where
    Self: Default + AddAssign,
{
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        let mut result = Self::default();
        for item in iter {
            result += item;
        }
        result
    }
}

#[cfg(feature = "macro")]
impl<S: ToTokens, T: ToTokens> ToTokens for GenericPitch<S, T> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let GenericPitch { step, tone } = self;
        tokens.extend(quote! {
            ::fantazia::pitch::GenericPitch {
                step: #step,
                tone: #tone,
            }
        });
    }
}

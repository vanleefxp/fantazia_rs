//! Standard **12-tone equal temperament** (**12-TET** / **12-EDO**) pitch arithmetic.
//!
//! This crate is the 12-EDO model of the `fantazia` workspace. It provides octave-unspecific and
//! octave-specific representations of *steps*, *tones*, *pitches*, *accidentals* and *intervals*.
//! All of them are small integer types, so pitch arithmetic can be done in `const` contexts.
//!
//! # Pitch model
//!
//! A named pitch is described by two independent components (see [`pitch::OPitch`]):
//!
//! - the **step**, i.e. the note letter `C`–`B`, which carries the *spelling* of the pitch;
//! - the **tone**, i.e. the number of semitones above middle `C`, which determines the *sound*.
//!
//! Keeping both components makes *enharmonic* spellings distinguishable:
//! [`pitch::OPitch::C_SHARP`] and [`pitch::OPitch::D_FLAT`] sound the same, but have different
//! steps. An accidental is the difference between the tone of a pitch and the diatonic tone of its
//! step (see [`acci::Acci`]).
//!
//! The [specific](fantazia_core::pitch::SpecificPitch) types ([`step::Step`], [`tone::Tone`],
//! [`pitch::Pitch`] and [`interval::Interval`]) add an octave to the corresponding
//! octave-unspecific types, while [`fantazia_core::pitch::GeneralPitch`] puts an octave-unspecific
//! value back into a chosen octave.
//!
//! # Circle of fifths
//!
//! Steps and pitches have a position on the **circle (chain) of fifths** as well as in the
//! chromatic scale. This position can be obtained with `co5_order` and used to construct a pitch
//! with `from_co5_order`; it makes it easy to reason about key signatures and enharmonic
//! respelling.
//!
//! # Features
//!
//! Every module is behind a feature flag:
//!
//! - `step` (default): the [`step`] module. Also enables `tone`.
//! - `tone` (default): the [`tone`] module. Also enables `acci`.
//! - `acci` (default): the [`acci`] module. Also enables `tone`, because tones and accidentals
//!   provide conversions and arithmetic for each other.
//! - `pitch` (default): the [`pitch`] module. Enables `step`, `tone` and `acci`.
//! - `interval` (default): the [`interval`] module. Enables `pitch`.
//! - `acci-pref` (default): the [`acci_pref`] module. Enables `pitch`.
//! - `serde`: `serde` support for the public types.
//! - `rkyv`: `rkyv` support for the public types, except `AcciPref`, which only supports `serde`.
//! - `macro`: `quote::ToTokens` implementations, so that values can be embedded in generated code
//!   as `fantazia` tokens.
//!
//! # Examples
//!
//! ```
//! use fantazia_edo12::{interval::OInterval, pitch::OPitch, step::Step};
//!
//! // Two perfect fifths above C make D.
//! let d = OPitch::C + OInterval::PERFECT_FIFTH + OInterval::PERFECT_FIFTH;
//! assert_eq!(d, OPitch::D);
//!
//! // C sharp and D flat sound the same, but are spelled differently.
//! assert!(OPitch::C_SHARP.is_enharmonic(&OPitch::D_FLAT));
//! assert_ne!(OPitch::C_SHARP, OPitch::D_FLAT);
//!
//! // All types are plain integers, so they also work in `const` contexts.
//! const C_IN_OCTAVE_1: Step = Step::new(7);
//! assert_eq!(C_IN_OCTAVE_1, Step::C_1);
//! ```

#![allow(internal_features)]
#![warn(missing_docs)]
#![cfg_attr(not(test), deny(clippy::unwrap_used))]
#![feature(
    const_trait_impl,
    const_convert,
    const_ops,
    const_default,
    const_cmp,
    const_clone,
    const_closures,
    const_destruct,
    const_index,
    const_result_trait_fn,
    derive_const,
    pattern_type_range_trait,
    pattern_types,
    impl_restriction,
    formatting_options
)]

/// Accidentals and accidental arithmetic.
#[cfg(feature = "acci")]
pub mod acci;
/// Mapping chromatic tones to named pitches according to an accidental preference.
#[cfg(feature = "acci-pref")]
pub mod acci_pref;
/// Intervals and interval arithmetic.
#[cfg(feature = "interval")]
pub mod interval;
/// Named pitches and pitch arithmetic.
#[cfg(feature = "pitch")]
pub mod pitch;
/// Octave-specific and octave-unspecific note steps.
#[cfg(feature = "step")]
pub mod step;
/// Chromatic tones measured in semitones.
#[cfg(feature = "tone")]
pub mod tone;

#[cfg(all(test, feature = "macro", feature = "interval", feature = "acci-pref"))]
mod macro_tests {
    use quote::ToTokens;

    use crate::{
        acci::Acci,
        acci_pref::AcciPref,
        interval::{Interval, IntervalQual, OInterval},
        pitch::{OPitch, Pitch},
        step::{OStep, Step},
        tone::{OPitchTone, OTone, Tone},
    };

    #[test]
    fn tokens_reference_the_fantazia_paths() {
        let tokens = [
            OStep::C.to_token_stream().to_string(),
            Step::C_1.to_token_stream().to_string(),
            OTone::C_SHARP.to_token_stream().to_string(),
            OPitchTone::new(1).to_token_stream().to_string(),
            Tone::C_1.to_token_stream().to_string(),
            Acci::SHARP.to_token_stream().to_string(),
            OPitch::C_SHARP.to_token_stream().to_string(),
            Pitch::C_1.to_token_stream().to_string(),
            IntervalQual::MAJOR.to_token_stream().to_string(),
            OInterval::MAJOR_THIRD.to_token_stream().to_string(),
            Interval::from(Pitch::E_1).to_token_stream().to_string(),
            AcciPref::ALL_FLAT.to_token_stream().to_string(),
        ];
        for token in tokens {
            assert!(!token.is_empty());
            assert!(
                token.contains(":: fantazia :: edo12 ::"),
                "unexpected tokens: {token}"
            );
        }
    }
}

#[cfg(all(test, feature = "serde", feature = "acci-pref"))]
mod serde_tests {
    use serde::{Deserialize, Serialize};

    fn assert_serde<T: Serialize + for<'de> Deserialize<'de>>() {}

    #[test]
    fn public_types_implement_serde() {
        assert_serde::<crate::acci::Acci>();
        assert_serde::<crate::acci_pref::AcciPref>();
        assert_serde::<crate::interval::Interval>();
        assert_serde::<crate::interval::IntervalQual>();
        assert_serde::<crate::interval::OInterval>();
        assert_serde::<crate::pitch::OPitch>();
        assert_serde::<crate::pitch::Pitch>();
        assert_serde::<crate::step::OStep>();
        assert_serde::<crate::step::Step>();
        assert_serde::<crate::tone::OPitchTone>();
        assert_serde::<crate::tone::OTone>();
        assert_serde::<crate::tone::Tone>();
    }
}

//! Compile-time literal constructors for the `fantazia` 12-EDO types.
//!
//! Every macro in this crate accepts a bare literal and a string literal, and resolves the
//! `fantazia` (or `fantazia_edo12`) crate at the call site:
//!
//! ```
//! use fantazia_edo12::pitch::OPitch;
//! use fantazia_edo12_macro::opitch;
//!
//! assert_eq!(opitch!(F+), OPitch::F_SHARP);
//! assert_eq!(opitch!("F+"), OPitch::F_SHARP);
//! ```
//!
//! The macros accept the same notation as the `FromStr` implementations of the corresponding
//! `fantazia_edo12` types:
//!
//! | Macro | Type | Examples |
//! |-------|------|----------|
//! | [`ostep!`] | `OStep` | `ostep!(C)`, `ostep!(do)`, `ostep!(7)` |
//! | [`step!`] | `Step` | `step!(C_0)`, `step!(B_-1)`, `step!(7)` |
//! | [`otone!`] | `OTone` | `otone!(C)`, `otone!(F+)`, `otone!(13)` |
//! | [`tone!`] | `Tone` | `tone!(C_1)`, `tone!(C+)`, `tone!(6)` |
//! | [`opitch!`] | `OPitch` | `opitch!(F+)`, `opitch!(B-)`, `opitch!(C[+4])` |
//! | [`pitch!`] | `Pitch` | `pitch!(C_1)`, `pitch!(F+_0)`, `pitch!(G[+3]_2)` |
//! | [`acci!`] | `Acci` | `acci!(++)`, `acci!([+4])`, `acci!(-3)` |
//! | [`ointerval!`] | `OInterval` | `ointerval!(M3)`, `ointerval!(P5)` |
//! | [`interval!`] | `Interval` | `interval!(M10)`, `interval!(M3_1)` |
//!
//! When the `fantazia` facade is used, the macros are re-exported as
//! `fantazia::edo12::opitch!` and friends; that is the recommended way to use them.
//!
//! # Crate resolution
//!
//! The macros emit paths for the crate that is available at the call site: `fantazia::edo12` when
//! the facade is a dependency, and `fantazia_edo12` otherwise. Add one of the two crates to
//! `Cargo.toml` together with this crate.

#![cfg_attr(not(test), deny(clippy::unwrap_used))]

mod emit;
mod input;
mod path;

use fantazia_edo12::{
    acci::Acci,
    interval::{Interval, OInterval},
    pitch::{OPitch, Pitch},
    step::{OStep, Step},
    tone::{OTone, Tone},
};
use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use syn::Error;

/// Creates an [`OStep`] from a compile-time literal.
///
/// The literal can be a note name, one of its solfège aliases, or an integer step value. Integer
/// values wrap modulo seven, preserving the behaviour of a general step.
///
/// ```
/// use fantazia_edo12::step::OStep;
/// use fantazia_edo12_macro::ostep;
///
/// assert_eq!(ostep!(C), OStep::C);
/// assert_eq!(ostep!("C"), OStep::C);
/// assert_eq!(ostep!(do), OStep::C);
/// assert_eq!(ostep!(sol), OStep::G);
/// assert_eq!(ostep!(7), OStep::C);
/// ```
#[proc_macro]
pub fn ostep(input: TokenStream) -> TokenStream {
    expand(input, |source| {
        let ctx = path::resolver(source.span)?;
        Ok(emit::ostep(&ctx, parse_ostep(source)?))
    })
}

/// Creates a [`Step`] from a compile-time literal.
///
/// The literal is either an octave-specific note name or an integer step value.
///
/// ```
/// use fantazia_edo12::step::Step;
/// use fantazia_edo12_macro::step;
///
/// assert_eq!(step!(C_0), Step::C_0);
/// assert_eq!(step!("C_1"), Step::C_1);
/// assert_eq!(step!(B_-1), Step::B_M1);
/// assert_eq!(step!(7), Step::C_1);
/// ```
#[proc_macro]
pub fn step(input: TokenStream) -> TokenStream {
    expand(input, |source| {
        let ctx = path::resolver(source.span)?;
        Ok(emit::step(&ctx, parse_step(source)?))
    })
}

/// Creates an [`OTone`] from a compile-time literal.
///
/// The literal can be a note name, an octave-specific pitch (which is generalized), or an integer
/// tone value modulo twelve.
///
/// ```
/// use fantazia_edo12::tone::OTone;
/// use fantazia_edo12_macro::otone;
///
/// assert_eq!(otone!(C), OTone::C);
/// assert_eq!(otone!(F+), OTone::F_SHARP);
/// assert_eq!(otone!("F+"), OTone::F_SHARP);
/// assert_eq!(otone!(C_1), OTone::C);
/// assert_eq!(otone!(13), OTone::C_SHARP);
/// ```
#[proc_macro]
pub fn otone(input: TokenStream) -> TokenStream {
    expand(input, |source| {
        let ctx = path::resolver(source.span)?;
        Ok(emit::otone(&ctx, parse_otone(source)?))
    })
}

/// Creates a [`Tone`] from a compile-time literal.
///
/// The literal can be an octave-specific pitch, an octave-unspecific pitch (whose tone is used
/// as-is), or an integer tone value.
///
/// ```
/// use fantazia_edo12::tone::Tone;
/// use fantazia_edo12_macro::tone;
///
/// assert_eq!(tone!(C_0), Tone::C_0);
/// assert_eq!(tone!("C_1"), Tone::C_1);
/// assert_eq!(tone!(C+), Tone::new(1));
/// assert_eq!(tone!(6), Tone::new(6));
/// ```
#[proc_macro]
pub fn tone(input: TokenStream) -> TokenStream {
    expand(input, |source| {
        let ctx = path::resolver(source.span)?;
        Ok(emit::tone(&ctx, parse_tone(source)?))
    })
}

/// Creates an [`OPitch`] from a compile-time literal.
///
/// The literal uses the notation of `OPitch`'s `FromStr` implementation: a note name followed by
/// an optional accidental written with `+`, `-` or a bracketed integer.
///
/// ```
/// use fantazia_edo12::pitch::OPitch;
/// use fantazia_edo12_macro::opitch;
///
/// assert_eq!(opitch!(F+), OPitch::F_SHARP);
/// assert_eq!(opitch!("F+"), OPitch::F_SHARP);
/// assert_eq!(opitch!(B-), OPitch::B_FLAT);
/// assert_eq!(opitch!(C[+4]).to_string(), "C[+4]");
/// ```
#[proc_macro]
pub fn opitch(input: TokenStream) -> TokenStream {
    expand(input, |source| {
        let ctx = path::resolver(source.span)?;
        Ok(emit::opitch(&ctx, parse_opitch(source)?))
    })
}

/// Creates a [`Pitch`] from a compile-time literal.
///
/// The literal is an octave-unspecific pitch followed by `_` and its octave number.
///
/// ```
/// use fantazia_edo12::{acci::Acci, pitch::Pitch};
/// use fantazia_edo12_macro::pitch;
///
/// assert_eq!(pitch!(C_1), Pitch::C_1);
/// assert_eq!(pitch!("F+_0"), Pitch::F_SHARP_0);
/// assert_eq!(pitch!(G[+3]_2), pitch!(G+++_2));
/// assert_eq!(pitch!(G[+3]_2).acci(), Acci::TRIPLE_SHARP);
/// ```
#[proc_macro]
pub fn pitch(input: TokenStream) -> TokenStream {
    expand(input, |source| {
        let ctx = path::resolver(source.span)?;
        Ok(emit::pitch(&ctx, parse_pitch(source)?))
    })
}

/// Creates an [`Acci`] from a compile-time literal.
///
/// The literal is a sequence of `+`, `-` and `=` characters, a bracketed integer, or a plain
/// integer accidental value.
///
/// ```
/// use fantazia_edo12::acci::Acci;
/// use fantazia_edo12_macro::acci;
///
/// assert_eq!(acci!(++), Acci::DOUBLE_SHARP);
/// assert_eq!(acci!("--"), Acci::DOUBLE_FLAT);
/// assert_eq!(acci!([+4]), Acci::new(4));
/// assert_eq!(acci!(-3), Acci::TRIPLE_FLAT);
/// assert_eq!(acci!(=), Acci::NATURAL);
/// ```
#[proc_macro]
pub fn acci(input: TokenStream) -> TokenStream {
    expand(input, |source| {
        let ctx = path::resolver(source.span)?;
        Ok(emit::acci(&ctx, parse_acci(source)?))
    })
}

/// Creates an [`OInterval`] from a compile-time literal.
///
/// The literal is an interval quality followed by an interval number between 1 and 7, e.g. `M3`,
/// `P5` or `d5`.
///
/// ```
/// use fantazia_edo12::interval::OInterval;
/// use fantazia_edo12_macro::ointerval;
///
/// assert_eq!(ointerval!(M3), OInterval::MAJOR_THIRD);
/// assert_eq!(ointerval!("P5"), OInterval::PERFECT_FIFTH);
/// assert_eq!(ointerval!(d5), OInterval::DIMINISHED_FIFTH);
/// assert_eq!(ointerval!(M3).to_string(), "M3");
/// ```
#[proc_macro]
pub fn ointerval(input: TokenStream) -> TokenStream {
    expand(input, |source| {
        let ctx = path::resolver(source.span)?;
        Ok(emit::ointerval(&ctx, parse_ointerval(source)?))
    })
}

/// Creates an [`Interval`] from a compile-time literal.
///
/// The literal is either a compound interval number such as `M10`, or an octave-unspecific
/// interval followed by `_` and an octave number such as `M3_1`. A leading `+` or `-` sets the
/// direction.
///
/// ```
/// use fantazia_edo12::interval::Interval;
/// use fantazia_edo12_macro::interval;
///
/// assert_eq!(interval!(M3).to_string(), "M3");
/// assert_eq!(interval!(M10), interval!(M3_1));
/// assert_eq!(interval!("M10"), interval!(M10));
/// assert_eq!(interval!(-M3).to_string(), "-M3");
/// ```
#[proc_macro]
pub fn interval(input: TokenStream) -> TokenStream {
    expand(input, |source| {
        let ctx = path::resolver(source.span)?;
        Ok(emit::interval(&ctx, parse_interval(source)?))
    })
}

fn expand(
    tokens: TokenStream,
    f: impl FnOnce(&input::Source) -> syn::Result<TokenStream2>,
) -> TokenStream {
    match input::parse(tokens.into()).and_then(|source| f(&source)) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

fn parse_or_error<T>(value: Result<T, impl std::fmt::Display>, span: Span) -> syn::Result<T> {
    value.map_err(|err| Error::new(span, err))
}

fn parse_ostep(source: &input::Source) -> syn::Result<OStep> {
    if let Some(value) = source.as_i8() {
        return OStep::new(value.rem_euclid(7))
            .ok_or_else(|| Error::new(source.span, "internal error: step index out of range"));
    }

    match source.text.parse::<OStep>() {
        Ok(step) => Ok(step),
        Err(err) => match source.text.parse::<Step>() {
            Ok(step) => OStep::new(step.as_i8().rem_euclid(7))
                .ok_or_else(|| Error::new(source.span, "internal error: step index out of range")),
            Err(_) => Err(Error::new(source.span, err)),
        },
    }
}

fn parse_step(source: &input::Source) -> syn::Result<Step> {
    if let Some(value) = source.as_i8() {
        return Ok(Step::new(value));
    }
    parse_or_error(source.text.parse::<Step>(), source.span)
}

fn parse_otone(source: &input::Source) -> syn::Result<OTone> {
    let value = parse_tone_value(source)?;
    OTone::new(value.rem_euclid(12))
        .ok_or_else(|| Error::new(source.span, "internal error: tone index out of range"))
}

fn parse_tone(source: &input::Source) -> syn::Result<Tone> {
    Ok(Tone::new(parse_tone_value(source)?))
}

fn parse_tone_value(source: &input::Source) -> syn::Result<i8> {
    if let Some(value) = source.as_i8() {
        return Ok(value);
    }

    match source.text.parse::<Pitch>() {
        Ok(pitch) => Ok(pitch.tone.as_i8()),
        Err(err) => match source.text.parse::<OPitch>() {
            Ok(pitch) => Ok(pitch.tone.as_i8()),
            Err(_) => Err(Error::new(source.span, err)),
        },
    }
}

fn parse_opitch(source: &input::Source) -> syn::Result<OPitch> {
    parse_or_error(source.text.parse::<OPitch>(), source.span)
}

fn parse_pitch(source: &input::Source) -> syn::Result<Pitch> {
    parse_or_error(source.text.parse::<Pitch>(), source.span)
}

fn parse_acci(source: &input::Source) -> syn::Result<Acci> {
    if let Some(value) = source.as_i8() {
        return Ok(Acci::new(value));
    }
    parse_or_error(source.text.parse::<Acci>(), source.span)
}

fn parse_ointerval(source: &input::Source) -> syn::Result<OInterval> {
    parse_or_error(source.text.parse::<OInterval>(), source.span)
}

fn parse_interval(source: &input::Source) -> syn::Result<Interval> {
    parse_or_error(source.text.parse::<Interval>(), source.span)
}

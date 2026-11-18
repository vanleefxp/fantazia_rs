//! Token emission for parsed literal values.
//!
//! The emitted expressions only use public constructors of `fantazia_edo12`, so they work in both
//! the `fantazia` facade and a direct dependency. Intervals are validated by the parser before
//! this module is reached, which is why the fallback constructors can be called without a runtime
//! check.

use fantazia_edo12::{
    acci::Acci,
    interval::{Interval, IntervalQual, OInterval},
    pitch::{OPitch, Pitch},
    step::{OStep, Step},
    tone::{OTone, Tone},
};
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;

const STEP_NAMES: [&str; 7] = ["C", "D", "E", "F", "G", "A", "B"];
const OTONE_NAMES: [&str; 12] = [
    "C", "C_SHARP", "D", "D_SHARP", "E", "F", "F_SHARP", "G", "G_SHARP", "A", "A_SHARP", "B",
];

fn ident(name: &str) -> Ident {
    Ident::new(name, Span::call_site())
}

fn step_ident(step: OStep) -> Ident {
    ident(STEP_NAMES[step as i8 as usize])
}

pub(crate) fn ostep(ctx: &TokenStream, step: OStep) -> TokenStream {
    let step = step_ident(step);
    quote!(#ctx::step::OStep::#step)
}

pub(crate) fn step(ctx: &TokenStream, step: Step) -> TokenStream {
    let value = step.as_i8();
    quote!(#ctx::step::Step::new(#value))
}

pub(crate) fn otone(ctx: &TokenStream, tone: OTone) -> TokenStream {
    let tone = ident(OTONE_NAMES[tone as i8 as usize]);
    quote!(#ctx::tone::OTone::#tone)
}

pub(crate) fn tone(ctx: &TokenStream, tone: Tone) -> TokenStream {
    let value = tone.as_i8();
    quote!(#ctx::tone::Tone::new(#value))
}

pub(crate) fn opitch(ctx: &TokenStream, pitch: OPitch) -> TokenStream {
    let step = step_ident(pitch.step);
    let tone = pitch.tone.as_i8();
    quote!(#ctx::pitch::OPitch::new(#ctx::step::OStep::#step, #tone))
}

pub(crate) fn pitch(ctx: &TokenStream, pitch: Pitch) -> TokenStream {
    let step = pitch.step.as_i8();
    let tone = pitch.tone.as_i8();
    quote! {
        #ctx::pitch::Pitch::new(#ctx::step::Step::new(#step), #ctx::tone::Tone::new(#tone))
    }
}

pub(crate) fn acci(ctx: &TokenStream, acci: Acci) -> TokenStream {
    let value = acci.as_i8();
    quote!(#ctx::acci::Acci::new(#value))
}

pub(crate) fn ointerval(ctx: &TokenStream, interval: OInterval) -> TokenStream {
    let (step, qual) = (interval.step(), interval.qual());

    // Prefer the named constants, both for readability and to avoid `unsafe` in the common case.
    if let Some(name) = ointerval_const(step, qual) {
        let name = ident(name);
        return quote!(#ctx::interval::OInterval::#name);
    }

    let step = step_ident(step);
    let qual = interval_qual(ctx, qual);
    quote! {
        unsafe { #ctx::interval::OInterval::new_unchecked(#ctx::step::OStep::#step, #qual) }
    }
}

pub(crate) fn interval(ctx: &TokenStream, interval: Interval) -> TokenStream {
    let step = interval.step().as_i8();
    let qual = interval_qual(ctx, interval.qual());
    quote! {
        unsafe { #ctx::interval::Interval::new_unchecked(#ctx::step::Step::new(#step), #qual) }
    }
}

fn interval_qual(ctx: &TokenStream, qual: IntervalQual) -> TokenStream {
    match qual.as_i8() {
        3 => quote!(#ctx::interval::IntervalQual::DOUBLY_AUGMENTED),
        2 => quote!(#ctx::interval::IntervalQual::AUGMENTED),
        1 => quote!(#ctx::interval::IntervalQual::MAJOR),
        0 => quote!(#ctx::interval::IntervalQual::PERFECT),
        -1 => quote!(#ctx::interval::IntervalQual::MINOR),
        -2 => quote!(#ctx::interval::IntervalQual::DIMINISHED),
        -3 => quote!(#ctx::interval::IntervalQual::DOUBLY_DIMINISHED),
        n if n > 0 => {
            let n = (n - 1) as u8;
            quote!(#ctx::interval::IntervalQual::augmented(#n))
        }
        n => {
            let n = (-n - 1) as u8;
            quote!(#ctx::interval::IntervalQual::diminished(#n))
        }
    }
}

/// Returns the named `OInterval` constant for the usual qualities, if there is one.
fn ointerval_const(step: OStep, qual: IntervalQual) -> Option<&'static str> {
    use OStep::*;
    Some(match (step, qual.as_i8()) {
        (C, -2) => "DIMINISHED_UNISON",
        (C, 0) => "PERFECT_UNISON",
        (C, 2) => "AUGMENTED_UNISON",
        (D, -2) => "DIMINISHED_SECOND",
        (D, -1) => "MINOR_SECOND",
        (D, 1) => "MAJOR_SECOND",
        (D, 2) => "AUGMENTED_SECOND",
        (E, -2) => "DIMINISHED_THIRD",
        (E, -1) => "MINOR_THIRD",
        (E, 1) => "MAJOR_THIRD",
        (E, 2) => "AUGMENTED_THIRD",
        (F, -2) => "DIMINISHED_FOURTH",
        (F, 0) => "PERFECT_FOURTH",
        (F, 2) => "AUGMENTED_FOURTH",
        (G, -2) => "DIMINISHED_FIFTH",
        (G, 0) => "PERFECT_FIFTH",
        (G, 2) => "AUGMENTED_FIFTH",
        (A, -2) => "DIMINISHED_SIXTH",
        (A, -1) => "MINOR_SIXTH",
        (A, 1) => "MAJOR_SIXTH",
        (A, 2) => "AUGMENTED_SIXTH",
        (B, -2) => "DIMINISHED_SEVENTH",
        (B, -1) => "MINOR_SEVENTH",
        (B, 1) => "MAJOR_SEVENTH",
        (B, 2) => "AUGMENTED_SEVENTH",
        _ => return None,
    })
}

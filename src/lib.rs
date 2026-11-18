//! # `fantazia`: semantic music computation package
//!
//! `fantazia` re-exports the workspace crates behind feature flags. With the default `edo12` and
//! `macro` features, the [`edo12`] module provides both the 12-EDO types and the compile-time
//! literal macros `opitch!`, `pitch!`, `ostep!`, `step!` and friends.

// The literal macros expand to `::fantazia::edo12` paths, which must also resolve inside this
// crate (unit tests and doctests).
extern crate self as fantazia;

pub use fantazia_core::*;

/// The standard 12-EDO model, re-exported from `fantazia_edo12`.
///
/// With the `macro` feature (enabled by default), the module also re-exports the literal
/// construction macros from `fantazia_edo12_macro`:
///
/// - `opitch!` / `pitch!` for named pitches,
/// - `ostep!` / `step!` for note steps,
/// - `otone!` / `tone!` for tones,
/// - `acci!` for accidentals,
/// - `ointerval!` / `interval!` for intervals.
///
/// Each macro accepts a bare literal and a string literal.
#[cfg(feature = "edo12")]
pub mod edo12 {
    pub use fantazia_edo12::*;

    #[cfg(feature = "macro")]
    pub use fantazia_edo12_macro::*;
}

#[cfg(all(test, feature = "macro", feature = "edo12"))]
mod literal_macro_tests {
    use crate::edo12::{
        opitch, pitch,
        pitch::{OPitch, Pitch},
        step,
        step::Step,
    };

    #[test]
    fn expands_inside_the_defining_crate() {
        assert_eq!(opitch!(F+), OPitch::F_SHARP);
        assert_eq!(opitch!("F+"), OPitch::F_SHARP);
        assert_eq!(step!(C_1), Step::C_1);
        assert_eq!(step!("B_-1"), Step::B_M1);
        assert_eq!(pitch!(C_1), Pitch::C_1);
    }
}

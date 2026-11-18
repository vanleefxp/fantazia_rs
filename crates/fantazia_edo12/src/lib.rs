//! [`fantazia_edo12`] provides basic pitch arithmetics support for the standard **12-TET** / **12-EDO**.

#![allow(internal_features)]
#![warn(missing_docs)]
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
    pattern_type_macro,
    pattern_type_range_trait,
    pattern_types,
    impl_restriction,
    formatting_options
)]

#[cfg(feature = "acci")]
pub mod acci;
#[cfg(feature = "acci-pref")]
pub mod acci_pref;
#[cfg(feature = "interval")]
pub mod interval;
#[cfg(feature = "pitch")]
pub mod pitch;
#[cfg(feature = "step")]
pub mod step;
#[cfg(feature = "tone")]
pub mod tone;

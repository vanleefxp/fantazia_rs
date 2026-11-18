#![feature(
    const_trait_impl,
    const_ops,
    const_clone,
    const_default,
    const_cmp,
    const_bool,
    const_closures,
    const_destruct,
    const_result_trait_fn,
    derive_const,
    const_convert,
    mapped_lock_guards
    // pattern_type_macro,
    // pattern_types,
    // pattern_type_range_trait,
    // generic_const_exprs,
    // generic_pattern_types,
    // const_range,
    // const_range_bounds
)]
#![warn(clippy::unnecessary_cast)]
// #![allow(internal_features, incomplete_features)]
pub mod pitch;
pub mod step;
pub mod temprament;
pub mod tone;

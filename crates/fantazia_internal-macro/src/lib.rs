//! Internal proc-macros for code generation used by `fantazia`.
//! This is not public API.

#![feature(titlecase, iter_intersperse)]

#[cfg(feature = "edo12")]
mod edo12;

use proc_macro as pm;

#[cfg(feature = "edo12")]
#[proc_macro]
pub fn gen_step_name_to_step_table(_input: pm::TokenStream) -> pm::TokenStream {
    edo12::gen_step_name_to_step_table().into()
}

#[cfg(feature = "edo12")]
#[proc_macro]
pub fn gen_ostep_consts(_input: pm::TokenStream) -> pm::TokenStream {
    edo12::gen_ostep_consts().into()
}

#[cfg(feature = "edo12")]
#[proc_macro]
pub fn gen_step_consts(_input: pm::TokenStream) -> pm::TokenStream {
    edo12::gen_step_consts(-2..2).into()
}

/// Programmatically generate associated `OPitch` constants.
#[cfg(feature = "edo12")]
#[proc_macro]
pub fn gen_opitch_consts(_input: pm::TokenStream) -> pm::TokenStream {
    edo12::gen_opitch_consts().into()
}

/// Programmatically generate associated `Pitch` constants.
#[cfg(feature = "edo12")]
#[proc_macro]
pub fn gen_pitch_consts(_input: pm::TokenStream) -> pm::TokenStream {
    edo12::gen_pitch_consts(-2i8..=2).into()
}

#[cfg(feature = "edo12")]
#[proc_macro]
pub fn gen_otone_consts(_input: pm::TokenStream) -> pm::TokenStream {
    edo12::gen_otone_consts().into()
}

#[cfg(feature = "edo12")]
#[proc_macro]
pub fn gen_tone_consts(_input: pm::TokenStream) -> pm::TokenStream {
    edo12::gen_tone_consts(-2i8..=2).into()
}

#[cfg(feature = "edo12")]
#[proc_macro]
pub fn gen_ointerval_consts(_input: pm::TokenStream) -> pm::TokenStream {
    edo12::gen_ointerval_consts().into()
}

#[cfg(feature = "edo12")]
#[proc_macro]
pub fn gen_black_key_struct(_input: pm::TokenStream) -> pm::TokenStream {
    edo12::gen_black_key_struct().into()
}

#[cfg(feature = "edo12")]
#[proc_macro]
pub fn gen_acci_pref_methods(_input: pm::TokenStream) -> pm::TokenStream {
    edo12::gen_acci_pref_methods().into()
}

#[cfg(feature = "edo12")]
#[proc_macro]
pub fn gen_acci_ops_trait(_input: pm::TokenStream) -> pm::TokenStream {
    edo12::gen_acci_ops_trait().into()
}

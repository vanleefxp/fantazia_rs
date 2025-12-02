use proc_macro::TokenStream;
use syn::{parse::{Parse, ParseStream}, parse_macro_input};
use quote::ToTokens;
use paste::paste;

use fantazia_lib::edo12::{OPitch, Pitch, OInterval, Interval};

macro_rules! make_parse_proc_macro {
    ($($t:ty, $fn_name:ident);*$(;)?) => {
        paste! {
            $(
                struct [< $t MacroInput >]([< $t >]);
                #[proc_macro]
                pub fn $fn_name(ts: TokenStream) -> TokenStream {
                    let [< $t MacroInput >](value) = parse_macro_input!(ts as [< $t MacroInput >]);
                    value.to_token_stream().into()
                }
            )*
        }
    };
}


impl Parse for OPitchMacroInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let src = input.parse::<syn::LitStr>()?.value();
        if let Ok(pitch) = src.parse::<OPitch>() {
            Ok(OPitchMacroInput(pitch))
        } else if let Ok(interval) = src.parse::<OInterval>() {
            Ok(OPitchMacroInput(interval.into()))
        } else {
            Err(syn::Error::new(input.span(), "Invalid input"))
        }
    }
}

impl Parse for PitchMacroInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let src = input.parse::<syn::LitStr>()?.value();
        if let Ok(pitch) = src.parse::<Pitch>() {
            Ok(PitchMacroInput(pitch))
        } else if let Ok(interval) = src.parse::<Interval>() {
            Ok(PitchMacroInput(interval.into()))
        } else {
            Err(syn::Error::new(input.span(), "Invalid input"))
        }
    }
}

// impl Parse for SimpleIntervalMacroInput {
//     fn parse(input: ParseStream) -> syn::Result<Self> {
//         let src = input.parse::<syn::LitStr>()?.value();
//         if let Ok(interval) = src.parse::<SimpleInterval>() {
//             Ok(SimpleIntervalMacroInput(interval))
//         } else if let Ok(pitch) = src.parse::<OPitch>() {
//             Ok(SimpleIntervalMacroInput(pitch.into()))
//         } else {
//             Err(syn::Error::new(input.span(), "Invalid input"))
//         }
//     }
// }

// impl Parse for IntervalMacroInput {
//     fn parse(input: ParseStream) -> syn::Result<Self> {
//         let src = input.parse::<syn::LitStr>()?.value();
//         if let Ok(interval) = src.parse::<Interval>() {
//             Ok(IntervalMacroInput(interval))
//         } else if let Ok(pitch) = src.parse::<Pitch>() {
//             Ok(IntervalMacroInput(pitch.into()))
//         } else {
//             Err(syn::Error::new(input.span(), "Invalid input"))
//         }
//     }
// }

make_parse_proc_macro!(
    OPitch, opitch;
    Pitch, pitch;
    /*SimpleInterval, Interval*/
);

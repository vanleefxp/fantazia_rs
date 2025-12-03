use proc_macro::TokenStream;
use syn::{parse::{Parse, ParseStream}, parse_macro_input};
use quote::ToTokens;
use paste::paste;

use fantazia_lib::pitch::edo12::{OPitch, Pitch, OInterval, Interval, Acci, OStep, Step
};

macro_rules! make_parse_proc_macro_helper {
    ($t:ty, $fn_name:ident$(,)?) => {
        paste! {
            struct [< $t MacroInput >]([< $t >]);
            #[proc_macro]
            pub fn $fn_name(ts: TokenStream) -> TokenStream {
                let [< $t MacroInput >](value) = parse_macro_input!(ts as [< $t MacroInput >]);
                value.to_token_stream().into()
            }
        }
    };
    ($t:ty$(,)?) => {
        paste! {
            make_parse_proc_macro_helper!($t, [< $t:lower >]);
        }
    }
}

macro_rules! make_parse_proc_macro {
    ($($args:tt);*$(;)?) => {
        $(
            make_parse_proc_macro_helper!($args);
        )*
    };
}

fn get_src_from_parse_stream(input: ParseStream) -> syn::Result<String> {
    let src = if input.peek(syn::Ident) {
        input.parse::<syn::Ident>()?.to_string()
    } else {
        input.parse::<syn::LitStr>()?.value()
    };
    Ok(src)
}

impl Parse for OPitchMacroInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let src = get_src_from_parse_stream(input)?;
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
        let src = get_src_from_parse_stream(input)?;
        if let Ok(pitch) = src.parse::<Pitch>() {
            Ok(PitchMacroInput(pitch))
        } else if let Ok(interval) = src.parse::<Interval>() {
            Ok(PitchMacroInput(interval.into()))
        } else {
            Err(syn::Error::new(input.span(), "Invalid input"))
        }
    }
}

impl Parse for OStepMacroInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.peek(syn::LitInt) {
            let token = input.parse::<syn::LitInt>()?;
            let value = token.base10_parse::<u8>()?;
            let value: OStep = value.try_into().map_err(|err| syn::Error::new(input.span(), err))?;
            Ok(OStepMacroInput(value))
        } else {
            let src = get_src_from_parse_stream(input)?;
            let value: OStep = src.parse().map_err(|err| syn::Error::new(input.span(), err))?;
            Ok(OStepMacroInput(value))
        }
    }
}

impl Parse for StepMacroInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.peek(syn::LitInt) {
            let token = input.parse::<syn::LitInt>()?;
            let value = token.base10_parse::<i8>()?;
            let value: Step = value.into();
            Ok(StepMacroInput(value))
        } else {
            let src = get_src_from_parse_stream(input)?;
            let value: Step = src.parse().map_err(|err| syn::Error::new(input.span(), err))?;
            Ok(StepMacroInput(value))
        }
    }
}

impl Parse for AcciMacroInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.is_empty() {
            Ok(AcciMacroInput(Acci::NATURAL))
        } else if input.peek(syn::LitInt) {
            if let Ok(token) = input.parse::<syn::LitInt>() {
                let value: Acci = token.base10_parse::<i8>()?.into();
                Ok(AcciMacroInput(value))
            } else {
                unreachable!()
            }
        } else if let Ok(token) = input.parse::<syn::LitStr>() {
            let value: Acci = token.value().parse().map_err(|err| syn::Error::new(input.span(), err))?;
            Ok(AcciMacroInput(value))
        } else {
            Err(syn::Error::new(input.span(), "Invalid input"))
        }
    }
}

impl Parse for OIntervalMacroInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let src = get_src_from_parse_stream(input)?;
        if let Ok(interval) = src.parse::<OInterval>() {
            Ok(OIntervalMacroInput(interval))
        } else if let Ok(pitch) = src.parse::<OPitch>() {
            Ok(OIntervalMacroInput(pitch.into()))
        } else {
            Err(syn::Error::new(input.span(), "Invalid input"))
        }
    }
}

impl Parse for IntervalMacroInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let src = get_src_from_parse_stream(input)?;
        if let Ok(interval) = src.parse::<Interval>() {
            Ok(IntervalMacroInput(interval))
        } else if let Ok(pitch) = src.parse::<Pitch>() {
            Ok(IntervalMacroInput(pitch.into()))
        } else {
            Err(syn::Error::new(input.span(), "Invalid input"))
        }
    }
}

// [TODO] more detailed error messages for parsing errors

make_parse_proc_macro!(OPitch; Pitch; OStep; Step; Acci; OInterval; Interval);

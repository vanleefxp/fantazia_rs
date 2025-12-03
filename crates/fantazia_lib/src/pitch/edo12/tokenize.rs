use malachite_base::strings::ToDebugString;
use proc_macro2::{TokenStream};
use quote::{ToTokens, quote};

use super::{OStep, Step, OPitch, Pitch, OIntervalDeg, IntervalQual, IntervalDeg, Acci, Interval, OInterval};

macro_rules! derive_to_tokens_for_newtype {
    ($t:ty, $mod_path:path) => {
        impl ToTokens for $t {
            fn to_tokens(&self, tokens: &mut TokenStream) {
                let num = self.0.to_token_stream();
                tokens.extend(quote!(
                    $mod_path::$t::from(#num)
                ));
            }
        }
    };
}

macro_rules! derive_to_tokens_for_enum {
    ($t:ty, $mod_path:path) => {
        impl ToTokens for $t {
            fn to_tokens(&self, tokens: &mut TokenStream) {
                let ts: TokenStream = self.to_debug_string().parse().unwrap();
                tokens.extend(quote!(
                    $mod_path::$t::#ts
                ));
            }
        }
    };
}

derive_to_tokens_for_newtype!(Step, fantazia_lib::pitch::edo12);
derive_to_tokens_for_newtype!(Acci, fantazia_lib::pitch::edo12);
derive_to_tokens_for_newtype!(IntervalDeg, fantazia_lib::pitch::edo12);
derive_to_tokens_for_enum!(OStep, fantazia_lib::pitch::edo12);
derive_to_tokens_for_enum!(OIntervalDeg, fantazia_lib::pitch::edo12);
derive_to_tokens_for_enum!(IntervalQual, fantazia_lib::pitch::edo12);

impl ToTokens for OPitch {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let step = self.step.to_token_stream();
        let tone = self.tone.to_token_stream();
        tokens.extend(quote!(
            fantazia_lib::pitch::edo12::OPitch::from_step_and_tone(#step, #tone)
        ));
    }
}

impl ToTokens for Pitch {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let step = self.step.to_token_stream();
        let tone = self.tone.to_token_stream();
        tokens.extend(quote!(
            fantazia_lib::pitch::edo12::Pitch::from_step_and_tone(#step, #tone)
        ));
    }
}

impl ToTokens for OInterval {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let deg = self.deg.to_token_stream();
        let qual = self.qual.to_token_stream();
        tokens.extend(quote!(
            unsafe { fantazia_lib::pitch::edo12::OInterval::from_deg_and_qual_unchecked(#deg, #qual) }
        ));
    }
}

impl ToTokens for Interval {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let deg = self.deg.to_token_stream();
        let qual = self.qual.to_token_stream();
        tokens.extend(quote!(
            unsafe { fantazia_lib::pitch::edo12::Interval::from_deg_and_qual_unchecked(#deg, #qual) }
        ));
    }
}
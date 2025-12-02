use malachite_base::strings::ToDebugString;
use proc_macro2::{Span, TokenStream};
use quote::{ToTokens, quote};

use crate::pitch::edo12::Acci;

use super::{OStep, Step, OPitch, Pitch};

impl ToTokens for OStep {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let ident = syn::Ident::new(&self.to_debug_string(), Span::call_site());
        tokens.extend(quote!(
            fantazia_lib::pitch::edo12::OStep::#ident
        ));
    }
}

impl ToTokens for Step {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let num = self.0.to_token_stream();
        tokens.extend(quote!(
            fantazia_lib::pitch::edo12::Step::from(#num)
        ));
    }
}

impl ToTokens for Acci {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let num = self.0.to_token_stream();
        tokens.extend(quote!(
            fantazia_lib::pitch::edo12::Acci::from(#num)
        ));
    }
}

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
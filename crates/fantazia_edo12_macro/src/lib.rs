// #![allow(non_snake_case)]

// use std::fmt::{self, Display};

// use fantazia_edo12::step::Step;
// use proc_macro as pm;
// use proc_macro2 as pm2;
// use quote::{ToTokens, TokenStreamExt, quote};
// use syn::{
//     Token,
//     parse::{Parse, ParseStream},
// };

// use crate::StepLiteral::Symbolic;

// macro_rules! path_of {
//     (OStep) => {
//         quote!(::fantazia::edo12::step::OStep)
//     };
//     (Step) => {
//         quote!(::fantazia::edo12::step::Step)
//     };
//     (GeneralPitch) => {
//         quote!(::fantazia::GeneralPitch)
//     };
// }

// // MARK: Sign

// #[derive(Clone, Copy)]
// enum Sign {
//     Plus(Token![+]),
//     Minus(Token![-]),
// }

// impl Parse for Sign {
//     fn parse(input: ParseStream) -> syn::Result<Self> {
//         if let Ok(plus) = input.parse::<Token![+]>() {
//             Ok(Self::Plus(plus))
//         } else if let Ok(minus) = input.parse::<Token![-]>() {
//             Ok(Self::Minus(minus))
//         } else {
//             Err(input.error("expect `+` or `-`"))
//         }
//     }
// }

// impl ToTokens for Sign {
//     fn to_tokens(&self, tokens: &mut pm2::TokenStream) {
//         match *self {
//             Self::Plus(plus) => tokens.extend(quote!(+)),
//             Self::Minus(minus) => tokens.extend(quote!(-)),
//         }
//     }
// }

// // MARK: SignedLitInt

// struct SignedLitInt {
//     pub sign: Option<Sign>,
//     pub value: syn::LitInt,
// }

// impl Parse for SignedLitInt {
//     fn parse(input: ParseStream) -> syn::Result<Self> {
//         let sign = input.parse().ok();
//         let value = input.parse()?;
//         Ok(Self { sign, value })
//     }
// }

// impl From<syn::LitInt> for SignedLitInt {
//     fn from(value: syn::LitInt) -> Self {
//         Self { sign: None, value }
//     }
// }

// impl ToTokens for SignedLitInt {
//     fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
//         let Self { sign, value } = self;
//         tokens.extend(quote! {
//             #sign #value
//         });
//     }
// }

// // MARK: OStepLiteral

// enum OStepLiteral {
//     Symbolic(syn::Ident),
//     LitStr(syn::LitStr),
// }

// impl Parse for OStepLiteral {
//     fn parse(input: ParseStream) -> syn::Result<Self> {
//         if let Ok(lit_str) = input.parse::<syn::LitStr>() {
//             Ok(Self::LitStr(lit_str))
//         } else if let Ok(ident) = input.parse::<syn::Ident>() {
//             Ok(Self::Symbolic(ident))
//         } else {
//             Err(input.error("expect string literal or identifier"))
//         }
//     }
// }

// impl ToTokens for OStepLiteral {
//     fn to_tokens(&self, tokens: &mut pm2::TokenStream) {
//         let (span, mut name) = match self {
//             Self::Symbolic(ident) => (ident.span(), ident.to_string()),
//             Self::LitStr(lit_str) => (lit_str.span(), lit_str.value()),
//         };
//         name.make_ascii_uppercase();
//         let name = syn::Ident::new(&name, span);
//         tokens.extend(quote! {
//             ::fantazia::edo12::step::OStep::#name
//         });
//     }
// }

// // MARK: StepLiteral

// enum StepLiteral {
//     Symbolic {
//         name: syn::Ident,
//         octave_num: SignedLitInt,
//     },
//     LitStr(syn::LitStr),
// }

// impl Parse for StepLiteral {
//     fn parse(input: ParseStream) -> syn::Result<Self> {
//         if let Ok(lit_str) = input.parse::<syn::LitStr>() {
//             Ok(Self::LitStr(lit_str))
//         } else {
//             let src = input.parse::<syn::Ident>()?.to_string();
//             let (name, octave_src) = src
//                 .split_once('_')
//                 .ok_or_else(|| input.error("missing octave"))?;
//             let name = syn::Ident::new(name, input.span());
//             let octave_num = if octave_src.is_empty() {
//                 syn::LitInt::new(octave_src, input.span()).into()
//             } else {
//                 input.parse::<SignedLitInt>()?
//             };
//             if !octave_num.value.suffix().is_empty() {
//                 return Err(input.error("suffix is not supported"));
//             }
//             Ok(Self::Symbolic { name, octave_num })
//         }
//     }
// }

// impl ToTokens for StepLiteral {
//     fn to_tokens(&self, tokens: &mut pm2::TokenStream) {
//         match self {
//             Self::Symbolic { name, octave_num } => {
//                 let _OStep = path_of!(OStep);
//                 let _Step = path_of!(Step);
//                 let _GeneralPitch = path_of!(GeneralPitch);
//                 let ostep = OStepLiteral::Symbolic(name.clone());
//                 tokens.extend(quote! {
//                     <#_OStep as #_GeneralPitch<#_Step>>::put_in_octave(#ostep, #octave_num)
//                 });
//             }
//             Self::LitStr(lit_str) => {
//                 // let Ok(value) = lit_str.value().parse::<Step>() else {

//                 // };
//                 match lit_str.value().parse::<Step>() {
//                     Ok(value) => {
//                         tokens.extend(quote!(#value));
//                     }
//                     Err(err) => {
//                         tokens.err
//                     }
//                 }
//             }
//         }
//     }
// }

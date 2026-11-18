#![feature(
    const_trait_impl,
    const_cmp,
    const_clone,
    const_convert,
    const_try,
    derive_const
)]
#![allow(internal_features)]
#![cfg_attr(not(test), deny(clippy::unwrap_used))]

mod expand;
mod repr;
mod utils;

use proc_macro::{self as pm};
use proc_macro2 as pm2;
use quote::{ToTokens, format_ident, quote};
use syn::{
    Attribute, Data, DeriveInput, Ident, LitInt, Token, Visibility, braced, parenthesized,
    parse::{Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
    token::{Brace, Paren},
};

use crate::{
    expand::{expand_new_unchecked_impl, expand_ops_impls},
    repr::*,
    utils::{cmp_path, fmt_path, hash_path, mem_path, pat_path},
};

#[derive(Clone)]
struct NamedVariant {
    attrs: Vec<Attribute>,
    name: Ident,
}

impl Parse for NamedVariant {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let attrs = Attribute::parse_outer(input)?;
        let name = input.parse()?;
        Ok(Self { attrs, name })
    }
}

impl ToTokens for NamedVariant {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        for attr in &self.attrs {
            tokens.extend(attr.to_token_stream());
        }
        tokens.extend(self.name.to_token_stream());
    }
}

#[derive(Clone)]
struct ModulusEnum {
    _enum: Token![enum],
    _brace: Brace,
    variants: Punctuated<NamedVariant, Token![,]>,
}

#[derive(Clone)]
struct ModulusStruct {
    _struct: Token![struct],
    _paren: Paren,
    divisor: LitInt,
    _semi: Token![;],
}

#[derive(Clone)]
enum ModulusTypeInner {
    Enum(ModulusEnum),
    Struct(ModulusStruct),
}

impl ModulusTypeInner {
    fn divisor(&self) -> u128 {
        match self {
            Self::Enum(enum_decl) => enum_decl.variants.len() as u128,
            // SAFETY: parsing should succeed as validity is checked before
            Self::Struct(struct_decl) => unsafe {
                struct_decl.divisor.base10_parse().unwrap_unchecked()
            },
        }
    }
}

#[derive(Clone)]
struct ModulusType {
    attrs: Vec<Attribute>,
    vis: Visibility,
    type_name: Ident,
    inner: ModulusTypeInner,
}

struct ModulusTypeDecls(Vec<ModulusType>);

impl Parse for ModulusType {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let attrs = Attribute::parse_outer(input)?;
        let vis = input.parse()?;
        let (type_name, inner) = if let Ok(_enum) = input.parse::<Token![enum]>() {
            let type_name = input.parse()?;
            let content;
            let _brace = braced!(content in input);
            let variants = Punctuated::parse_terminated(&content)?;
            if variants.is_empty() {
                return Err(content.error("requires at least 1 variant"));
            }
            if !content.is_empty() {
                return Err(content.error("unexpected tokens"));
            }
            let enum_decl = ModulusEnum {
                _enum,
                _brace,
                variants,
            };
            (type_name, ModulusTypeInner::Enum(enum_decl))
        } else if let Ok(_struct) = input.parse::<Token![struct]>() {
            let type_name = input.parse()?;
            let content;
            let _paren = parenthesized!(content in input);
            let divisor = content.parse::<LitInt>()?;
            if !divisor.suffix().is_empty() {
                return Err(content.error("suffix is not supported"));
            }
            if divisor.base10_parse::<u128>().is_err() {
                return Err(content.error("invalid divisor"));
            }
            if !content.is_empty() {
                return Err(content.error("unexpected tokens"));
            }
            let _semi = input.parse()?;
            let struct_decl = ModulusStruct {
                _struct,
                _paren,
                divisor,
                _semi,
            };
            (type_name, ModulusTypeInner::Struct(struct_decl))
        } else {
            return Err(input.error("expect `enum` or `struct`"));
        };
        Ok(Self {
            attrs,
            vis,
            type_name,
            inner,
        })
    }
}

impl Parse for ModulusTypeDecls {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut decls = Vec::new();
        while !input.is_empty() {
            decls.push(input.parse()?);
        }
        Ok(Self(decls))
    }
}

impl ToTokens for ModulusType {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let mem = mem_path();

        let Self {
            attrs,
            vis,
            type_name,
            inner,
        } = self;

        // other attributes
        tokens.extend(attrs.iter().map(ToTokens::to_token_stream));
        let common_attrs = quote! {
            #[derive(Copy, Hash)]
            #[derive_const(Clone, Default, PartialEq, Eq, PartialOrd, Ord)]
        };

        let divisor = Divisor::new(inner.divisor());
        let repr_type = divisor.repr_type();
        match inner {
            ModulusTypeInner::Enum(ModulusEnum { variants, .. }) => {
                let has_default = variants.iter().any(|v| {
                    v.attrs
                        .iter()
                        .any(|v| v.meta.path().get_ident().is_some_and(|v| v == "default"))
                });
                let first_default_attr = (!has_default).then(|| quote!(#[default]));
                let repr_attr = ReprType::try_from(repr_type)
                    .ok()
                    .map(|repr_type| quote!(#[repr(#repr_type)]));
                tokens.extend(quote! {
                    #common_attrs
                    #repr_attr
                    #vis enum #type_name {
                        // if no default is specified, then uses the first value as default
                        #first_default_attr
                        #variants
                    }
                });

                if let Ok(divisor) = NonBoundaryDivisor::try_from(divisor) {
                    expand_new_unchecked_impl(tokens, type_name, divisor);
                } else if repr_type != ReprTypeOrZst::Zst {
                    tokens.extend(quote! {
                        const impl #type_name {
                            /// Creates a new value directly from its raw representation.
                            #[inline(always)]
                            pub fn new(value: #repr_type) -> Self {
                                unsafe { #mem::transmute(value) }
                            }
                        }
                    });
                }
            }
            ModulusTypeInner::Struct(_) => {
                if let Ok(divisor) = NonBoundaryDivisor::try_from(divisor) {
                    let cmp = cmp_path();
                    let hash = hash_path();
                    let pat = pat_path();

                    tokens.extend(quote! {
                        #[derive(Copy)]
                        #[repr(transparent)]
                        #vis struct #type_name (
                            #pat::pattern_type!(#repr_type is 0..#divisor)
                        );
                    });
                    expand_new_unchecked_impl(tokens, type_name, divisor);
                    tokens.extend(quote! {
                        // manual implement `Clone`, `Default`, `PartialEq`, `Eq`, `PartialOrd`, `Ord`,
                        // `Hash`
                        // because deriving these traits from pattern types is not yet supported

                        const impl Clone for #type_name {
                            fn clone(&self) -> Self {
                                *self
                            }
                        }

                        const impl Default for #type_name {
                            fn default() -> Self {
                                unsafe { #mem::transmute(0 as #repr_type) }
                            }
                        }

                        const impl PartialEq for #type_name {
                            fn eq(&self, rhs: &Self) -> bool {
                                **self == **rhs
                            }
                        }

                        const impl Eq for #type_name {}

                        const impl PartialOrd for #type_name {
                            fn partial_cmp(&self, rhs: &Self) -> Option<#cmp::Ordering> {
                                (&**self).partial_cmp(rhs)
                            }
                        }

                        const impl Ord for #type_name {
                            fn cmp(&self, rhs: &Self) -> #cmp::Ordering {
                                (&**self).cmp(rhs)
                            }
                        }

                        impl #hash::Hash for #type_name {
                            fn hash<H>(&self, state: &mut H)
                            where
                                H: #hash::Hasher,
                            {
                                (&**self).hash(state);
                            }
                        }
                    });

                    // generate constants for modulus type with a small divisor
                    let count = divisor.count();
                    if (1..=256).contains(&count) {
                        let max = (count - 1) as u8;
                        let mut const_decls = quote!();
                        for i in 0..=max {
                            let ident = format_ident!("_{}", i);
                            const_decls.extend(quote! {
                                // SAFETY: macro-generated code
                                pub const #ident: Self = unsafe { #mem::transmute(#i as #repr_type) };
                            });
                        }
                        tokens.extend(quote! {
                            impl #type_name { #const_decls }
                        });
                    }
                } else if repr_type == ReprTypeOrZst::Zst {
                    tokens.extend(quote! {
                        #common_attrs
                        #vis struct #type_name;
                    });
                } else {
                    tokens.extend(quote! {
                        #common_attrs
                        #[repr(transparent)]
                        #vis struct #type_name(#repr_type);

                        const impl #type_name {
                            #[inline(always)]
                            pub fn new(value: #repr_type) -> Self {
                                Self(value)
                            }
                        }
                    });
                }
            }
        }

        expand_ops_impls(tokens, type_name, divisor);
    }
}

impl ToTokens for ModulusTypeDecls {
    fn to_tokens(&self, tokens: &mut pm2::TokenStream) {
        for quotient_type in &self.0 {
            quotient_type.to_tokens(tokens);
        }
    }
}

#[proc_macro]
pub fn new_modulus_type(input: pm::TokenStream) -> pm::TokenStream {
    let input = parse_macro_input!(input as ModulusTypeDecls);
    input.into_token_stream().into()
}

/// Generate [`Debug`](`std::fmt::Debug`) implementation for modulus types.
#[proc_macro_derive(Debug)]
pub fn modulus_type_debug(input: pm::TokenStream) -> pm::TokenStream {
    let fmt = fmt_path();
    let input = parse_macro_input!(input as DeriveInput);
    let type_name = input.ident;
    let type_name_string = type_name.to_string();
    match input.data {
        Data::Enum(data) => {
            let mut variant_branches = quote!();
            for variant in data.variants {
                let variant = variant.ident;
                let variant_string = variant.to_string();
                variant_branches.extend(quote! {
                    #type_name::#variant => write!(__cyclique_debug_formatter, #variant_string)?,
                });
            }
            quote! {
                impl #fmt::Debug for #type_name {
                    fn fmt(&self, __cyclique_debug_formatter: &mut #fmt::Formatter<'_>) -> #fmt::Result {
                        __cyclique_debug_formatter.write_str(#type_name_string)?;
                        __cyclique_debug_formatter.write_str("::")?;
                        match *self { #variant_branches }
                        Ok(())
                    }
                }
            }
            .into()
        }
        Data::Struct(data) => {
            if data.fields.is_empty() {
                quote! {
                    impl #fmt::Debug for #type_name {
                        fn fmt(&self, __cyclique_debug_formatter: &mut #fmt::Formatter<'_>) -> #fmt::Result {
                            __cyclique_debug_formatter.write_str(#type_name_string)?;
                            __cyclique_debug_formatter.write_str("(0)")?;
                            Ok(())
                        }
                    }
                }.into()
            } else {
                quote! {
                impl #fmt::Debug for #type_name {
                    fn fmt(&self, __cyclique_debug_formatter: &mut #fmt::Formatter<'_>) -> #fmt::Result {
                        __cyclique_debug_formatter.write_str(&#type_name_string)?;
                        write!(__cyclique_debug_formatter, "({:?})", **self)?;
                        Ok(())
                    }
                }
            }.into()
            }
        }
        Data::Union(_) => quote! {
            compile_error!("union is not supported")
        }
        .into(),
    }
}

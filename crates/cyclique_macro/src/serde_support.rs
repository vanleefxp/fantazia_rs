//! Generated `serde` implementations for modulus types.
//!
//! Pattern types do not implement the serde traits, so these implementations
//! are written manually: the residue is stored as the underlying
//! representation integer and validated when deserializing.

use proc_macro2::TokenStream;
use quote::quote;
use syn::Ident;

use crate::{Divisor, ReprType, ReprTypeOrZst, mem_path, repr::repr_literal};

fn serialize_method(repr: ReprType) -> TokenStream {
    match repr {
        ReprType::U8 => quote!(serialize_u8),
        ReprType::U16 => quote!(serialize_u16),
        ReprType::U32 => quote!(serialize_u32),
        ReprType::U64 => quote!(serialize_u64),
        ReprType::U128 => quote!(serialize_u128),
    }
}

/// Appends `Serialize` and `Deserialize` implementations for `type_name`.
pub fn expand_serde_impls(tokens: &mut TokenStream, type_name: &Ident, divisor: Divisor) {
    let mem = mem_path();
    let serde = quote!(::cyclique::__private::serde);

    match divisor.repr_type() {
        ReprTypeOrZst::Zst => {
            tokens.extend(quote! {
                impl #serde::Serialize for #type_name {
                    #[inline(always)]
                    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                    where
                        S: #serde::Serializer,
                    {
                        serializer.serialize_u8(0)
                    }
                }

                impl<'de> #serde::Deserialize<'de> for #type_name {
                    #[inline(always)]
                    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
                    where
                        D: #serde::Deserializer<'de>,
                    {
                        let _ = <u8 as #serde::Deserialize<'de>>::deserialize(deserializer)?;
                        Ok(#type_name::default())
                    }
                }
            });
        }
        ReprTypeOrZst::NonZero(repr) => {
            let method = serialize_method(repr);
            let range_check = if divisor.is_boundary() {
                quote!()
            } else {
                let modulus = repr_literal(repr, divisor.count());
                quote! {
                    if value >= #modulus {
                        return Err(<D::Error as #serde::de::Error>::custom(concat!(
                            "value out of range for modulus type `",
                            stringify!(#type_name),
                            "`"
                        )));
                    }
                }
            };

            tokens.extend(quote! {
                impl #serde::Serialize for #type_name {
                    #[inline(always)]
                    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                    where
                        S: #serde::Serializer,
                    {
                        serializer.#method(**self)
                    }
                }

                impl<'de> #serde::Deserialize<'de> for #type_name {
                    #[inline(always)]
                    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
                    where
                        D: #serde::Deserializer<'de>,
                    {
                        let value =
                            <#repr as #serde::Deserialize<'de>>::deserialize(deserializer)?;
                        #range_check
                        // SAFETY: the residue was checked against the modulus above, and the
                        // generated type has the same layout as its representation.
                        Ok(unsafe { #mem::transmute::<#repr, #type_name>(value) })
                    }
                }
            });
        }
    }
}

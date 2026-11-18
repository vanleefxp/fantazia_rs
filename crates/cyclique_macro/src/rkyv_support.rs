//! Generated `rkyv` implementations for modulus types.
//!
//! The archived representation is the archived form of the underlying
//! representation integer. Pattern types have no `rkyv` support of their own,
//! so the implementations are written manually and reject out-of-range
//! residues when deserializing.

use proc_macro2::TokenStream;
use quote::quote;
use syn::Ident;

use crate::{Divisor, ReprType, ReprTypeOrZst, mem_path, repr::repr_literal};

/// Returns the portable archived type of a representation type.
fn archived_type(rkyv: &TokenStream, repr: ReprType) -> TokenStream {
    match repr {
        ReprType::U8 => quote!(u8),
        ReprType::U16 => quote!(#rkyv::primitive::ArchivedU16),
        ReprType::U32 => quote!(#rkyv::primitive::ArchivedU32),
        ReprType::U64 => quote!(#rkyv::primitive::ArchivedU64),
        ReprType::U128 => quote!(#rkyv::primitive::ArchivedU128),
    }
}

/// Appends `Archive`, `Serialize` and `Deserialize` implementations for
/// `type_name`.
pub fn expand_rkyv_impls(tokens: &mut TokenStream, type_name: &Ident, divisor: Divisor) {
    let mem = mem_path();
    let rkyv = quote!(::cyclique::__private::rkyv);
    let error = quote!(::cyclique::__private::ResidueOutOfRange);

    match divisor.repr_type() {
        ReprTypeOrZst::Zst => {
            tokens.extend(quote! {
                impl #rkyv::Archive for #type_name {
                    type Archived = u8;
                    type Resolver = <u8 as #rkyv::Archive>::Resolver;

                    #[inline(always)]
                    fn resolve(
                        &self,
                        _resolver: Self::Resolver,
                        out: #rkyv::Place<Self::Archived>,
                    ) {
                        out.write(0u8)
                    }
                }

                impl<S> #rkyv::Serialize<S> for #type_name
                where
                    S: #rkyv::rancor::Fallible + ?Sized,
                {
                    #[inline(always)]
                    fn serialize(&self, _serializer: &mut S) -> Result<Self::Resolver, S::Error> {
                        Ok(())
                    }
                }

                impl<D> #rkyv::Deserialize<#type_name, D> for u8
                where
                    D: #rkyv::rancor::Fallible + ?Sized,
                {
                    #[inline(always)]
                    fn deserialize(&self, _deserializer: &mut D) -> Result<#type_name, D::Error> {
                        Ok(#type_name::default())
                    }
                }
            });
        }
        ReprTypeOrZst::NonZero(repr) => {
            let archived = archived_type(&rkyv, repr);
            let resolve = match repr {
                ReprType::U8 => quote!(<u8 as #rkyv::Archive>::resolve(&**self, resolver, out)),
                _ => quote!(out.write(<#archived>::from_native(**self))),
            };
            let native_value = match repr {
                ReprType::U8 => quote!(*self),
                _ => quote!(self.to_native()),
            };
            let boundary = divisor.is_boundary();
            let range_check = if boundary {
                quote!()
            } else {
                let modulus = repr_literal(repr, divisor.count());
                quote! {
                    if value >= #modulus {
                        return Err(<D::Error as #rkyv::rancor::Source>::new(
                            #error::new(stringify!(#type_name)),
                        ));
                    }
                }
            };
            let error_bound = if boundary {
                quote!()
            } else {
                quote!(D::Error: #rkyv::rancor::Source,)
            };

            tokens.extend(quote! {
                impl #rkyv::Archive for #type_name {
                    type Archived = #archived;
                    type Resolver = ();

                    #[inline(always)]
                    fn resolve(
                        &self,
                        resolver: Self::Resolver,
                        out: #rkyv::Place<Self::Archived>,
                    ) {
                        #resolve
                    }
                }

                impl<S> #rkyv::Serialize<S> for #type_name
                where
                    S: #rkyv::rancor::Fallible + ?Sized,
                {
                    #[inline(always)]
                    fn serialize(&self, _serializer: &mut S) -> Result<Self::Resolver, S::Error> {
                        Ok(())
                    }
                }

                impl<D> #rkyv::Deserialize<#type_name, D> for #archived
                where
                    D: #rkyv::rancor::Fallible + ?Sized,
                    #error_bound
                {
                    #[inline(always)]
                    fn deserialize(&self, _deserializer: &mut D) -> Result<#type_name, D::Error> {
                        let value: #repr = #native_value;
                        #range_check
                        // SAFETY: the residue was checked against the modulus above (except for
                        // boundary moduli, where every representation value is valid).
                        Ok(unsafe { #mem::transmute::<#repr, #type_name>(value) })
                    }
                }
            });
        }
    }
}

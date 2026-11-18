//! Resolution of the crate path that contains the `fantazia_edo12` types.

use proc_macro_crate::{FoundCrate, crate_name};
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;

/// Returns the module prefix that refers to the 12-EDO types at the macro call site.
///
/// A crate that depends on the `fantazia` facade gets `::fantazia::edo12`, while a crate that
/// depends on `fantazia_edo12` directly gets that crate's path. The facade case also covers
/// `fantazia`'s own tests and doctests: the crate declares an `extern crate self as fantazia;`
/// alias, so `::fantazia::edo12` resolves inside the crate as well.
pub(crate) fn resolver(span: Span) -> syn::Result<TokenStream> {
    if let Ok(found) = crate_name("fantazia") {
        return Ok(match found {
            FoundCrate::Itself => quote!(::fantazia::edo12),
            FoundCrate::Name(name) => {
                let name = Ident::new(&name, span);
                quote!(::#name::edo12)
            }
        });
    }

    match crate_name("fantazia_edo12") {
        Ok(FoundCrate::Itself) => Ok(quote!(crate)),
        Ok(FoundCrate::Name(name)) => {
            let name = Ident::new(&name, span);
            Ok(quote!(::#name))
        }
        Err(_) => Err(syn::Error::new(
            span,
            "the literal macros need a `fantazia` or `fantazia_edo12` dependency",
        )),
    }
}

use proc_macro2 as pm2;
use quote::quote;

pub fn mem_path() -> pm2::TokenStream {
    quote!(::cyclique::__private::core::mem)
}

pub fn ops_path() -> pm2::TokenStream {
    quote!(::cyclique::__private::core::ops)
}

pub fn pat_path() -> pm2::TokenStream {
    quote!(::cyclique::__private::core::pat)
}

pub fn num_path() -> pm2::TokenStream {
    quote!(::cyclique::__private::core::num)
}

pub fn cmp_path() -> pm2::TokenStream {
    quote!(::cyclique::__private::core::cmp)
}

pub fn hash_path() -> pm2::TokenStream {
    quote!(::cyclique::__private::core::hash)
}

pub fn fmt_path() -> pm2::TokenStream {
    quote!(::cyclique::__private::core::fmt)
}

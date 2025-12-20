//! This crate provides the derive macro for Soars.

use proc_macro::TokenStream;

mod soars;
use soars::soa;

#[proc_macro_derive(Soars, attributes(align, soa_derive, soa_array))]
pub fn derive_soars(input: TokenStream) -> TokenStream {
    soa(input)
}

mod from_soa_ref;
use from_soa_ref::from_soa_ref;

#[proc_macro_derive(FromSoaRef)]
pub fn derive_from_soa_ref(input: TokenStream) -> TokenStream {
    from_soa_ref(input)
}

//! This crate provides the derive macro for Soars.

use proc_macro::TokenStream;

mod soars;
use soars::soa;

#[proc_macro_derive(Soars, attributes(align, soa_derive, soa_array))]
pub fn derive_soars(input: TokenStream) -> TokenStream {
    soa(input)
}

mod soa_clone;
use soa_clone::soa_clone;

#[proc_macro_derive(SoaClone)]
pub fn derive_soa_clone(input: TokenStream) -> TokenStream {
    soa_clone(input)
}

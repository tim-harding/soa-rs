//! This crate provides the derive macro for Soars.

mod fields;
mod from_soa_ref_derive;
mod soars_derive;
mod zst;

use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(Soars, attributes(align, soa_derive, soa_array))]
pub fn soa(input: TokenStream) -> TokenStream {
    let input: DeriveInput = parse_macro_input!(input);
    match soars_derive::soars_derive(input) {
        Ok(tokens) => tokens,
        Err(e) => e.into_compile_error(),
    }
    .into()
}

#[proc_macro_derive(FromSoaRef)]
pub fn from_soa_ref(input: TokenStream) -> TokenStream {
    let input: DeriveInput = parse_macro_input!(input);
    match from_soa_ref_derive::from_soa_ref_derive(input) {
        Ok(tokens) => tokens,
        Err(e) => e.into_compile_error(),
    }
    .into()
}

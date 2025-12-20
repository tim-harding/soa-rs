mod fields;
mod set_by_path_error;
mod soa_attrs;
mod soa_derive;
mod soa_derive_parse;
mod zst;

use crate::soars::{fields::fields_struct, soa_attrs::SoaAttrs};
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use syn::{Data, DeriveInput, parse_macro_input};

pub fn soa(input: TokenStream) -> TokenStream {
    let input: DeriveInput = parse_macro_input!(input);
    match soars_derive(input) {
        Ok(tokens) => tokens,
        Err(e) => e.into_compile_error(),
    }
    .into()
}

fn soars_derive(input: DeriveInput) -> Result<TokenStream2, syn::Error> {
    let DeriveInput {
        ident,
        vis,
        data,
        attrs,
        generics,
    } = input;

    let attrs = SoaAttrs::new(&attrs)?;
    match data {
        Data::Struct(strukt) => fields_struct(ident, vis, strukt.fields, attrs, generics),
        Data::Enum(_) | Data::Union(_) => Err(syn::Error::new_spanned(
            ident,
            "Soars only applies to structs",
        )),
    }
}

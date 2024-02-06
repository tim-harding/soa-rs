//! This crate provides the derive macro for Soapy.

mod fields;
mod zst;

use fields::{fields_struct, FieldKind};
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote_spanned;
use syn::{parse_macro_input, Data, DeriveInput, Fields};
use zst::{zst_struct, ZstKind};

#[proc_macro_derive(Soapy)]
pub fn soa(input: TokenStream) -> TokenStream {
    let input: DeriveInput = parse_macro_input!(input);
    let span = input.ident.span();
    match soa_inner(input) {
        Ok(tokens) => tokens,
        Err(e) => match e {
            SoapyError::NotAStruct => quote_spanned! {
                span => compile_error!("Soapy only applies to structs");
            },
            SoapyError::Syn(e) => e.into_compile_error(),
        },
    }
    .into()
}

fn soa_inner(input: DeriveInput) -> Result<TokenStream2, SoapyError> {
    let DeriveInput {
        ident,
        vis,
        data,
        attrs: _,
        generics: _,
    } = input;

    match data {
        Data::Struct(strukt) => match strukt.fields {
            Fields::Named(fields) => Ok(fields_struct(ident, vis, fields.named, FieldKind::Named)?),
            Fields::Unnamed(fields) => Ok(fields_struct(
                ident,
                vis,
                fields.unnamed,
                FieldKind::Unnamed,
            )?),
            Fields::Unit => Ok(zst_struct(ident, vis, ZstKind::Unit)),
        },
        Data::Enum(_) | Data::Union(_) => Err(SoapyError::NotAStruct),
    }
}

#[derive(Debug, Clone)]
enum SoapyError {
    NotAStruct,
    Syn(syn::Error),
}

impl From<syn::Error> for SoapyError {
    fn from(value: syn::Error) -> Self {
        Self::Syn(value)
    }
}

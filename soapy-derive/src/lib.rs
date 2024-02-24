//! This crate provides the derive macro for Soapy.

mod fields;
mod zst;

use fields::{fields_struct, FieldKind};
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote_spanned;
use syn::{parse_macro_input, Attribute, Data, DeriveInput, Fields};
use zst::{zst_struct, ZstKind};

#[proc_macro_derive(Soapy, attributes(align, extra_impl))]
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
        attrs,
        generics: _,
    } = input;

    let extra_impl = ExtraImpl::try_from(attrs)?;
    match data {
        Data::Struct(strukt) => match strukt.fields {
            Fields::Named(fields) => Ok(fields_struct(
                ident,
                vis,
                fields.named,
                FieldKind::Named,
                extra_impl,
            )?),
            Fields::Unnamed(fields) => Ok(fields_struct(
                ident,
                vis,
                fields.unnamed,
                FieldKind::Unnamed,
                extra_impl,
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

#[derive(Debug, Copy, Clone, Default)]
struct ExtraImpl {
    pub debug: bool,
    pub partial_eq: bool,
    pub eq: bool,
    pub partial_ord: bool,
    pub ord: bool,
    pub hash: bool,
    pub default: bool,
    pub clone: bool,
    pub copy: bool,
}

impl TryFrom<Vec<Attribute>> for ExtraImpl {
    type Error = syn::Error;

    fn try_from(value: Vec<Attribute>) -> Result<Self, Self::Error> {
        let mut out = Self::default();
        for attr in value {
            if attr.path().is_ident("extra_impl") {
                attr.parse_nested_meta(|meta| {
                    macro_rules! ident {
                        ($i:ident, $s:expr) => {
                            if meta.path.is_ident($s) {
                                out.$i = true;
                                return Ok(());
                            }
                        };
                    }
                    ident!(debug, "Debug");
                    ident!(partial_eq, "PartialEq");
                    ident!(eq, "Eq");
                    ident!(partial_ord, "PartialOrd");
                    ident!(ord, "Ord");
                    ident!(hash, "Hash");
                    ident!(default, "Default");
                    ident!(clone, "Clone");
                    ident!(copy, "Copy");
                    Err(meta.error("unrecognized extra impl"))
                })?;
            }
        }
        Ok(out)
    }
}

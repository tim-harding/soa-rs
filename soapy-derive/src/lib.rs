//! This crate provides the derive macro for Soapy.

mod fields;
mod zst;

use std::collections::HashSet;

use fields::{fields_struct, FieldKind};
use proc_macro::TokenStream;
use proc_macro2::{Ident, Span, TokenStream as TokenStream2};
use quote::{quote, quote_spanned};
use syn::{parse_macro_input, Attribute, Data, DeriveInput, Fields};
use zst::{zst_struct, ZstKind};

#[proc_macro_derive(Soapy, attributes(align, soa_derive))]
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

    let soa_derive = SoaDerive::try_from(attrs)?;
    match data {
        Data::Struct(strukt) => match strukt.fields {
            Fields::Named(fields) => Ok(fields_struct(
                ident,
                vis,
                fields.named,
                FieldKind::Named,
                soa_derive,
            )?),
            Fields::Unnamed(fields) => Ok(fields_struct(
                ident,
                vis,
                fields.unnamed,
                FieldKind::Unnamed,
                soa_derive,
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

#[derive(Debug, Clone, Default)]
struct SoaDerive {
    derives: HashSet<syn::Path>,
}

impl SoaDerive {
    fn into_derive(self) -> TokenStream2 {
        let Self { derives } = self;
        let derives = derives.into_iter();
        quote! {
            #[derive(#(#derives),*)]
        }
    }

    fn insert(&mut self, derive: &str) {
        self.derives.insert(syn::Path::from(syn::PathSegment {
            ident: Ident::new(derive, Span::call_site()),
            arguments: syn::PathArguments::None,
        }));
    }
}

impl TryFrom<Vec<Attribute>> for SoaDerive {
    type Error = syn::Error;

    fn try_from(value: Vec<Attribute>) -> Result<Self, Self::Error> {
        let mut out = Self::default();
        for attr in value {
            if attr.path().is_ident("soa_derive") {
                let _ = attr.parse_nested_meta(|meta| {
                    out.derives.insert(meta.path);
                    Ok(())
                });
            }
        }
        Ok(out)
    }
}

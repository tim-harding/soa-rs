//! This crate provides the derive macro for Soars.

mod fields;
mod zst;

use fields::fields_struct;
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, quote_spanned};
use std::{
    error::Error,
    fmt::{self, Display, Formatter},
};
use syn::{Attribute, Data, DeriveInput, parse_macro_input};

#[proc_macro_derive(Soars, attributes(align, soa_derive, soa_array))]
pub fn soa(input: TokenStream) -> TokenStream {
    let input: DeriveInput = parse_macro_input!(input);
    let span = input.ident.span();
    match soa_inner(input) {
        Ok(tokens) => tokens,
        Err(e) => match e {
            SoarsError::NotAStruct => quote_spanned! {
                span => compile_error!("Soars only applies to structs");
            },
            SoarsError::Syn(e) => e.into_compile_error(),
        },
    }
    .into()
}

fn soa_inner(input: DeriveInput) -> Result<TokenStream2, SoarsError> {
    let DeriveInput {
        ident,
        vis,
        data,
        attrs,
        generics,
    } = input;

    let attrs = SoaAttrs::new(&attrs)?;
    match data {
        Data::Struct(strukt) => Ok(fields_struct(ident, vis, strukt.fields, attrs, generics)?),
        Data::Enum(_) | Data::Union(_) => Err(SoarsError::NotAStruct),
    }
}

#[derive(Debug, Clone)]
enum SoarsError {
    NotAStruct,
    Syn(syn::Error),
}

impl From<syn::Error> for SoarsError {
    fn from(value: syn::Error) -> Self {
        Self::Syn(value)
    }
}

#[derive(Debug, Clone)]
struct SoaAttrs {
    pub derive: SoaDerive,
    pub include_array: bool,
}

impl SoaAttrs {
    pub fn new(attributes: &[Attribute]) -> Result<Self, syn::Error> {
        let mut derive_parse = SoaDeriveParse::default();
        let mut include_array = false;
        for attr in attributes {
            let path = attr.path();
            if path.is_ident("soa_derive") {
                derive_parse.append(attr)?;
            } else if path.is_ident("soa_array") {
                include_array = true;
            }
        }

        Ok(Self {
            derive: derive_parse.into_derive(),
            include_array,
        })
    }
}

#[derive(Debug, Clone, Default)]
struct SoaDeriveParse {
    r#ref: Vec<syn::Path>,
    ref_mut: Vec<syn::Path>,
    slices: Vec<syn::Path>,
    slices_mut: Vec<syn::Path>,
    array: Vec<syn::Path>,
}

impl SoaDeriveParse {
    fn into_derive(self) -> SoaDerive {
        let Self {
            r#ref: reff,
            ref_mut,
            slices,
            slices_mut,
            array,
        } = self;
        SoaDerive {
            r#ref: quote! {
                #[derive(#(#reff),*)]
            },
            ref_mut: quote! {
                #[derive(#(#ref_mut),*)]
            },
            slices: quote! {
                #[derive(#(#slices),*)]
            },
            slices_mut: quote! {
                #[derive(#(#slices_mut),*)]
            },
            array: quote! {
                #[derive(#(#array),*)]
            },
        }
    }

    pub fn append(&mut self, attr: &Attribute) -> Result<(), syn::Error> {
        let mut collected = vec![];
        let mut mask = SoaDeriveMask::new();
        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("include") {
                mask = SoaDeriveMask::splat(false);
                meta.parse_nested_meta(|meta| {
                    mask.set_by_path(&meta.path, true).map_err(|_| {
                        meta.error(format!("unknown include specifier {:?}", meta.path))
                    })
                })?;
            } else if meta.path.is_ident("exclude") {
                meta.parse_nested_meta(|meta| {
                    mask.set_by_path(&meta.path, false).map_err(|_| {
                        meta.error(format!("unknown exclude specifier {:?}", meta.path))
                    })
                })?;
            } else {
                collected.push(meta.path);
            }
            Ok(())
        })?;

        let to_extend = mask
            .r#ref
            .then_some(&mut self.r#ref)
            .into_iter()
            .chain(mask.ref_mut.then_some(&mut self.ref_mut))
            .chain(mask.slice.then_some(&mut self.slices))
            .chain(mask.slice_mut.then_some(&mut self.slices_mut))
            .chain(mask.array.then_some(&mut self.array));

        for set in to_extend {
            set.extend(collected.iter().cloned());
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Default)]
struct SoaDerive {
    pub r#ref: TokenStream2,
    pub ref_mut: TokenStream2,
    pub slices: TokenStream2,
    pub slices_mut: TokenStream2,
    pub array: TokenStream2,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
struct SoaDeriveMask {
    pub r#ref: bool,
    pub ref_mut: bool,
    pub slice: bool,
    pub slice_mut: bool,
    pub array: bool,
}

impl SoaDeriveMask {
    pub const fn new() -> Self {
        Self::splat(true)
    }

    pub const fn splat(value: bool) -> Self {
        Self {
            r#ref: value,
            ref_mut: value,
            slice: value,
            slice_mut: value,
            array: value,
        }
    }

    pub fn set_by_path(&mut self, path: &syn::Path, value: bool) -> Result<(), SetByPathError> {
        if path.is_ident("Ref") {
            self.r#ref = value;
        } else if path.is_ident("RefMut") {
            self.ref_mut = value;
        } else if path.is_ident("Slices") {
            self.slice = value;
        } else if path.is_ident("SlicesMut") {
            self.slice_mut = value;
        } else if path.is_ident("Array") {
            self.array = value;
        } else {
            return Err(SetByPathError);
        }
        Ok(())
    }
}

impl Default for SoaDeriveMask {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
struct SetByPathError;

impl Display for SetByPathError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "unknown mask specifier")
    }
}

impl Error for SetByPathError {}

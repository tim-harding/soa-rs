use crate::fields::fields_struct;
use core::{
    error::Error,
    fmt::{self, Display, Formatter},
};
use proc_macro2::TokenStream;
use syn::{Attribute, Data, DeriveInput};

pub fn soars_derive(input: DeriveInput) -> Result<TokenStream, syn::Error> {
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

#[derive(Debug, Clone)]
pub struct SoaAttrs {
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
            r#ref: quote::quote! {
                #[derive(#(#reff),*)]
            },
            ref_mut: quote::quote! {
                #[derive(#(#ref_mut),*)]
            },
            slices: quote::quote! {
                #[derive(#(#slices),*)]
            },
            slices_mut: quote::quote! {
                #[derive(#(#slices_mut),*)]
            },
            array: quote::quote! {
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
pub struct SoaDerive {
    pub r#ref: TokenStream,
    pub ref_mut: TokenStream,
    pub slices: TokenStream,
    pub slices_mut: TokenStream,
    pub array: TokenStream,
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

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
struct SetByPathError;

impl Display for SetByPathError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "unknown mask specifier")
    }
}

impl Error for SetByPathError {}

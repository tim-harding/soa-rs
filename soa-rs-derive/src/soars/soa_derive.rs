use crate::soars::set_by_path_error::SetByPathError;
use proc_macro2::TokenStream as TokenStream2;

#[derive(Debug, Clone, Default)]
pub struct SoaDerive {
    pub r#ref: TokenStream2,
    pub ref_mut: TokenStream2,
    pub slices: TokenStream2,
    pub slices_mut: TokenStream2,
    pub array: TokenStream2,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct SoaDeriveMask {
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

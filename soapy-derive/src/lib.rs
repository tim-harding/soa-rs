use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote, quote_spanned, ToTokens};
use std::fmt::{self, Display, Formatter};
use syn::{
    parse_macro_input, punctuated::Punctuated, token::Comma, Data, DeriveInput, Field, Fields,
    Ident, Index, Visibility,
};

#[proc_macro_derive(Soapy)]
pub fn soa(input: TokenStream) -> TokenStream {
    let input: DeriveInput = parse_macro_input!(input);
    let span = input.ident.span();
    match soa_inner(input) {
        Ok(tokens) => tokens,
        Err(e) => {
            let s: &str = e.into();
            quote_spanned! {
                span => compile_error!(#s);
            }
        }
    }
    .into()
}

fn soa_inner(input: DeriveInput) -> Result<TokenStream2, SoapyError> {
    let DeriveInput {
        ident, vis, data, ..
    } = input;
    match data {
        Data::Struct(strukt) => match strukt.fields {
            Fields::Named(fields) => fields_struct(ident, vis, fields.named, FieldKind::Named),
            Fields::Unit => zst_struct(ident, vis, ZstKind::Unit),
            Fields::Unnamed(fields) => {
                fields_struct(ident, vis, fields.unnamed, FieldKind::Unnamed)
            }
        },
        Data::Enum(_) | Data::Union(_) => Err(SoapyError::NotAStruct),
    }
}

fn fields_struct(
    ident: Ident,
    vis: Visibility,
    fields: Punctuated<Field, Comma>,
    kind: FieldKind,
) -> Result<TokenStream2, SoapyError> {
    let fields_len = fields.len();
    let (vis_all, (ty_all, ident_all)): (Vec<_>, (Vec<_>, Vec<FieldIdent>)) = fields
        .into_iter()
        .enumerate()
        .map(|(i, field)| (field.vis, (field.ty, (i, field.ident).into())))
        .unzip();
    let ident_rev: Vec<_> = ident_all.iter().cloned().rev().collect();

    let (_vis_head, ident_head, ty_head) = match (
        vis_all.first().cloned(),
        ty_all.first().cloned(),
        ident_all.first().cloned(),
    ) {
        (Some(vis), Some(ty), Some(ident)) => (vis, ident, ty),
        _ => {
            let zst_kind = match kind {
                FieldKind::Named => ZstKind::Empty,
                FieldKind::Unnamed => ZstKind::EmptyTuple,
            };
            return zst_struct(ident, vis, zst_kind);
        }
    };

    let _vis_tail: Vec<_> = vis_all.iter().skip(1).cloned().collect();
    let ty_tail: Vec<_> = ty_all.iter().skip(1).cloned().collect();
    let ident_tail: Vec<_> = ident_all.iter().skip(1).cloned().collect();

    let offsets = format_ident!("{ident}SoaOffsets");
    let slices = format_ident!("{ident}SoaSlices");
    let raw = format_ident!("{ident}SoaRaw");

    let raw_body = match kind {
        FieldKind::Named => quote! {
            { #(#vis_all #ident_all: ::std::ptr::NonNull<#ty_all>,)* }
        },
        FieldKind::Unnamed => quote! {
            ( #(#vis_all ::std::ptr::NonNull<#ty_all>),* );
        },
    };

    let offsets_body = match kind {
        FieldKind::Named => quote! {
            { #(#ident_tail: usize),* }
        },

        FieldKind::Unnamed => {
            let tuple_fields = std::iter::repeat(quote! { usize }).take(fields_len - 1);
            quote! {
                ( #(#tuple_fields),* );
            }
        }
    };

    let slices_def = match kind {
        FieldKind::Named => quote! {
            { #(#vis_all #ident_all: &'a [#ty_all]),* }
        },

        FieldKind::Unnamed => quote! {
            ( #(#vis_all &'a [#ty_all]),* );
        },
    };

    let offsets_vars: Vec<_> = ident_tail
        .iter()
        .enumerate()
        .map(|(i, ident)| match ident {
            FieldIdent::Named(ident) => ident.clone(),
            FieldIdent::Unnamed(_) => format_ident!("f{}", i),
        })
        .collect();

    let offsets_idents: Vec<FieldIdent> = ident_tail
        .iter()
        .enumerate()
        .map(|(i, ident)| {
            let ident = match ident {
                FieldIdent::Named(ident) => Some(ident.clone()),
                FieldIdent::Unnamed(_) => None,
            };
            (i, ident).into()
        })
        .collect();

    let construct_offsets = match kind {
        FieldKind::Named => quote! {
            { #(#offsets_vars),* }
        },
        FieldKind::Unnamed => quote! {
            ( #(#offsets_vars),* )
        },
    };

    Ok(quote! {
        impl ::soapy_shared::Soapy for #ident {
            type SoaRaw = #raw;
        }

        #[derive(Copy, Clone)]
        struct #offsets #offsets_body

        #[derive(Copy, Clone)]
        #vis struct #raw #raw_body

        impl #raw {
            fn layout_and_offsets(cap: usize) -> (::std::alloc::Layout, #offsets) {
                let layout = ::std::alloc::Layout::array::<#ty_head>(cap).unwrap();
                #(
                let array = ::std::alloc::Layout::array::<#ty_tail>(cap).unwrap();
                let (layout, #offsets_vars) = layout.extend(array).unwrap();
                )*
                let offsets = #offsets #construct_offsets;
                (layout, offsets)
            }

            unsafe fn with_offsets(ptr: *mut u8, offsets: #offsets) -> Self {
                Self {
                    #ident_head: ::std::ptr::NonNull::new_unchecked(ptr as *mut #ty_head),
                    #(
                        #ident_tail: ::std::ptr::NonNull::new_unchecked(
                            ptr.add(offsets.#offsets_idents) as *mut #ty_tail,
                        ),
                    )*
                }
            }
        }

        impl ::soapy_shared::SoaRaw<#ident> for #raw {
            type Slices<'a> = #slices<'a> where Self: 'a;

            fn new() -> Self {
                Self {
                    #(#ident_all: ::std::ptr::NonNull::dangling(),)*
                }
            }

            fn slices(&self, len: usize) -> Self::Slices<'_> {
                #slices::new(self, len)
            }

            unsafe fn grow(&mut self, old_capacity: usize, new_capacity: usize, length: usize) {
                let (new_layout, new_offsets) = Self::layout_and_offsets(new_capacity);
                *self = if old_capacity == 0 {
                    let ptr = ::std::alloc::alloc(new_layout);
                    assert_ne!(ptr as *const u8, ::std::ptr::null());
                    Self::with_offsets(ptr, new_offsets)
                } else {
                    let (old_layout, old_offsets) = Self::layout_and_offsets(old_capacity);
                    // Grow allocation first
                    let ptr = self.#ident_head.as_ptr() as *mut u8;
                    let ptr = ::std::alloc::realloc(ptr, old_layout, new_layout.size());
                    assert_ne!(ptr as *const u8, ::std::ptr::null());
                    // Pointer may have moved, can't reuse self
                    let old = Self::with_offsets(ptr, old_offsets);
                    let new = Self::with_offsets(ptr, new_offsets);
                    // Copy do destination in reverse order to avoid
                    // overwriting data
                    #(::std::ptr::copy(old.#ident_rev.as_ptr(), new.#ident_rev.as_ptr(), length);)*
                    new
                }
            }

            unsafe fn shrink(&mut self, old_capacity: usize, new_capacity: usize, length: usize) {
                let (old_layout, _) = Self::layout_and_offsets(old_capacity);
                *self = match new_capacity {
                    // Deallocate
                    0 => {
                        let ptr = self.#ident_head.as_ptr() as *mut u8;
                        ::std::alloc::dealloc(ptr, old_layout);
                        Self::new()
                    }

                    // Move data and reallocate
                    _ => {
                        let (new_layout, new_offsets) = Self::layout_and_offsets(new_capacity);
                        // Move data before reallocating as some data
                        // may be past the end of the new allocation.
                        // Copy from front to back to avoid overwriting data.
                        let dst = Self::with_offsets(self.#ident_head.as_ptr() as *mut u8, new_offsets);
                        #(::std::ptr::copy(self.#ident_all.as_ptr(), dst.#ident_all.as_ptr(), length);)*
                        let ptr = self.#ident_head.as_ptr() as *mut u8;
                        let ptr = ::std::alloc::realloc(ptr, old_layout, new_layout.size());
                        assert_ne!(ptr as *const u8, ::std::ptr::null());
                        // Pointer may have moved, can't reuse dst
                        Self::with_offsets(ptr, new_offsets)
                    }
                };
            }

            unsafe fn dealloc(&mut self, capacity: usize) {
                if capacity == 0 || ::std::mem::size_of::<#ident>() == 0 {
                    return;
                }
                let (layout, _) = Self::layout_and_offsets(capacity);
                ::std::alloc::dealloc(self.#ident_head.as_ptr() as *mut u8, layout);
            }

            unsafe fn copy(&mut self, src: usize, dst: usize, count: usize) {
                #(
                let ptr = self.#ident_all.as_ptr();
                ::std::ptr::copy(ptr.add(src), ptr.add(dst), count);
                )*
            }

            unsafe fn set(&mut self, index: usize, element: #ident) {
                #(self.#ident_all.as_ptr().add(index).write(element.#ident_all);)*
            }

            unsafe fn get(&self, index: usize) -> #ident {
                #ident {
                    #(#ident_all: self.#ident_all.as_ptr().add(index).read(),)*
                }
            }
        }

        #vis struct #slices<'a> #slices_def

        impl<'a> #slices<'a> {
            fn new(raw: &'a #raw, len: usize) -> Self {
                Self {
                    #(
                    #ident_all: unsafe {
                        ::std::slice::from_raw_parts(raw.#ident_all.as_ptr(), len)
                    },
                    )*
                }
            }
        }
    })
}

fn zst_struct(ident: Ident, vis: Visibility, kind: ZstKind) -> Result<TokenStream2, SoapyError> {
    let raw = format_ident!("{ident}SoaRaw");
    let unit_construct = match kind {
        ZstKind::Unit => quote! {},
        ZstKind::Empty => quote! { {} },
        ZstKind::EmptyTuple => quote! { () },
    };

    Ok(quote! {
        impl ::soapy_shared::Soapy for #ident {
            type SoaRaw = #raw;
        }

        #[derive(Copy, Clone)]
        #vis struct #raw;

        impl ::soapy_shared::SoaRaw<#ident> for #raw {
            type Slices<'a> = ();

            fn new() -> Self { Self }
            fn slices(&self, len: usize) -> Self::Slices<'_> { () }
            unsafe fn grow(&mut self, old_capacity: usize, new_capacity: usize, length: usize) { }
            unsafe fn shrink(&mut self, old_capacity: usize, new_capacity: usize, length: usize) { }
            unsafe fn dealloc(&mut self, capacity: usize) {}
            unsafe fn copy(&mut self, src: usize, dst: usize, count: usize) { }
            unsafe fn set(&mut self, index: usize, element: #ident) { }
            unsafe fn get(&self, index: usize) -> #ident {
                #ident #unit_construct
            }
        }
    })
}

#[derive(Clone, PartialEq, Eq)]
enum FieldIdent {
    Named(Ident),
    Unnamed(Index),
}

impl From<(usize, Option<Ident>)> for FieldIdent {
    fn from(value: (usize, Option<Ident>)) -> Self {
        match value {
            (_, Some(ident)) => Self::Named(ident),
            (i, None) => Self::Unnamed(Index::from(i)),
        }
    }
}

impl ToTokens for FieldIdent {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        match self {
            FieldIdent::Named(ident) => ident.to_tokens(tokens),
            FieldIdent::Unnamed(i) => i.to_tokens(tokens),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum FieldKind {
    Named,
    Unnamed,
}

enum ZstKind {
    /// struct Unit;
    Unit,
    /// struct Unit {};
    Empty,
    #[allow(unused)]
    /// struct Unit();
    EmptyTuple,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum SoapyError {
    NotAStruct,
}

impl std::error::Error for SoapyError {}

impl Display for SoapyError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let s: &str = (*self).into();
        write!(f, "{}", s)
    }
}

impl From<SoapyError> for &str {
    fn from(value: SoapyError) -> Self {
        match value {
            SoapyError::NotAStruct => "Soapy only applies to structs",
        }
    }
}

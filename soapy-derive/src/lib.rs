use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote, quote_spanned};
use std::fmt::{self, Display, Formatter};
use syn::{parse_macro_input, Data, DeriveInput, Fields, FieldsNamed, Ident, Visibility};

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
            Fields::Named(fields) => named_fields_struct(ident, vis, fields),
            Fields::Unit => zst_struct(ident, vis, ZstKind::Unit),
            // TODO: Support unnamed fields
            Fields::Unnamed(_) => Err(SoapyError::UnnamedFields),
        },
        Data::Enum(_) | Data::Union(_) => Err(SoapyError::NotAStruct),
    }
}

fn named_fields_struct(
    ident: Ident,
    vis: Visibility,
    fields: FieldsNamed,
) -> Result<TokenStream2, SoapyError> {
    let fields = fields.named;
    let ((vis_head, vis_tail), (ident_head, ident_tail), (ty_head, ty_tail)) = {
        let mut fields = fields.into_iter();
        let Some(head) = fields.next() else {
            // No fields is equivalent to a unit struct
            return zst_struct(ident, vis, ZstKind::Empty);
        };
        let mut vis_tail = Vec::with_capacity(fields.len() - 1);
        let mut ident_tail = Vec::with_capacity(fields.len() - 1);
        let mut ty_tail = Vec::with_capacity(fields.len() - 1);
        for field in fields {
            vis_tail.push(field.vis);
            ident_tail.push(field.ident.unwrap());
            ty_tail.push(field.ty);
        }
        (
            (head.vis, vis_tail),
            (head.ident.unwrap(), ident_tail),
            (head.ty, ty_tail),
        )
    };

    let idents: Vec<_> = std::iter::once(&ident_head)
        .chain(ident_tail.iter())
        .cloned()
        .collect();

    let rev_idents: Vec<_> = idents.iter().cloned().rev().collect();

    let offsets = format_ident!("{ident}SoaOffsets");
    let fields = format_ident!("{ident}SoaFields");
    let fields_mut = format_ident!("{ident}SoaFieldsMut");
    let raw = format_ident!("{ident}SoaRaw");

    Ok(quote! {
        impl ::soapy_shared::Soapy for #ident {
            type SoaRaw = #raw;
        }

        #[derive(Copy, Clone)]
        struct #offsets {
            #(#ident_tail: usize,)*
        }

        #[derive(Copy, Clone)]
        #vis struct #raw {
            #ident_head: ::std::ptr::NonNull<#ty_head>,
            #(#ident_tail: ::std::ptr::NonNull<#ty_tail>,)*
        }

        impl #raw {
            fn layout_and_offsets(cap: usize) -> (::std::alloc::Layout, #offsets) {
                let layout = ::std::alloc::Layout::array::<#ty_head>(cap).unwrap();
                #(
                let array = ::std::alloc::Layout::array::<#ty_tail>(cap).unwrap();
                let (layout, #ident_tail) = layout.extend(array).unwrap();
                )*
                let offsets = #offsets {
                    #(#ident_tail,)*
                };
                (layout, offsets)
            }

            unsafe fn with_offsets(ptr: *mut u8, offsets: #offsets) -> Self {
                Self {
                    #ident_head: ::std::ptr::NonNull::new_unchecked(ptr as *mut #ty_head),
                    #(
                    #ident_tail: ::std::ptr::NonNull::new_unchecked(
                        ptr.add(offsets.#ident_tail) as *mut #ty_tail,
                    ),
                    )*
                }
            }
        }

        impl ::soapy_shared::SoaRaw<#ident> for #raw {
            type Fields<'a> = #fields<'a> where Self: 'a;
            type FieldsMut<'a> = #fields_mut<'a> where Self: 'a;

            fn new() -> Self {
                Self {
                    #(#idents: ::std::ptr::NonNull::dangling(),)*
                }
            }

            fn fields(&self, len: usize) -> Self::Fields<'_> {
                #fields {
                    raw: self,
                    len,
                }
            }

            fn fields_mut(&mut self, len: usize) -> Self::FieldsMut<'_> {
                #fields_mut {
                    raw: self,
                    len,
                }
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
                    #(::std::ptr::copy(old.#rev_idents.as_ptr(), new.#rev_idents.as_ptr(), length);)*
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
                        #(::std::ptr::copy(self.#idents.as_ptr(), dst.#idents.as_ptr(), length);)*
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
                let #idents = self.#idents.as_ptr();
                ::std::ptr::copy(#idents.add(src), #idents.add(dst), count);
                )*
            }

            unsafe fn set(&mut self, index: usize, element: #ident) {
                #(self.#idents.as_ptr().add(index).write(element.#idents);)*
            }

            unsafe fn get(&self, index: usize) -> #ident {
                #ident {
                    #(#idents: self.#idents.as_ptr().add(index).read(),)*
                }
            }
        }

        pub struct #fields<'a> {
            raw: &'a #raw,
            len: usize,
        }

        impl<'a> #fields<'a> {
            #vis_head fn #ident_head(&self) -> &[#ty_head] {
                unsafe { ::std::slice::from_raw_parts(self.raw.#ident_head.as_ptr(), self.len) }
            }

            #(
            #vis_tail fn #ident_tail(&self) -> &[#ty_tail] {
                unsafe { ::std::slice::from_raw_parts(self.raw.#ident_tail.as_ptr(), self.len) }
            }
            )*
        }

        pub struct #fields_mut<'a> {
            raw: &'a mut #raw,
            len: usize,
        }

        impl<'a> #fields_mut<'a> {
            #vis_head fn #ident_head(&mut self) -> &mut [#ty_head] {
                unsafe { ::std::slice::from_raw_parts_mut(self.raw.#ident_head.as_ptr(), self.len) }
            }

            #(
            #vis_tail fn #ident_tail(&mut self) -> &mut [#ty_tail] {
                unsafe { ::std::slice::from_raw_parts_mut(self.raw.#ident_tail.as_ptr(), self.len) }
            }
            )*
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
            type Fields<'a> = ();
            type FieldsMut<'a> = ();

            fn new() -> Self { Self }
            fn fields(&self, len: usize) -> Self::Fields<'_> { () }
            fn fields_mut(&mut self, len: usize) -> Self::FieldsMut<'_> { () }
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
    UnnamedFields,
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
            SoapyError::UnnamedFields => "Soapy only applies to structs with named fields",
        }
    }
}

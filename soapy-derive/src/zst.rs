use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};
use syn::Visibility;

pub fn zst_struct(ident: Ident, vis: Visibility, kind: ZstKind) -> TokenStream {
    let raw = format_ident!("{ident}RawSoa");
    let slice = format_ident!("{ident}SoaSlice");
    let unit_construct = match kind {
        ZstKind::Unit => quote! {},
        ZstKind::Empty => quote! { {} },
        ZstKind::EmptyTuple => quote! { () },
    };

    quote! {
        #[automatically_derived]
        impl ::soapy_shared::Soapy for #ident {
            type RawSoa = #raw;
            type Slice = #slice;
            type Deref = ();
            type Slices<'a> = ();
            type SlicesMut<'a> = ();
            type Ref<'a> = #ident;
            type RefMut<'a> = #ident;
        }

        #[automatically_derived]
        impl ::soapy_shared::WithRef for #ident {
            type Item = #ident;

            fn with_ref<F, R>(&self, f: F) -> R
            where
                F: ::std::ops::FnOnce(&Self::Item) -> R
            {
                f(self)
            }
        }

        #[automatically_derived]
        #[derive(Copy, Clone)]
        #vis struct #raw;

        #[automatically_derived]
        #vis struct #slice {
            raw: #raw,
            len: usize,
        }

        impl ::soapy_shared::Slice for #slice {
            type Raw = #raw;

            fn empty() -> Self {
                Self {
                    raw: #raw,
                    len: 0,
                }
            }

            fn len(&self) -> usize {
                self.len
            }

            unsafe fn set_len(&mut self, length: usize) {
                self.len = length
            }

            fn raw(&self) -> Self::Raw {
                self.raw
            }
        }

        #[automatically_derived]
        unsafe impl ::soapy_shared::RawSoa for #raw {
            type Item = #ident;

            #[inline]
            fn dangling() -> Self { Self }
            #[inline]
            fn as_ptr(self) -> *mut u8 { ::std::ptr::null::<u8>() as *mut _ }
            #[inline]
            unsafe fn slices(&self, start: usize, end: usize) -> () { () }
            #[inline]
            unsafe fn slices_mut(&mut self, start: usize, end: usize) -> () { () }
            #[inline]
            unsafe fn from_parts(ptr: *mut u8, capacity: usize) -> Self { Self }
            #[inline]
            unsafe fn alloc(capacity: usize) -> Self { Self }
            #[inline]
            unsafe fn realloc_grow(&mut self, old_capacity: usize, new_capacity: usize, length: usize) { }
            #[inline]
            unsafe fn realloc_shrink(&mut self, old_capacity: usize, new_capacity: usize, length: usize) { }
            #[inline]
            unsafe fn dealloc(self, old_capacity: usize) { }
            #[inline]
            unsafe fn copy(&mut self, src: usize, dst: usize, count: usize) { }
            #[inline]
            unsafe fn set(&mut self, index: usize, element: #ident) { }
            #[inline]
            unsafe fn get(&self, index: usize) -> #ident { #ident #unit_construct }
            #[inline]
            unsafe fn get_ref<'a>(&self, index: usize) -> <#ident as Soapy>::Ref<'a> { #ident #unit_construct }
            #[inline]
            unsafe fn get_mut<'a>(&self, index: usize) -> <#ident as Soapy>::RefMut<'a> { #ident #unit_construct }
        }
    }
}

pub enum ZstKind {
    /// struct Unit;
    Unit,
    /// struct Unit {};
    Empty,
    #[allow(unused)]
    /// struct Unit();
    EmptyTuple,
}

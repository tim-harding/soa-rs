use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};
use syn::Visibility;

pub fn zst_struct(ident: Ident, vis: Visibility, kind: ZstKind) -> TokenStream {
    let raw = format_ident!("{ident}SoaRaw");
    let unit_construct = match kind {
        ZstKind::Unit => quote! {},
        ZstKind::Empty => quote! { {} },
        ZstKind::EmptyTuple => quote! { () },
    };

    quote! {
        #[automatically_derived]
        unsafe impl ::soapy::Soapy for #ident {
            type Raw = #raw;
            type Deref = ();
            type Ref<'a> = #ident;
            type RefMut<'a> = #ident;
        }

        #[automatically_derived]
        impl ::soapy::WithRef for #ident {
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
        unsafe impl ::soapy::SoaRaw for #raw {
            type Item = #ident;

            #[inline]
            fn dangling() -> Self { Self }
            #[inline]
            unsafe fn from_parts(ptr: *mut u8, capacity: usize) -> Self { Self }
            #[inline]
            fn into_parts(self) -> *mut u8 { ::std::ptr::NonNull::dangling().as_ptr() }
            #[inline]
            unsafe fn alloc(capacity: usize) -> Self { Self }
            #[inline]
            unsafe fn realloc_grow(&mut self, old_capacity: usize, new_capacity: usize, length: usize) { }
            #[inline]
            unsafe fn realloc_shrink(&mut self, old_capacity: usize, new_capacity: usize, length: usize) { }
            #[inline]
            unsafe fn dealloc(self, old_capacity: usize) { }
            #[inline]
            unsafe fn copy_to(self, dst: Self, count: usize) { }
            #[inline]
            unsafe fn set(self, element: #ident) { }
            #[inline]
            unsafe fn get(self) -> #ident { #ident #unit_construct }
            #[inline]
            unsafe fn get_ref<'a>(self) -> <#ident as Soapy>::Ref<'a> { #ident #unit_construct }
            #[inline]
            unsafe fn get_mut<'a>(self) -> <#ident as Soapy>::RefMut<'a> { #ident #unit_construct }
            #[inline]
            unsafe fn offset(self, count: usize) -> Self { Self }
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

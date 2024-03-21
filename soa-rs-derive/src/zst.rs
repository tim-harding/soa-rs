use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};
use syn::Visibility;

pub fn zst_struct(ident: Ident, vis: Visibility, kind: ZstKind) -> TokenStream {
    let raw = format_ident!("{ident}SoaRaw");
    let deref = format_ident!("{ident}Deref");
    let array = format_ident!("{ident}Array");
    let unit_construct = match kind {
        ZstKind::Unit => quote! {},
        ZstKind::Empty => quote! { {} },
        ZstKind::EmptyTuple => quote! { () },
    };

    quote! {
        #[automatically_derived]
        unsafe impl ::soa_rs::Soars for #ident {
            type Raw = #raw;
            type Deref = #deref;
            type Ref<'a> = #ident;
            type RefMut<'a> = #ident;
            type Slices<'a> = #ident;
            type SlicesMut<'a> = #ident;
            type Array<const N: usize> = #array<N>;
        }

        #[automatically_derived]
        #vis struct #array<const N: usize>;

        #[automatically_derived]
        impl<const N: usize> ::soa_rs::AsSlice for #array<N> {
            type Item = #ident;

            fn as_slice(&self) -> ::soa_rs::SliceRef<'_, Self::Item> {
                let raw = #raw;
                let slice = ::soa_rs::Slice::with_raw(raw);
                unsafe { ::soa_rs::SliceRef::from_slice(slice, N) }
            }
        }

        impl<const N: usize> ::soa_rs::AsMutSlice for #array<N> {
            fn as_mut_slice(&mut self) -> ::soa_rs::SliceMut<'_, Self::Item> {
                let raw = #raw;
                let slice = ::soa_rs::Slice::with_raw(raw);
                unsafe { ::soa_rs::SliceMut::from_slice(slice, N) }
            }
        }

        // TODO: Consolidate duplication from fields
        #[automatically_derived]
        #[repr(transparent)]
        #vis struct #deref(::soa_rs::Slice<#ident>);

        impl ::soa_rs::SoaDeref for #deref {
            type Item = #ident;

            fn from_slice(slice: &::soa_rs::Slice<Self::Item>) -> &Self {
                unsafe { ::std::mem::transmute(slice) }
            }

            fn from_slice_mut(slice: &mut ::soa_rs::Slice<Self::Item>) -> &mut Self {
                unsafe { ::std::mem::transmute(slice) }
            }
        }

        #[automatically_derived]
        impl ::soa_rs::AsSoaRef for #ident {
            type Item = #ident;

            fn as_soa_ref(&self) -> Self::Item {
                #ident #unit_construct
            }
        }

        #[automatically_derived]
        #[derive(Copy, Clone)]
        #vis struct #raw;

        #[automatically_derived]
        unsafe impl ::soa_rs::SoaRaw for #raw {
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
            unsafe fn realloc_grow(
                &mut self,
                old_capacity: usize,
                new_capacity: usize,
                length: usize,
            ) -> Self { Self }

            #[inline]
            unsafe fn realloc_shrink(
                &mut self,
                old_capacity: usize,
                new_capacity: usize,
                length: usize,
            ) -> Self { Self }

            #[inline]
            unsafe fn dealloc(self, old_capacity: usize) { }

            #[inline]
            unsafe fn copy_to(self, dst: Self, count: usize) { }

            #[inline]
            unsafe fn set(self, element: #ident) { }

            #[inline]
            unsafe fn get(self) -> #ident { #ident #unit_construct }

            #[inline]
            unsafe fn get_ref<'a>(self) -> <#ident as Soars>::Ref<'a> { #ident #unit_construct }

            #[inline]
            unsafe fn get_mut<'a>(self) -> <#ident as Soars>::RefMut<'a> { #ident #unit_construct }

            #[inline]
            unsafe fn offset(self, count: usize) -> Self { Self }

            #[inline]
            unsafe fn slices<'a>(self, len: usize) -> <#ident as Soars>::Slices<'a> {
                #ident #unit_construct
            }

            #[inline]
            unsafe fn slices_mut<'a>(self, len: usize) -> <#ident as Soars>::SlicesMut<'a> {
                #ident #unit_construct
            }
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

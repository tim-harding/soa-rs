use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};
use syn::{Fields, Visibility};

pub fn zst_struct(ident: Ident, vis: Visibility, kind: Fields) -> TokenStream {
    let raw = format_ident!("{ident}SoaRaw");
    let deref = format_ident!("{ident}Deref");
    let array = format_ident!("{ident}Array");
    let unit_construct = match kind {
        Fields::Unit => quote! {},
        Fields::Named(_) => quote! { {} },
        Fields::Unnamed(_) => quote! { () },
    };

    quote! {
        // SAFETY: Self::Deref is repr(transparent) with soa_rs::Slice<Self::Raw>
        #[automatically_derived]
        unsafe impl ::soa_rs::Soars for #ident {
            type Raw = #raw;
            type Deref = #deref;
            type Ref<'a> = Self;
            type RefMut<'a> = Self;
            type Slices<'a> = Self;
            type SlicesMut<'a> = Self;
        }

        #[allow(dead_code)]
        #vis struct #array<const N: usize>;

        #[automatically_derived]
        impl<const N: usize> ::soa_rs::AsSlice for #array<N> {
            type Item = #ident;

            fn as_slice(&self) -> ::soa_rs::SliceRef<'_, Self::Item> {
                let raw = #raw;
                let slice = ::soa_rs::Slice::with_raw(raw);
                // SAFETY: Aliasing is respected because the return type
                // has the lifetime of self. We know the given slice and length
                // are compatible because they come from an array.
                unsafe { ::soa_rs::SliceRef::from_slice(slice, N) }
            }
        }

        #[automatically_derived]
        impl<const N: usize> ::soa_rs::AsMutSlice for #array<N> {
            fn as_mut_slice(&mut self) -> ::soa_rs::SliceMut<'_, Self::Item> {
                let raw = #raw;
                let slice = ::soa_rs::Slice::with_raw(raw);
                // SAFETY: Aliasing is respected because the return type
                // has the lifetime of self. We know the given slice and length
                // are compatible because they come from an array.
                unsafe { ::soa_rs::SliceMut::from_slice(slice, N) }
            }
        }

        // TODO: Consolidate duplication from fields
        #[allow(dead_code)]
        #[repr(transparent)]
        #vis struct #deref(::soa_rs::Slice<#ident>);

        #[automatically_derived]
        impl ::soa_rs::SoaDeref for #deref {
            type Item = #ident;

            fn from_slice(slice: &::soa_rs::Slice<Self::Item>) -> &Self {
                // SAFETY: Self is `repr(transparent)` of Slice
                #[allow(clippy::transmute_ptr_to_ptr)]
                unsafe { ::std::mem::transmute(slice) }
            }

            fn from_slice_mut(slice: &mut ::soa_rs::Slice<Self::Item>) -> &mut Self {
                // SAFETY: Self is `repr(transparent)` of Slice
                #[allow(clippy::transmute_ptr_to_ptr)]
                unsafe { ::std::mem::transmute(slice) }
            }
        }

        #[automatically_derived]
        impl ::soa_rs::AsSoaRef for #ident {
            type Item = Self;

            fn as_soa_ref(&self) -> Self::Item {
                Self #unit_construct
            }
        }

        #[allow(dead_code)]
        #[derive(Copy, Clone)]
        #vis struct #raw;

        #[automatically_derived]
        unsafe impl ::soa_rs::SoaRaw for #raw {
            type Item = #ident;

            #[inline]
            fn dangling() -> Self { Self }

            #[inline]
            unsafe fn from_parts(ptr: ::std::ptr::NonNull<u8>, capacity: usize) -> Self { Self }

            #[inline]
            fn into_parts(self) -> ::std::ptr::NonNull<u8> {
                ::std::ptr::NonNull::dangling()
            }

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

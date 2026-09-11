use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};
use syn::{Fields, Visibility};

pub fn zst_struct(
    ident: Ident,
    vis: Visibility,
    kind: Fields,
    derive_ref_packed: Option<&TokenStream>,
    derive_ref_mut_packed: Option<&TokenStream>,
) -> TokenStream {
    let raw = format_ident!("{ident}SoaRaw");
    let deref = format_ident!("{ident}Deref");
    let array = format_ident!("{ident}Array");
    let item_ref_packed = format_ident!("{ident}RefPacked");
    let item_ref_mut_packed = format_ident!("{ident}RefMutPacked");
    let unit_construct = match kind {
        Fields::Unit => quote! {},
        Fields::Named(_) => quote! { {} },
        Fields::Unnamed(_) => quote! { () },
    };

    let ref_packed_items = if let Some(derive_ref_packed) = derive_ref_packed {
        quote! {
            #derive_ref_packed
            #[allow(dead_code)]
            #vis struct #item_ref_packed<'soa>(::core::marker::PhantomData<&'soa #ident>);

            // Derive would impose unnecessary Copy restrictions on generic params
            impl ::core::marker::Copy for #item_ref_packed<'_> {}
            impl ::core::clone::Clone for #item_ref_packed<'_> {
                fn clone(&self) -> Self {
                    *self
                }
            }

            // SAFETY: ZST packed refs carry no raw storage; construction is a no-op
            #[automatically_derived]
            unsafe impl ::soa_rs::SoarsPacked for #ident {
                type RefPacked<'soa> = #item_ref_packed<'soa>;

                #[inline]
                unsafe fn get_ref_packed<'soa>(
                    _base: ::core::ptr::NonNull<Self::Raw>,
                    _idx: usize,
                ) -> Self::RefPacked<'soa> {
                    #item_ref_packed(::core::marker::PhantomData)
                }
            }
        }
    } else {
        TokenStream::new()
    };

    let ref_mut_packed_items = if let Some(derive_ref_mut_packed) = derive_ref_mut_packed {
        quote! {
            #derive_ref_mut_packed
            #[allow(dead_code)]
            #vis struct #item_ref_mut_packed<'soa>(::core::marker::PhantomData<&'soa mut #ident>);

            // SAFETY: ZST packed mut refs carry no raw storage; construction is a no-op
            #[automatically_derived]
            unsafe impl ::soa_rs::SoarsMutPacked for #ident {
                type RefMutPacked<'soa> = #item_ref_mut_packed<'soa>;

                #[inline]
                unsafe fn get_ref_mut_packed<'soa>(
                    _base: ::core::ptr::NonNull<Self::Raw>,
                    _idx: usize,
                ) -> Self::RefMutPacked<'soa> {
                    #item_ref_mut_packed(::core::marker::PhantomData)
                }
            }
        }
    } else {
        TokenStream::new()
    };

    quote! {
        // SAFETY: Self::Deref is repr(transparent) with soa_rs::Slice<Self::Raw>
        #[automatically_derived]
        unsafe impl ::soa_rs::Soars for #ident {
            type Raw = #raw;
            type Deref = #deref;
            type Ref<'soa> = Self;
            type RefMut<'soa> = Self;
            type Slices<'soa> = Self;
            type SlicesMut<'soa> = Self;
        }

        #ref_packed_items
        #ref_mut_packed_items

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
                unsafe { ::core::mem::transmute(slice) }
            }

            fn from_slice_mut(slice: &mut ::soa_rs::Slice<Self::Item>) -> &mut Self {
                // SAFETY: Self is `repr(transparent)` of Slice
                #[allow(clippy::transmute_ptr_to_ptr)]
                unsafe { ::core::mem::transmute(slice) }
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
            unsafe fn from_parts(ptr: ::core::ptr::NonNull<u8>, capacity: usize) -> Self { Self }

            #[inline]
            fn into_parts(self) -> ::core::ptr::NonNull<u8> {
                ::core::ptr::NonNull::dangling()
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
            unsafe fn get_ref<'soa>(self) -> <#ident as Soars>::Ref<'soa> { #ident #unit_construct }

            #[inline]
            unsafe fn get_mut<'soa>(self) -> <#ident as Soars>::RefMut<'soa> { #ident #unit_construct }

            #[inline]
            unsafe fn offset(self, count: usize) -> Self { Self }

            #[inline]
            unsafe fn slices<'soa>(self, len: usize) -> <#ident as Soars>::Slices<'soa> {
                #ident #unit_construct
            }

            #[inline]
            unsafe fn slices_mut<'soa>(self, len: usize) -> <#ident as Soars>::SlicesMut<'soa> {
                #ident #unit_construct
            }
        }
    }
}

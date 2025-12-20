use crate::soars::{soa_attrs::SoaAttrs, soa_derive::SoaDerive, zst::zst_struct};
use proc_macro2::TokenStream;
use quote::{TokenStreamExt, format_ident, quote};
use std::borrow::Cow;
use syn::{Fields, Generics, Ident, Index, LitInt, Member, Visibility};

pub fn fields_struct(
    ident: Ident,
    vis: Visibility,
    fields: Fields,
    soa_attrs: SoaAttrs,
    generics: Generics,
) -> Result<TokenStream, syn::Error> {
    let SoaAttrs {
        derive:
            SoaDerive {
                r#ref: ref derive_ref,
                ref_mut: ref derive_ref_mut,
                slices: ref derive_slices,
                slices_mut: ref derive_slices_mut,
                array: ref derive_array,
            },
        include_array,
    } = soa_attrs;

    let fields_len = fields.len();
    let ident_all = fields.members().collect::<Vec<_>>();
    let (vis_all, ty_all): (Vec<_>, Vec<_>) =
        fields.iter().map(|field| (&field.vis, &field.ty)).collect();

    let align_all = fields
        .iter()
        .map(|field| {
            for attr in &field.attrs {
                if attr.path().is_ident("align") {
                    let align_literal: LitInt = attr.parse_args()?;
                    let align: usize = align_literal.base10_parse()?;
                    if !align.is_power_of_two() {
                        return Err(syn::Error::new_spanned(
                            align_literal,
                            "align should be a power of two",
                        ));
                    }
                    return Ok(Some(align));
                }
            }
            Ok(None)
        })
        .collect::<syn::Result<Vec<_>>>()?;

    let ident_rev = ident_all.iter().rev();

    let (Some(_vis_head), Some((ident_head, ident_tail)), Some((&ty_head, ty_tail))) = (
        vis_all.first(),
        ident_all.split_first(),
        ty_all.split_first(),
    ) else {
        return Ok(zst_struct(ident, vis, fields));
    };

    let deref = format_ident!("{ident}Deref");
    let item_ref = format_ident!("{ident}Ref");
    let item_ref_mut = format_ident!("{ident}RefMut");
    let slices = format_ident!("{ident}Slices");
    let slices_mut = format_ident!("{ident}SlicesMut");
    let array = format_ident!("{ident}Array");
    let raw = format_ident!("{ident}SoaRaw");

    let mut out = TokenStream::new();

    let slice_getters_ref = ident_all.iter().map(|ident| match ident {
        Member::Named(named) => Cow::Borrowed(named),
        Member::Unnamed(Index { index, span }) => {
            Cow::Owned(Ident::new(&format!("f{index}"), *span))
        }
    });
    let slice_getters_mut = ident_all.iter().map(|ident| match ident {
        Member::Named(named) => format_ident!("{named}_mut"),
        Member::Unnamed(Index { index, span }) => Ident::new(&format!("f{index}_mut"), *span),
    });

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let mut generics_ref = generics.clone();
    generics_ref.params.push(syn::parse_quote!('soa));
    let (impl_generics_ref, ty_generics_ref, where_clause_ref) = generics_ref.split_for_impl();

    let mut generics_array = generics.clone();
    generics_array
        .params
        .push(syn::parse_quote!(const N: usize));
    let (impl_generics_array, ty_generics_array, where_clause_array) =
        generics_array.split_for_impl();

    out.append_all(quote! {
        #[allow(dead_code)]
        #[repr(transparent)]
        #vis struct #deref #impl_generics #where_clause (::soa_rs::Slice<#ident #ty_generics>);

        #[automatically_derived]
        impl #impl_generics ::soa_rs::SoaDeref for #deref #ty_generics #where_clause {
            type Item = #ident #ty_generics;

            fn from_slice(slice: &::soa_rs::Slice<Self::Item>) -> &Self {
                // SAFETY: Self is `repr(transparent)` of Slice
                unsafe { &*(slice as *const _ as *const Self) }
            }

            fn from_slice_mut(slice: &mut ::soa_rs::Slice<Self::Item>) -> &mut Self {
                // SAFETY: Self is `repr(transparent)` of Slice
                unsafe { &mut *(slice as *mut _ as *mut Self) }
            }
        }

        #[automatically_derived]
        impl #impl_generics #deref #ty_generics #where_clause {
            #(
            #vis_all const fn #slice_getters_ref(&self) -> &[#ty_all] {
                let slice = ::core::ptr::NonNull::slice_from_raw_parts(
                    self.0.raw().#ident_all,
                    self.0.len(),
                );
                // SAFETY:
                // Aliasing rules are respected because the returned lifetime
                // is bound to self. The inner type comes from the safe public API,
                // so we expect the requirements of NonNull::as_ref to be upheld.
                unsafe { slice.as_ref() }
            }

            #vis_all const fn #slice_getters_mut(&mut self) -> &mut [#ty_all] {
                let mut slice = ::core::ptr::NonNull::slice_from_raw_parts(
                    self.0.raw().#ident_all,
                    self.0.len(),
                );
                // SAFETY:
                // Aliasing rules are respected because the returned lifetime
                // is bound to self. The inner type comes from the safe public API,
                // so we expect the requirements of NonNull::as_ref to be upheld.
                unsafe { slice.as_mut() }
            }
            )*
        }
    });

    let define = |type_mapper: fn(&syn::Type) -> TokenStream| {
        let ty_mapped = ty_all.iter().copied().map(type_mapper);
        if let Fields::Named(_) = fields {
            quote! {
                { #(#vis_all #ident_all: #ty_mapped),* }
            }
        } else {
            quote! {
                ( #(#vis_all #ty_mapped),* );
            }
        }
    };

    let item_ref_def = define(|ty| quote! { &'soa #ty });
    out.append_all(quote! {
        #derive_ref
        #[allow(dead_code)]
        #vis struct #item_ref #impl_generics_ref #where_clause_ref #item_ref_def

        // Derive would impose unnecessary Copy restrictions on generic params
        impl #impl_generics_ref ::core::marker::Copy for #item_ref #ty_generics_ref #where_clause_ref {}
        impl #impl_generics_ref ::core::clone::Clone for #item_ref #ty_generics_ref #where_clause_ref {
            fn clone(&self) -> Self {
                *self
            }
        }

        #[automatically_derived]
        impl #impl_generics_ref ::soa_rs::AsSoaRef for #item_ref #ty_generics_ref #where_clause_ref {
            type Item = #ident #ty_generics;

            fn as_soa_ref(&self) -> <Self::Item as Soars>::Ref<'_> {
                *self
            }
        }
    });

    let item_ref_mut_def = define(|ty| quote! { &'soa mut #ty });
    out.append_all(quote! {
        #derive_ref_mut
        #[allow(dead_code)]
        #vis struct #item_ref_mut #impl_generics_ref #where_clause_ref #item_ref_mut_def

        #[automatically_derived]
        impl #impl_generics_ref ::soa_rs::AsSoaRef for #item_ref_mut #ty_generics_ref #where_clause_ref {
            type Item = #ident #ty_generics;

            fn as_soa_ref(&self) -> <Self::Item as Soars>::Ref<'_> {
                #item_ref {
                    #(
                    #ident_all: self.#ident_all,
                    )*
                }
            }
        }
    });

    let slices_def = define(|ty| quote! { &'soa [#ty] });
    out.append_all(quote! {
        #derive_slices
        #[allow(dead_code)]
        #vis struct #slices #impl_generics_ref #where_clause_ref #slices_def

        // Derive would impose unnecessary Copy restrictions on generic params
        impl #impl_generics_ref ::core::marker::Copy for #slices #ty_generics_ref #where_clause_ref {}
        impl #impl_generics_ref ::core::clone::Clone for #slices #ty_generics_ref #where_clause_ref {
            fn clone(&self) -> Self {
                *self
            }
        }
    });

    let slices_mut_def = define(|ty| quote! { &'soa mut [#ty] });
    out.append_all(quote! {
        #derive_slices_mut
        #[allow(dead_code)]
        #vis struct #slices_mut #impl_generics_ref #where_clause_ref #slices_mut_def
    });

    if include_array {
        let array_def = define(|ty| quote! { [#ty; N] });
        out.append_all(quote! {
            #derive_array
            #[allow(dead_code)]
            #vis struct #array #impl_generics_array #where_clause_array #array_def

            #[automatically_derived]
            impl #impl_generics_array #array #ty_generics_array #where_clause_array {
                #vis const fn from_array(array: [#ident #ty_generics; N]) -> Self {
                    let array = ::core::mem::ManuallyDrop::new(array);
                    let array = ::core::ptr::from_ref::<::core::mem::ManuallyDrop<[#ident #ty_generics; N]>>(&array);
                    let array = array.cast::<[#ident #ty_generics; N]>();
                    // SAFETY: Getting a slice this way is okay
                    // because the memory comes from an array,
                    // which is initialized and well-aligned.
                    let array = unsafe { &*array };

                    Self {
                        #(
                        #ident_all: {
                            let mut uninit = [const { ::core::mem::MaybeUninit::uninit() }; N];
                            let mut i = 0;
                            while i < N {
                                let src = ::core::ptr::from_ref(&array[i].#ident_all);
                                // SAFETY: This pointer is safe to read
                                // because it comes from a reference.
                                let read = unsafe { src.read() };
                                uninit[i] = ::core::mem::MaybeUninit::new(read);
                                i += 1;
                            }
                            // TODO: Prefer MaybeUninit::transpose when stabilized
                            // SAFETY: MaybeUninit<[T; N]> is repr(transparent) of [T; N]
                            unsafe { ::core::mem::transmute_copy(&uninit) }
                        },
                        )*
                    }
                }
            }

            #[automatically_derived]
            impl #impl_generics_array ::soa_rs::AsSlice for #array #ty_generics_array #where_clause_array {
                type Item = #ident #ty_generics;

                fn as_slice(&self) -> ::soa_rs::SliceRef<'_, Self::Item> {
                    let raw = #raw {
                        #(
                        #ident_all: ::core::ptr::NonNull::from(
                            self.#ident_all.as_slice()
                        ).cast(),
                        )*
                    };
                    let slice = ::soa_rs::Slice::with_raw(raw);
                    // SAFETY: Aliasing is respected because the return type
                    // has the lifetime of self. We know the given slice and length
                    // are compatible because they come from an array.
                    unsafe { ::soa_rs::SliceRef::from_slice(slice, N) }
                }
            }

            #[automatically_derived]
            impl #impl_generics_array ::soa_rs::AsMutSlice for #array #ty_generics_array #where_clause_array {
                fn as_mut_slice(&mut self) -> ::soa_rs::SliceMut<'_, Self::Item> {
                    let raw = #raw {
                        #(
                        #ident_all: ::core::ptr::NonNull::from(
                            self.#ident_all.as_mut_slice()
                        ).cast(),
                        )*
                    };
                    let slice = ::soa_rs::Slice::with_raw(raw);
                    // SAFETY: Aliasing is respected because the return type
                    // has the lifetime of self. We know the given slice and length
                    // are compatible because they come from an array.
                    unsafe { ::soa_rs::SliceMut::from_slice(slice, N) }
                }
            }
        });
    }

    let indices = core::iter::repeat(()).enumerate().map(|(i, ())| i);
    let offsets_len = fields_len - 1;
    let raw_body = define(|ty| quote! { ::core::ptr::NonNull<#ty> });

    let layout_and_offsets_body = |checked: bool| {
        let check_head = if checked {
            quote! {
                match
            }
        } else {
            quote! {}
        };

        let check_tail = if checked {
            quote! {
                {
                    Ok(ok) => ok,
                    Err(e) => return Err(e),
                }
            }
        } else {
            quote! {
                .unwrap_unchecked()
            }
        };

        let mut raise_align = align_all.iter().map(|align| {
            align.as_ref().map(|align| {
                quote! {
                    let array = #check_head array.align_to(#align) #check_tail;
                }
            })
        });

        let raise_align_head = raise_align.next().flatten();
        let raise_align_tail = raise_align;

        let indices = indices.clone();
        quote! {
            let array = #check_head ::core::alloc::Layout::array::<#ty_head>(cap) #check_tail;
            #raise_align_head
            let layout = array;
            let mut offsets = [0usize; #offsets_len];

            #(
            let array = #check_head ::core::alloc::Layout::array::<#ty_tail>(cap) #check_tail;
            #raise_align_tail
            let (layout, offset) = #check_head layout.extend(array) #check_tail;
            offsets[#indices] = offset;
            )*
        }
    };

    let layout_and_offsets_checked_body = layout_and_offsets_body(true);
    let layout_and_offsets_unchecked_body = layout_and_offsets_body(false);

    out.append_all(quote! {
        #[allow(dead_code)]
        #vis struct #raw #impl_generics #where_clause #raw_body

        // Derive would impose unnecessary Copy restrictions on generic params
        impl #impl_generics ::core::marker::Copy for #raw #ty_generics #where_clause {}
        impl #impl_generics ::core::clone::Clone for #raw #ty_generics #where_clause {
            fn clone(&self) -> Self {
                *self
            }
        }

        // SAFETY: Self::Deref is repr(transparent) with soa_rs::Slice<Self::Raw>
        #[automatically_derived]
        unsafe impl #impl_generics ::soa_rs::Soars for #ident #ty_generics #where_clause {
            type Raw = #raw #ty_generics;
            type Deref = #deref #ty_generics;
            type Ref<'soa> = #item_ref #ty_generics_ref where Self: 'soa;
            type RefMut<'soa> = #item_ref_mut #ty_generics_ref where Self: 'soa;
            type Slices<'soa> = #slices #ty_generics_ref where Self: 'soa;
            type SlicesMut<'soa> = #slices_mut #ty_generics_ref where Self: 'soa;
        }

        #[automatically_derived]
        impl #impl_generics #raw #ty_generics #where_clause {
            #[inline]
            const fn layout_and_offsets(cap: usize)
                -> Result<(::core::alloc::Layout, [usize; #offsets_len]), ::core::alloc::LayoutError>
            {
                #layout_and_offsets_checked_body
                Ok((layout, offsets))
            }

            // TODO: Make this const if Result::unwrap_unchecked is const stabilized
            #[inline]
            unsafe fn layout_and_offsets_unchecked(cap: usize)
                -> (::core::alloc::Layout, [usize; #offsets_len])
            {
                #layout_and_offsets_unchecked_body
                (layout, offsets)
            }

            #[inline]
            const unsafe fn with_offsets(
                ptr: ::core::ptr::NonNull<u8>,
                offsets: [usize; #offsets_len],
            ) -> Self
            {
                Self {
                    #ident_head: ptr.cast(),
                    #(
                    #ident_tail: ptr
                        // SAFETY: Caller must verify that the offsets satisfy the
                        // requirements of NonNull::add:
                        // - `ptr` and `ptr + offset` are within the same allocation
                        // - `ptr + offset` <= `isize::MAX`
                        .add(offsets[#indices])
                        .cast(),
                    )*
                }
            }
        }

        #[automatically_derived]
        unsafe impl #impl_generics ::soa_rs::SoaRaw for #raw #ty_generics #where_clause {
            type Item = #ident #ty_generics;

            #[inline]
            fn dangling() -> Self {
                Self {
                    #(#ident_all: ::core::ptr::NonNull::dangling(),)*
                }
            }

            #[inline]
            unsafe fn from_parts(ptr: ::core::ptr::NonNull<u8>, capacity: usize) -> Self {
                // SAFETY: Caller ensures ptr and capacity are from a previous allocation
                let (_, offsets) = Self::layout_and_offsets_unchecked(capacity);
                Self::with_offsets(ptr, offsets)
            }

            #[inline]
            fn into_parts(self) -> ::core::ptr::NonNull<u8> {
                self.#ident_head.cast()
            }

            #[inline]
            unsafe fn alloc(capacity: usize) -> Self {
                let (new_layout, new_offsets) = Self::layout_and_offsets(capacity)
                    .expect("capacity overflow");

                // SAFETY: Caller ensures that Self is not zero-sized
                let ptr = ::soa_rs::__alloc::alloc::alloc(new_layout);
                let Some(ptr) = ::core::ptr::NonNull::new(ptr) else {
                    ::soa_rs::__alloc::alloc::handle_alloc_error(new_layout);
                };

                Self::with_offsets(ptr, new_offsets)
            }

            #[inline]
            unsafe fn realloc_grow(
                &mut self,
                old_capacity: usize,
                new_capacity: usize,
                length: usize,
            ) -> Self {
                // SAFETY: We already constructed this layout for a previous allocation
                let (old_layout, old_offsets) = Self::layout_and_offsets_unchecked(old_capacity);
                let (new_layout, new_offsets) = Self::layout_and_offsets(new_capacity)
                    .expect("capacity overflow");

                // old_layout was already checked
                assert!(
                    new_layout.size() + new_layout.align() <= isize::MAX as usize,
                    "capacity overflow"
                );

                // Grow allocation first
                let ptr = self.#ident_head.as_ptr().cast();
                // SAFETY:
                // We ensured that the layout does not overflow isize.
                // The caller ensures that
                // - ptr has been previously allocated
                // - Self is not zero-sized
                // - new_capacity is nonzero
                // - old_layout matches the previous layout because old_capacity
                //   matches the previously allocated capacity
                let ptr = ::soa_rs::__alloc::alloc::realloc(ptr, old_layout, new_layout.size());
                let Some(ptr) = ::core::ptr::NonNull::new(ptr) else {
                    ::soa_rs::__alloc::alloc::handle_alloc_error(new_layout);
                };

                // Pointer may have moved, can't reuse self
                let old = Self::with_offsets(ptr, old_offsets);
                let new = Self::with_offsets(ptr, new_offsets);

                // Copy do destination in reverse order to avoid
                // overwriting data
                #(
                old.#ident_rev.copy_to(new.#ident_rev, length);
                )*

                new
            }

            #[inline]
            unsafe fn realloc_shrink(
                &mut self,
                old_capacity: usize,
                new_capacity: usize,
                length: usize,
            ) -> Self {
                // SAFETY: We already constructed this layout for a previous allocation
                let (old_layout, _) = Self::layout_and_offsets_unchecked(old_capacity);
                let (new_layout, new_offsets) = Self::layout_and_offsets(new_capacity)
                    .expect("capacity overflow");

                // This is smaller than old_layout, but old_layout may not have had
                // this property checked if it came from alloc instead of realloc_grow.
                assert!(
                    new_layout.size() + new_layout.align() <= isize::MAX as usize,
                    "capacity overflow"
                );

                // Move data before reallocating as some data
                // may be past the end of the new allocation.
                // Copy from front to back to avoid overwriting data.
                let ptr = self.#ident_head.cast();
                let dst = Self::with_offsets(ptr, new_offsets);
                #(
                self.#ident_all.copy_to(dst.#ident_all, length);
                )*

                // SAFETY:
                // We ensured that the layout does not overflow isize.
                // The caller ensures that
                // - ptr has been previously allocated
                // - Self is not zero-sized
                // - new_capacity is nonzero
                // - old_layout matches the previous layout because old_capacity
                //   matches the previously allocated capacity
                let ptr = ::soa_rs::__alloc::alloc::realloc(ptr.as_ptr(), old_layout, new_layout.size());
                let Some(ptr) = ::core::ptr::NonNull::new(ptr) else {
                    ::soa_rs::__alloc::alloc::handle_alloc_error(new_layout);
                };

                // Pointer may have moved, can't reuse dst
                Self::with_offsets(ptr, new_offsets)
            }

            #[inline]
            unsafe fn dealloc(self, old_capacity: usize) {
                // SAFETY: We already constructed this layout for a previous allocation
                let (layout, _) = Self::layout_and_offsets_unchecked(old_capacity);
                // SAFETY: Caller ensures that
                // - This soa was previously allocated
                // - layout is the previously used layout because old_capacity
                //   is the previously allocated capacity
                ::soa_rs::__alloc::alloc::dealloc(self.#ident_head.as_ptr().cast(), layout);
            }

            #[inline]
            unsafe fn copy_to(self, dst: Self, count: usize) {
                #(
                // SAFETY: Caller ensures count elements are valid for self and dst
                self.#ident_all.copy_to(dst.#ident_all, count);
                )*
            }

            #[inline]
            unsafe fn set(self, element: Self::Item) {
                #(
                // SAFETY: Caller ensures that self points to a soa subset
                // with at least one element
                self.#ident_all.write(element.#ident_all);
                )*
            }

            #[inline]
            unsafe fn get(self) -> Self::Item {
                #ident {
                    #(
                    // SAFETY: Caller ensures that self points to a soa subset
                    // with at least one element
                    #ident_all: self.#ident_all.read(),
                    )*
                }
            }

            #[inline]
            unsafe fn get_ref<'soa>(self) -> #item_ref #ty_generics_ref {
                #item_ref {
                    #(
                    // SAFETY: Caller ensures that self points to a soa subset
                    // with at least one element
                    #ident_all: self.#ident_all.as_ref(),
                    )*
                }
            }

            #[inline]
            unsafe fn get_mut<'soa>(mut self) -> #item_ref_mut #ty_generics_ref {
                #item_ref_mut {
                    #(
                    // SAFETY: Caller ensures that self points to a soa subset
                    // with at least one element
                    #ident_all: self.#ident_all.as_mut(),
                    )*
                }
            }

            #[inline]
            unsafe fn offset(self, count: usize) -> Self {
                Self {
                    #(
                    // SAFETY: Caller ensures that self points to a soa subset
                    // with at least `count` elements
                    #ident_all: self.#ident_all.add(count),
                    )*
                }
            }

            #[inline]
            unsafe fn slices<'soa>(self, len: usize) -> #slices #ty_generics_ref {
                #slices {
                    #(
                    #ident_all: ::core::ptr::NonNull::slice_from_raw_parts(
                        self.#ident_all,
                        len,
                    )
                    // SAFETY: Caller ensures that self points to a soa subset
                    // with at least `len` elements
                    .as_ref(),
                    )*
                }
            }

            #[inline]
            unsafe fn slices_mut<'soa>(self, len: usize) -> #slices_mut #ty_generics_ref {
                #slices_mut {
                    #(
                    #ident_all: ::core::ptr::NonNull::slice_from_raw_parts(
                        self.#ident_all,
                        len,
                    )
                    // SAFETY: Caller ensures that self points to a soa subset
                    // with at least `len` elements
                    .as_mut(),
                    )*
                }
            }
        }

        #[automatically_derived]
        impl #impl_generics ::soa_rs::AsSoaRef for #ident #ty_generics #where_clause {
            type Item = Self;

            fn as_soa_ref(&self) -> <Self::Item as ::soa_rs::Soars>::Ref<'_> {
                #item_ref {
                    #(
                    #ident_all: &self.#ident_all,
                    )*
                }
            }
        }
    });

    Ok(out)
}

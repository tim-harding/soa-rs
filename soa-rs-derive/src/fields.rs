use crate::{
    zst::{zst_struct, ZstKind},
    SoaAttrs, SoaDerive,
};
use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens, TokenStreamExt};
use syn::{punctuated::Punctuated, token::Comma, Field, Ident, Index, LitInt, Visibility};

pub fn fields_struct(
    ident: Ident,
    vis: Visibility,
    fields: Punctuated<Field, Comma>,
    kind: FieldKind,
    soa_attrs: SoaAttrs,
) -> Result<TokenStream, syn::Error> {
    let SoaAttrs {
        derive:
            SoaDerive {
                r#ref: derive_ref,
                ref_mut: derive_ref_mut,
                slices: derive_slices,
                slices_mut: derive_slices_mut,
                array: derive_array,
            },
        include_array,
    } = soa_attrs;

    let fields_len = fields.len();
    let (vis_all, ty_all, ident_all, attrs_all): (Vec<_>, Vec<_>, Vec<_>, Vec<_>) = fields
        .into_iter()
        .enumerate()
        .map(|(i, field)| {
            let Field {
                attrs,
                vis,
                mutability: _,
                ident,
                colon_token: _,
                ty,
            } = field;
            let ident: FieldIdent = (i, ident).into();
            (vis, ty, ident, attrs)
        })
        .collect();

    let align_all: Result<Vec<_>, syn::Error> = attrs_all
        .into_iter()
        .map(|attrs| {
            for attr in attrs {
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
        .collect();

    let align_all = align_all?;

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
            return Ok(zst_struct(ident, vis, zst_kind));
        }
    };

    let _vis_tail: Vec<_> = vis_all.iter().skip(1).cloned().collect();
    let ty_tail: Vec<_> = ty_all.iter().skip(1).cloned().collect();
    let ident_tail: Vec<_> = ident_all.iter().skip(1).cloned().collect();

    let deref = format_ident!("{ident}Deref");
    let item_ref = format_ident!("{ident}Ref");
    let item_ref_mut = format_ident!("{ident}RefMut");
    let slices = format_ident!("{ident}Slices");
    let slices_mut = format_ident!("{ident}SlicesMut");
    let array = format_ident!("{ident}Array");
    let raw = format_ident!("{ident}SoaRaw");

    let mut out = TokenStream::new();

    let (slice_getters_ref, slice_getters_mut): (Vec<_>, Vec<_>) = ident_all
        .iter()
        .map(|ident| match ident {
            FieldIdent::Named(named) => (named.clone(), format_ident!("{named}_mut")),
            FieldIdent::Unnamed(unnamed) => {
                (format_ident!("f{unnamed}"), format_ident!("f{unnamed}_mut"))
            }
        })
        .collect();

    out.append_all(quote! {
        #[allow(dead_code)]
        #[repr(transparent)]
        #vis struct #deref(::soa_rs::Slice<#ident>);

        #[automatically_derived]
        impl ::soa_rs::SoaDeref for #deref {
            type Item = #ident;

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
        impl #deref {
            #(
            #vis_all const fn #slice_getters_ref(&self) -> &[#ty_all] {
                let slice = ::std::ptr::NonNull::slice_from_raw_parts(
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
                let mut slice = ::std::ptr::NonNull::slice_from_raw_parts(
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

    let define = |type_mapper: &dyn Fn(&syn::Type) -> TokenStream| {
        let ty_mapped = ty_all.iter().map(type_mapper);
        match kind {
            FieldKind::Named => quote! {
                { #(#[allow(dead_code)] #vis_all #ident_all: #ty_mapped),* }
            },
            FieldKind::Unnamed => quote! {
                ( #(#[allow(dead_code)] #vis_all #ty_mapped),* );
            },
        }
    };

    let item_ref_def = define(&|ty| quote! { &'a #ty });
    out.append_all(quote! {
        #derive_ref
        #[allow(dead_code)]
        #vis struct #item_ref<'a> #item_ref_def

        #[automatically_derived]
        impl ::soa_rs::AsSoaRef for #item_ref<'_> {
            type Item = #ident;

            fn as_soa_ref(&self) -> <Self::Item as Soars>::Ref<'_> {
                *self
            }
        }
    });

    let item_ref_mut_def = define(&|ty| quote! { &'a mut #ty });
    out.append_all(quote! {
        #derive_ref_mut
        #[allow(dead_code)]
        #vis struct #item_ref_mut<'a> #item_ref_mut_def

        #[automatically_derived]
        impl ::soa_rs::AsSoaRef for #item_ref_mut<'_> {
            type Item = #ident;

            fn as_soa_ref(&self) -> <Self::Item as Soars>::Ref<'_> {
                #item_ref {
                    #(
                    #ident_all: self.#ident_all,
                    )*
                }
            }
        }
    });

    let slices_def = define(&|ty| quote! { &'a [#ty] });
    out.append_all(quote! {
        #derive_slices
        #[allow(dead_code)]
        #vis struct #slices<'a> #slices_def
    });

    let slices_mut_def = define(&|ty| quote! { &'a mut [#ty] });
    out.append_all(quote! {
        #derive_slices_mut
        #[allow(dead_code)]
        #vis struct #slices_mut<'a> #slices_mut_def
    });

    if include_array {
        let array_def = define(&|ty| quote! { [#ty; N] });
        out.append_all(quote! {
            #derive_array
            #[allow(dead_code)]
            #vis struct #array<const N: usize> #array_def

            #[automatically_derived]
            impl<const N: usize> #array<N> {
                #vis const fn from_array(array: [#ident; N]) -> Self {
                    let array = ::std::mem::ManuallyDrop::new(array);
                    let array = ::std::ptr::from_ref::<::std::mem::ManuallyDrop<[#ident; N]>>(&array);
                    let array = array.cast::<[#ident; N]>();
                    // SAFETY: Getting a slice this way is okay
                    // because the memory comes from an array,
                    // which is initialized and well-aligned.
                    let array = unsafe { &*array };

                    Self {
                        #(
                        #ident_all: {
                            let mut uninit = [const { ::std::mem::MaybeUninit::uninit() }; N];
                            let mut i = 0;
                            while i < N {
                                let src = ::std::ptr::from_ref(&array[i].#ident_all);
                                // SAFETY: This pointer is safe to read
                                // because it comes from a reference.
                                let read = unsafe { src.read() };
                                uninit[i] = ::std::mem::MaybeUninit::new(read);
                                i += 1;
                            }
                            // TODO: Prefer MaybeUninit::transpose when stabilized
                            // SAFETY: MaybeUninit<[T; N]> is repr(transparent) of [T; N]
                            unsafe { ::std::mem::transmute_copy(&uninit) }
                        },
                        )*
                    }
                }
            }

            #[automatically_derived]
            impl<const N: usize> ::soa_rs::AsSlice for #array<N> {
                type Item = #ident;

                fn as_slice(&self) -> ::soa_rs::SliceRef<'_, Self::Item> {
                    let raw = #raw {
                        #(
                        #ident_all: ::std::ptr::NonNull::from(
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
            impl<const N: usize> ::soa_rs::AsMutSlice for #array<N> {
                fn as_mut_slice(&mut self) -> ::soa_rs::SliceMut<'_, Self::Item> {
                    let raw = #raw {
                        #(
                        #ident_all: ::std::ptr::NonNull::from(
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

    let indices = std::iter::repeat(()).enumerate().map(|(i, ())| i);
    let offsets_len = fields_len - 1;
    let raw_body = define(&|ty| quote! { ::std::ptr::NonNull<#ty> });

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
        let raise_align_tail: Vec<_> = raise_align.collect();

        let indices = indices.clone();
        quote! {
            let array = #check_head ::std::alloc::Layout::array::<#ty_head>(cap) #check_tail;
            #raise_align_head
            let layout = array;
            let mut offsets = [0usize; #offsets_len];

            #(
            let array = #check_head ::std::alloc::Layout::array::<#ty_tail>(cap) #check_tail;
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
        #[derive(Copy, Clone)]
        #vis struct #raw #raw_body

        // SAFETY: Self::Deref is repr(transparent) with soa_rs::Slice<Self::Raw>
        #[automatically_derived]
        unsafe impl ::soa_rs::Soars for #ident {
            type Raw = #raw;
            type Deref = #deref;
            type Ref<'a> = #item_ref<'a> where Self: 'a;
            type RefMut<'a> = #item_ref_mut<'a> where Self: 'a;
            type Slices<'a> = #slices<'a> where Self: 'a;
            type SlicesMut<'a> = #slices_mut<'a> where Self: 'a;
        }

        #[automatically_derived]
        impl #raw {
            #[inline]
            const fn layout_and_offsets(cap: usize)
                -> Result<(::std::alloc::Layout, [usize; #offsets_len]), ::std::alloc::LayoutError>
            {
                #layout_and_offsets_checked_body
                Ok((layout, offsets))
            }

            // TODO: Make this const if Option::unwrap_unchecked is const stabilized
            #[inline]
            unsafe fn layout_and_offsets_unchecked(cap: usize)
                -> (::std::alloc::Layout, [usize; #offsets_len])
            {
                #layout_and_offsets_unchecked_body
                (layout, offsets)
            }

            #[inline]
            const unsafe fn with_offsets(
                ptr: ::std::ptr::NonNull<u8>,
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
        unsafe impl ::soa_rs::SoaRaw for #raw {
            type Item = #ident;

            #[inline]
            fn dangling() -> Self {
                Self {
                    #(#ident_all: ::std::ptr::NonNull::dangling(),)*
                }
            }

            #[inline]
            unsafe fn from_parts(ptr: ::std::ptr::NonNull<u8>, capacity: usize) -> Self {
                // SAFETY: Caller ensures ptr and capacity are from a previous allocation
                let (_, offsets) = Self::layout_and_offsets_unchecked(capacity);
                Self::with_offsets(ptr, offsets)
            }

            #[inline]
            fn into_parts(self) -> ::std::ptr::NonNull<u8> {
                self.#ident_head.cast()
            }

            #[inline]
            unsafe fn alloc(capacity: usize) -> Self {
                let (new_layout, new_offsets) = Self::layout_and_offsets(capacity)
                    .expect("capacity overflow");

                // SAFETY: Caller ensures that Self is not zero-sized
                let ptr = ::std::alloc::alloc(new_layout);
                let Some(ptr) = ::std::ptr::NonNull::new(ptr) else {
                    ::std::alloc::handle_alloc_error(new_layout);
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
                let ptr = ::std::alloc::realloc(ptr, old_layout, new_layout.size());
                let Some(ptr) = ::std::ptr::NonNull::new(ptr) else {
                    ::std::alloc::handle_alloc_error(new_layout);
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
                let ptr = ::std::alloc::realloc(ptr.as_ptr(), old_layout, new_layout.size());
                let Some(ptr) = ::std::ptr::NonNull::new(ptr) else {
                    ::std::alloc::handle_alloc_error(new_layout);
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
                ::std::alloc::dealloc(self.#ident_head.as_ptr().cast(), layout);
            }

            #[inline]
            unsafe fn copy_to(self, dst: Self, count: usize) {
                #(
                // SAFETY: Caller ensures count elements are valid for self and dst
                self.#ident_all.copy_to(dst.#ident_all, count);
                )*
            }

            #[inline]
            unsafe fn set(self, element: #ident) {
                #(
                // SAFETY: Caller ensures that self points to a soa subset
                // with at least one element
                self.#ident_all.write(element.#ident_all);
                )*
            }

            #[inline]
            unsafe fn get(self) -> #ident {
                #ident {
                    #(
                    // SAFETY: Caller ensures that self points to a soa subset
                    // with at least one element
                    #ident_all: self.#ident_all.read(),
                    )*
                }
            }

            #[inline]
            unsafe fn get_ref<'a>(self) -> #item_ref<'a> {
                #item_ref {
                    #(
                    // SAFETY: Caller ensures that self points to a soa subset
                    // with at least one element
                    #ident_all: self.#ident_all.as_ref(),
                    )*
                }
            }

            #[inline]
            unsafe fn get_mut<'a>(mut self) -> #item_ref_mut<'a> {
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
            unsafe fn slices<'a>(self, len: usize) -> #slices<'a> {
                #slices {
                    #(
                    #ident_all: ::std::ptr::NonNull::slice_from_raw_parts(
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
            unsafe fn slices_mut<'a>(self, len: usize) -> #slices_mut<'a> {
                #slices_mut {
                    #(
                    #ident_all: ::std::ptr::NonNull::slice_from_raw_parts(
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
        impl ::soa_rs::AsSoaRef for #ident {
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

#[derive(Clone, PartialEq, Eq)]
enum FieldIdent {
    Named(Ident),
    Unnamed(usize),
}

impl From<(usize, Option<Ident>)> for FieldIdent {
    fn from(value: (usize, Option<Ident>)) -> Self {
        match value {
            (_, Some(ident)) => Self::Named(ident),
            (i, None) => Self::Unnamed(i),
        }
    }
}

impl ToTokens for FieldIdent {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            FieldIdent::Named(ident) => ident.to_tokens(tokens),
            FieldIdent::Unnamed(i) => Index::from(*i).to_tokens(tokens),
        }
    }
}

impl std::fmt::Display for FieldIdent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FieldIdent::Named(ident) => write!(f, "{ident}"),
            FieldIdent::Unnamed(i) => write!(f, "{i}"),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum FieldKind {
    Named,
    Unnamed,
}

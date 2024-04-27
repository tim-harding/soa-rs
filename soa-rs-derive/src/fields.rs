use crate::{
    zst::{zst_struct, ZstKind},
    SoaDerive,
};
use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens, TokenStreamExt};
use syn::{punctuated::Punctuated, token::Comma, Field, Ident, Index, LitInt, Visibility};

pub fn fields_struct(
    ident: Ident,
    vis: Visibility,
    fields: Punctuated<Field, Comma>,
    kind: FieldKind,
    soa_derive: SoaDerive,
) -> Result<TokenStream, syn::Error> {
    let SoaDerive {
        r#ref: derive_ref,
        ref_mut: derive_ref_mut,
        slice: derive_slice,
        slice_mut: derive_slice_mut,
        array: derive_array,
    } = soa_derive;

    let fields_len = fields.len();
    let (vis_all, (ty_all, (ident_all, attrs_all))): (Vec<_>, (Vec<_>, (Vec<_>, Vec<_>))) = fields
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
            (vis, (ty, (ident, attrs)))
        })
        .unzip();

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
        .unzip();

    out.append_all(quote! {
        #[automatically_derived]
        #[repr(transparent)]
        #vis struct #deref(::soa_rs::Slice<#ident>);

        #[automatically_derived]
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
        impl #deref {
            #(
            #vis_all fn #slice_getters_ref(&self) -> &[#ty_all] {
                let ptr = self.0.raw().#ident_all.as_ptr();
                let len = self.0.len();
                unsafe {
                    ::std::slice::from_raw_parts(ptr, len)
                }
            }

            #vis_all fn #slice_getters_mut(&mut self) -> &mut [#ty_all] {
                let ptr = self.0.raw().#ident_all.as_ptr();
                let len = self.0.len();
                unsafe {
                    ::std::slice::from_raw_parts_mut(ptr, len)
                }
            }
            )*
        }
    });

    let define = |type_mapper: &dyn Fn(&syn::Type) -> TokenStream| {
        let ty_mapped = ty_all.iter().map(type_mapper);
        match kind {
            FieldKind::Named => quote! {
                { #(#[automatically_derived] #vis_all #ident_all: #ty_mapped),* }
            },
            FieldKind::Unnamed => quote! {
                ( #(#[automatically_derived] #vis_all #ty_mapped),* );
            },
        }
    };

    let item_ref_def = define(&|ty| quote! { &'a #ty });
    out.append_all(quote! {
        #derive_ref
        #[automatically_derived]
        #vis struct #item_ref<'a> #item_ref_def

        #[automatically_derived]
        impl<'a> ::soa_rs::AsSoaRef for #item_ref<'a> {
            type Item = #ident;

            fn as_soa_ref(&self) -> <Self::Item as Soars>::Ref<'_> {
                *self
            }
        }
    });

    let item_ref_mut_def = define(&|ty| quote! { &'a mut #ty });
    out.append_all(quote! {
        #derive_ref_mut
        #[automatically_derived]
        #vis struct #item_ref_mut<'a> #item_ref_mut_def

        #[automatically_derived]
        impl<'a> ::soa_rs::AsSoaRef for #item_ref_mut<'a> {
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
        #derive_slice
        #[automatically_derived]
        #vis struct #slices<'a> #slices_def
    });

    let slices_mut_def = define(&|ty| quote! { &'a mut [#ty] });
    out.append_all(quote! {
        #derive_slice_mut
        #[automatically_derived]
        #vis struct #slices_mut<'a> #slices_mut_def
    });

    let array_def = define(&|ty| quote! { [#ty; N] });
    let uninit_def = define(&|ty| quote! { [::std::mem::MaybeUninit<#ty>; K] });
    out.append_all(quote! {
        #derive_array
        #[automatically_derived]
        #vis struct #array<const N: usize> #array_def

        #[automatically_derived]
        impl<const N: usize> #array<N> {
            #vis const fn from_array(array: [#ident; N]) -> Self {
                let array = ::std::mem::ManuallyDrop::new(array);
                let array = ::std::ptr::from_ref::<::std::mem::ManuallyDrop<[#ident; N]>>(&array);
                let array = array.cast::<[#ident; N]>();
                let array = unsafe { &*array };

                struct Uninit<const K: usize> #uninit_def;

                let mut uninit: Uninit<N> = Uninit {
                    #(
                    // https://doc.rust-lang.org/std/mem/union.MaybeUninit.html#initializing-an-array-element-by-element
                    //
                    // TODO: Prefer when stablized:
                    // https://doc.rust-lang.org/std/mem/union.MaybeUninit.html#method.uninit_array
                    #ident_all: unsafe { ::std::mem::MaybeUninit::uninit().assume_init() },
                    )*
                };

                let mut i = 0;
                while i < N {
                    #(
                    let src = ::std::ptr::from_ref(&array[i].#ident_all);
                    unsafe {
                        uninit.#ident_all[i] = ::std::mem::MaybeUninit::new(src.read());
                    }
                    )*

                    i += 1;
                }

                Self {
                    #(
                    // TODO: Prefer when stabilized:
                    // https://doc.rust-lang.org/std/primitive.array.html#method.transpose
                    #ident_all: unsafe {
                        ::std::mem::transmute_copy(&::std::mem::ManuallyDrop::new(uninit.#ident_all))
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
                        #ident_all: {
                            let ptr = self.#ident_all.as_slice().as_ptr().cast_mut();
                            unsafe { ::std::ptr::NonNull::new_unchecked(ptr) }
                        },
                    )*
                };
                let slice = ::soa_rs::Slice::with_raw(raw);
                unsafe { ::soa_rs::SliceRef::from_slice(slice, N) }
            }
        }

        #[automatically_derived]
        impl<const N: usize> ::soa_rs::AsMutSlice for #array<N> {
            fn as_mut_slice(&mut self) -> ::soa_rs::SliceMut<'_, Self::Item> {
                let raw = #raw {
                    #(
                        #ident_all: {
                            let ptr = self.#ident_all.as_mut_slice().as_mut_ptr();
                            unsafe { ::std::ptr::NonNull::new_unchecked(ptr) }
                        },
                    )*
                };
                let slice = ::soa_rs::Slice::with_raw(raw);
                unsafe { ::soa_rs::SliceMut::from_slice(slice, N) }
            }
        }
    });

    let indices = std::iter::repeat(()).enumerate().map(|(i, ())| i);
    let offsets_len = fields_len - 1;
    let raw_body = define(&|ty| quote! { ::std::ptr::NonNull<#ty> });

    let layout_and_offsets_body = |checked: bool| {
        let check = if checked {
            quote! {
                ?
            }
        } else {
            quote! {
                .unwrap_unchecked()
            }
        };

        let mut raise_align = align_all.iter().map(|align| {
            align.as_ref().map(|align| {
                quote! {
                    let array = array.align_to(#align)#check;
                }
            })
        });

        let raise_align_head = raise_align.next().flatten();
        let raise_align_tail: Vec<_> = raise_align.collect();

        let indices = indices.clone();
        quote! {
            let array = ::std::alloc::Layout::array::<#ty_head>(cap)#check;
            #raise_align_head
            let layout = array;
            let mut offsets = [0usize; #offsets_len];
            #(
                let array = ::std::alloc::Layout::array::<#ty_tail>(cap)#check;
                #raise_align_tail
                let (layout, offset) = layout.extend(array)#check;
                offsets[#indices] = offset;
            )*
        }
    };

    let layout_and_offsets_checked_body = layout_and_offsets_body(true);
    let layout_and_offsets_unchecked_body = layout_and_offsets_body(false);

    out.append_all(quote! {
        #[automatically_derived]
        #[derive(Copy, Clone)]
        #vis struct #raw #raw_body

        #[automatically_derived]
        unsafe impl ::soa_rs::Soars for #ident {
            type Raw = #raw;
            type Deref = #deref;
            type Ref<'a> = #item_ref<'a> where Self: 'a;
            type RefMut<'a> = #item_ref_mut<'a> where Self: 'a;
            type Array<const N: usize> = #array<N>;
            type Slices<'a> = #slices<'a> where Self: 'a;
            type SlicesMut<'a> = #slices_mut<'a> where Self: 'a;
        }

        #[automatically_derived]
        impl #raw {
            #[inline]
            fn layout_and_offsets(cap: usize)
                -> Result<(::std::alloc::Layout, [usize; #offsets_len]), ::std::alloc::LayoutError>
            {
                #layout_and_offsets_checked_body
                Ok((layout, offsets))
            }

            #[inline]
            unsafe fn layout_and_offsets_unchecked(cap: usize)
                -> (::std::alloc::Layout, [usize; #offsets_len])
            {
                #layout_and_offsets_unchecked_body
                (layout, offsets)
            }

            #[inline]
            unsafe fn with_offsets(ptr: *mut u8, offsets: [usize; #offsets_len]) -> Self {
                Self {
                    #ident_head: ::std::ptr::NonNull::new_unchecked(ptr.cast()),
                    #(
                    #ident_tail: ::std::ptr::NonNull::new_unchecked(
                        ptr.add(offsets[#indices]).cast(),
                    )
                    ),*
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
            unsafe fn from_parts(ptr: *mut u8, capacity: usize) -> Self {
                // SAFETY: This should have come from a previous allocation
                let (_, offsets) = Self::layout_and_offsets_unchecked(capacity);
                Self::with_offsets(ptr, offsets)
            }

            #[inline]
            fn into_parts(self) -> *mut u8 {
                self.#ident_head.as_ptr().cast()
            }

            #[inline]
            unsafe fn alloc(capacity: usize) -> Self {
                let (new_layout, new_offsets) = Self::layout_and_offsets(capacity)
                    .expect("capacity overflow");

                let ptr = ::std::alloc::alloc(new_layout);
                if ptr.is_null() {
                    ::std::alloc::handle_alloc_error(new_layout);
                }

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

                // Grow allocation first
                let ptr = self.#ident_head.as_ptr().cast();
                let ptr = ::std::alloc::realloc(ptr, old_layout, new_layout.size());
                if ptr.is_null() {
                    ::std::alloc::handle_alloc_error(new_layout);
                }

                // Pointer may have moved, can't reuse self
                let old = Self::with_offsets(ptr, old_offsets);
                let new = Self::with_offsets(ptr, new_offsets);

                // Copy do destination in reverse order to avoid
                // overwriting data
                #(
                    ::std::ptr::copy(old.#ident_rev.as_ptr(), new.#ident_rev.as_ptr(), length);
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

                // Move data before reallocating as some data
                // may be past the end of the new allocation.
                // Copy from front to back to avoid overwriting data.
                let ptr = self.#ident_head.as_ptr().cast();
                let dst = Self::with_offsets(ptr, new_offsets);
                #(
                    ::std::ptr::copy(self.#ident_all.as_ptr(), dst.#ident_all.as_ptr(), length);
                )*

                let ptr = ::std::alloc::realloc(ptr, old_layout, new_layout.size());
                if ptr.is_null() {
                    ::std::alloc::handle_alloc_error(new_layout);
                }

                // Pointer may have moved, can't reuse dst
                Self::with_offsets(ptr, new_offsets)
            }

            #[inline]
            unsafe fn dealloc(self, old_capacity: usize) {
                // SAFETY: We already constructed this layout for a previous allocation
                let (layout, _) = Self::layout_and_offsets_unchecked(old_capacity);
                ::std::alloc::dealloc(self.#ident_head.as_ptr().cast(), layout);
            }

            #[inline]
            unsafe fn copy_to(self, dst: Self, count: usize) {
                #(
                    ::std::ptr::copy(self.#ident_all.as_ptr(), dst.#ident_all.as_ptr(), count);
                )*
            }

            #[inline]
            unsafe fn set(self, element: #ident) {
                #(self.#ident_all.as_ptr().write(element.#ident_all);)*
            }

            #[inline]
            unsafe fn get(self) -> #ident {
                #ident {
                    #(#ident_all: self.#ident_all.as_ptr().read(),)*
                }
            }

            #[inline]
            unsafe fn get_ref<'a>(self) -> #item_ref<'a> {
                #item_ref {
                    #(#ident_all: self.#ident_all.as_ptr().as_ref().unwrap_unchecked(),)*
                }
            }

            #[inline]
            unsafe fn get_mut<'a>(self) -> #item_ref_mut<'a> {
                #item_ref_mut {
                    #(#ident_all: self.#ident_all.as_ptr().as_mut().unwrap_unchecked(),)*
                }
            }

            #[inline]
            unsafe fn offset(self, count: usize) -> Self {
                Self {
                    #(
                    #ident_all: ::std::ptr::NonNull::new_unchecked(
                        self.#ident_all.as_ptr().add(count)
                    ),
                    )*
                }
            }

            #[inline]
            unsafe fn slices<'a>(self, len: usize) -> #slices<'a> {
                #slices {
                    #(
                        #ident_all: unsafe {
                            ::std::slice::from_raw_parts(self.#ident_all.as_ptr(), len)
                        },
                    )*
                }
            }

            #[inline]
            unsafe fn slices_mut<'a>(self, len: usize) -> #slices_mut<'a> {
                #slices_mut {
                    #(
                        #ident_all: unsafe {
                            ::std::slice::from_raw_parts_mut(self.#ident_all.as_ptr(), len)
                        },
                    )*
                }
            }
        }

        #[automatically_derived]
        impl ::soa_rs::AsSoaRef for #ident {
            type Item = #ident;

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

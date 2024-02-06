use crate::zst::{zst_struct, ZstKind};
use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens, TokenStreamExt};
use syn::{punctuated::Punctuated, token::Comma, Attribute, Field, Ident, Index, Visibility};

pub fn fields_struct(
    ident: Ident,
    vis: Visibility,
    fields: Punctuated<Field, Comma>,
    kind: FieldKind,
    _extra_impl: ExtraImpl,
) -> Result<TokenStream, syn::Error> {
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
            return Ok(zst_struct(ident, vis, zst_kind));
        }
    };

    let _vis_tail: Vec<_> = vis_all.iter().skip(1).cloned().collect();
    let ty_tail: Vec<_> = ty_all.iter().skip(1).cloned().collect();
    let ident_tail: Vec<_> = ident_all.iter().skip(1).cloned().collect();

    let deref = format_ident!("{ident}SoaDeref");
    let array = format_ident!("{ident}SoaArray");
    let uninit = format_ident!("{ident}SoaUninit");
    let item_ref = format_ident!("{ident}SoaRef");
    let item_ref_mut = format_ident!("{ident}SoaRefMut");
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
        #vis struct #deref(::soapy::Slice<#ident>);

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

    let array_def = match kind {
        FieldKind::Named => quote! {
            {
                #(
                #[automatically_derived]
                #vis_all #ident_all: [#ty_all; N],
                )*
            }
        },

        FieldKind::Unnamed => quote! {
            (
                #(
                #[automatically_derived]
                #vis_all [#ty_all; N],
                )*
            );
        },
    };

    let uninit_def = match kind {
        FieldKind::Named => quote! {
            {
                #(
                #[automatically_derived]
                #vis_all #ident_all: [::std::mem::MaybeUninit<#ty_all>; K],
                )*
            }
        },

        FieldKind::Unnamed => quote! {
            (
                #(
                #[automatically_derived]
                #vis_all [::std::mem::MaybeUninit<#ty_all>; K],
                )*
            );
        },
    };

    out.append_all(quote! {
        #[automatically_derived]
        #vis struct #array<const N: usize> #array_def

        impl<const N: usize> #array<N> {
            #vis const fn from_array(array: [#ident; N]) -> Self {
                struct #uninit<const K: usize> #uninit_def;
                let mut uninit: #uninit<N> = #uninit {
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
                    let src = &array[i].#ident_all as *const #ty_all;
                    let dst = uninit.#ident_all[i].as_ptr() as *mut #ty_all;
                    unsafe {
                        src.copy_to(dst, 1);
                    }
                    )*

                    i += 1;
                }

                ::std::mem::forget(array);
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

            const fn as_raw(&self) -> #raw {
                #raw {
                    #(
                        #ident_all: unsafe {
                            ::std::ptr::NonNull::new_unchecked(
                                self.#ident_all.as_slice().as_ptr() as *mut #ty_all
                            )
                        }
                    ),*
                }
            }

            #vis const fn as_slice(&self) -> ::soapy::SliceRef<'_, #ident> {
                unsafe {
                    let slice = ::soapy::Slice::<#ident>::from_raw_parts(self.as_raw(), N);
                    ::std::mem::transmute(slice)
                }
            }

            #vis fn as_mut_slice(&mut self) -> ::soapy::SliceMut<'_, #ident> {
                unsafe {
                    let slice = ::soapy::Slice::<#ident>::from_raw_parts(self.as_raw(), N);
                    ::std::mem::transmute(slice)
                }
            }
        }
    });

    let item_ref_def = match kind {
        FieldKind::Named => quote! {
            { #(#[automatically_derived] #vis_all #ident_all: &'a #ty_all),* }
        },
        FieldKind::Unnamed => quote! {
            ( #(#[automatically_derived] #vis_all &'a #ty_all),* );
        },
    };

    out.append_all(quote! {
        #[automatically_derived]
        #vis struct #item_ref<'a> #item_ref_def
    });

    let item_ref_mut_def = match kind {
        FieldKind::Named => quote! {
            { #(#[automatically_derived] #vis_all #ident_all: &'a mut #ty_all),* }
        },
        FieldKind::Unnamed => quote! {
            ( #(#[automatically_derived] #vis_all &'a mut #ty_all),* );
        },
    };

    out.append_all(quote! {
        #[automatically_derived]
        #vis struct #item_ref_mut<'a> #item_ref_mut_def
    });

    let with_ref_impl = |item| {
        quote! {
            impl<'a> ::soapy::WithRef for #item<'a> {
                type Item = #ident;

                fn with_ref<F, R>(&self, f: F) -> R
                where
                    F: FnOnce(&Self::Item) -> R,
                {
                    let t = ::std::mem::ManuallyDrop::new(#ident {
                        #(#ident_all: unsafe { (self.#ident_all as *const #ty_all).read() },)*
                    });
                    f(&t)
                }
            }
        }
    };

    out.append_all(with_ref_impl(item_ref.clone()));
    out.append_all(with_ref_impl(item_ref_mut.clone()));

    let indices = std::iter::repeat(()).enumerate().map(|(i, ())| i);

    let raw_body = match kind {
        FieldKind::Named => quote! {
            { #(#vis_all #ident_all: ::std::ptr::NonNull<#ty_all>,)* }
        },
        FieldKind::Unnamed => quote! {
            ( #(#vis_all ::std::ptr::NonNull<#ty_all>),* );
        },
    };

    out.append_all(quote! {
        #[automatically_derived]
        #[derive(Copy, Clone)]
        #vis struct #raw #raw_body

        #[automatically_derived]
        unsafe impl ::soapy::Soapy for #ident {
            type Raw = #raw;
            type Deref = #deref;
            type Ref<'a> = #item_ref<'a> where Self: 'a;
            type RefMut<'a> = #item_ref_mut<'a> where Self: 'a;
        }

        #[automatically_derived]
        impl #raw {
            #[inline]
            fn layout_and_offsets(cap: usize) -> (::std::alloc::Layout, [usize; #fields_len]) {
                // TODO: Replace unwraps with unwrap_unchecked
                let layout = ::std::alloc::Layout::array::<#ty_head>(cap).unwrap();
                let mut offsets = [0usize; #fields_len];
                let i = 0;
                #(
                    let array = ::std::alloc::Layout::array::<#ty_tail>(cap).unwrap();
                    let (layout, offset) = layout.extend(array).unwrap();
                    offsets[i] = offset;
                    let i = i + 1;
                )*
                (layout, offsets)
            }

            unsafe fn layout_and_offsets_unchecked(cap: usize) 
                -> (::std::alloc::Layout, [usize; #fields_len]) 
            {
                // TODO: Replace unwraps with unwrap_unchecked
                let layout = ::std::alloc::Layout::array::<#ty_head>(cap).unwrap_unchecked();
                let mut offsets = [0usize; #fields_len];
                let i = 0;
                #(
                    let array = ::std::alloc::Layout::array::<#ty_tail>(cap).unwrap_unchecked();
                    let (layout, offset) = layout.extend(array).unwrap_unchecked();
                    offsets[i] = offset;
                    let i = i + 1;
                )*
                (layout, offsets)
            }

            #[inline]
            unsafe fn with_offsets(ptr: *mut u8, offsets: [usize; #fields_len]) -> Self {
                Self {
                    #ident_head: ::std::ptr::NonNull::new_unchecked(ptr as *mut #ty_head),
                    #(
                    #ident_tail: ::std::ptr::NonNull::new_unchecked(
                        ptr.add(offsets[#indices]) as *mut #ty_tail,
                    )
                    ),*
                }
            }
        }

        #[automatically_derived]
        unsafe impl ::soapy::SoaRaw for #raw {
            type Item = #ident;

            #[inline]
            fn dangling() -> Self {
                Self {
                    #(#ident_all: ::std::ptr::NonNull::dangling(),)*
                }
            }

            #[inline]
            unsafe fn from_parts(ptr: *mut u8, capacity: usize) -> Self {
                let (_, offsets) = Self::layout_and_offsets(capacity);
                Self::with_offsets(ptr, offsets)
            }

            #[inline]
            fn into_parts(self) -> *mut u8 {
                self.#ident_head.as_ptr() as *mut _
            }

            #[inline]
            unsafe fn alloc(capacity: usize) -> Self {
                let (new_layout, new_offsets) = Self::layout_and_offsets(capacity);
                let ptr = ::std::alloc::alloc(new_layout);
                assert_ne!(ptr as *const u8, ::std::ptr::null());
                Self::with_offsets(ptr, new_offsets)
            }

            #[inline]
            unsafe fn realloc_grow(&mut self, old_capacity: usize, new_capacity: usize, length: usize) {
                // SAFETY: We already constructed this layout for a previous allocation
                let (old_layout, old_offsets) = Self::layout_and_offsets_unchecked(old_capacity);
                let (new_layout, new_offsets) = Self::layout_and_offsets(new_capacity);

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
                *self = new;
            }

            #[inline]
            unsafe fn realloc_shrink(&mut self, old_capacity: usize, new_capacity: usize, length: usize) {
                // SAFETY: We already constructed this layout for a previous allocation
                let (old_layout, _) = Self::layout_and_offsets_unchecked(old_capacity);
                let (new_layout, new_offsets) = Self::layout_and_offsets(new_capacity);
                // Move data before reallocating as some data
                // may be past the end of the new allocation.
                // Copy from front to back to avoid overwriting data.
                let ptr = self.#ident_head.as_ptr() as *mut u8;
                let dst = Self::with_offsets(ptr, new_offsets);
                #(::std::ptr::copy(self.#ident_all.as_ptr(), dst.#ident_all.as_ptr(), length);)*
                let ptr = ::std::alloc::realloc(ptr, old_layout, new_layout.size());
                assert_ne!(ptr as *const u8, ::std::ptr::null());
                // Pointer may have moved, can't reuse dst
                *self = Self::with_offsets(ptr, new_offsets);
            }

            #[inline]
            unsafe fn dealloc(self, old_capacity: usize) {
                // SAFETY: We already constructed this layout for a previous allocation
                let (layout, _) = Self::layout_and_offsets_unchecked(old_capacity);
                ::std::alloc::dealloc(self.#ident_head.as_ptr() as *mut _, layout);
            }

            #[inline]
            unsafe fn copy(&mut self, src: usize, dst: usize, count: usize) {
                #(
                    let ptr = self.#ident_all.as_ptr();
                    ::std::ptr::copy(ptr.add(src), ptr.add(dst), count);
                )*
            }

            #[inline]
            unsafe fn set(&mut self, index: usize, element: #ident) {
                #(self.#ident_all.as_ptr().add(index).write(element.#ident_all);)*
            }

            #[inline]
            unsafe fn get(&self, index: usize) -> #ident {
                #ident {
                    #(#ident_all: self.#ident_all.as_ptr().add(index).read(),)*
                }
            }

            #[inline]
            unsafe fn get_ref<'a>(&self, index: usize) -> #item_ref<'a> {
                #item_ref {
                    #(#ident_all: self.#ident_all.as_ptr().add(index).as_ref().unwrap_unchecked(),)*
                }
            }

            #[inline]
            unsafe fn get_mut<'a>(&self, index: usize) -> #item_ref_mut<'a> {
                #item_ref_mut {
                    #(#ident_all: self.#ident_all.as_ptr().add(index).as_mut().unwrap_unchecked(),)*
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
        }

        impl ::soapy::WithRef for #ident {
            type Item = Self;

            fn with_ref<F, R>(&self, f: F) -> R
            where
                F: FnOnce(&Self) -> R
            {
                f(self)
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

#[derive(Debug, Copy, Clone, Default)]
pub struct ExtraImpl {
    pub debug: bool,
    pub partial_eq: bool,
    pub eq: bool,
    pub partial_ord: bool,
    pub ord: bool,
    pub hash: bool,
    pub default: bool,
    pub clone: bool,
    pub copy: bool,
}

impl TryFrom<Vec<Attribute>> for ExtraImpl {
    type Error = syn::Error;

    fn try_from(value: Vec<Attribute>) -> Result<Self, Self::Error> {
        let mut out = Self::default();
        for attr in value {
            if attr.path().is_ident("extra_impl") {
                attr.parse_nested_meta(|meta| {
                    macro_rules! ident {
                        ($i:ident, $s:expr) => {
                            if meta.path.is_ident($s) {
                                out.$i = true;
                                return Ok(());
                            }
                        };
                    }
                    ident!(debug, "Debug");
                    ident!(partial_eq, "PartialEq");
                    ident!(eq, "Eq");
                    ident!(partial_ord, "PartialOrd");
                    ident!(ord, "Ord");
                    ident!(hash, "Hash");
                    ident!(default, "Default");
                    ident!(clone, "Clone");
                    ident!(copy, "Copy");
                    Err(meta.error("unrecognized extra impl"))
                })?;
            }
        }
        Ok(out)
    }
}

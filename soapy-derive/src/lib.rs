use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, Data, DeriveInput, Fields};

#[proc_macro_derive(Soapy)]
pub fn soa(input: TokenStream) -> TokenStream {
    let input: DeriveInput = parse_macro_input!(input);
    let strukt = match input.data {
        Data::Struct(s) => s,
        Data::Enum(_) | Data::Union(_) => return TokenStream::new(),
    };
    let vis = input.vis;
    let fields = match strukt.fields {
        Fields::Named(fields) => fields,
        Fields::Unnamed(_) | Fields::Unit => return TokenStream::new(),
    }
    .named;

    let ((vis_head, vis_tail), (ident_head, ident_tail), (ty_head, ty_tail)) = {
        let mut fields = fields.into_iter();
        let Some(head) = fields.next() else {
            return TokenStream::new();
        };
        let mut vis_tail = Vec::with_capacity(fields.len() - 1);
        let mut ident_tail = Vec::with_capacity(fields.len() - 1);
        let mut ty_tail = Vec::with_capacity(fields.len() - 1);
        for field in fields {
            vis_tail.push(field.vis);
            ident_tail.push(field.ident.unwrap()); // TODO: No unwrap
            ty_tail.push(field.ty);
        }
        (
            (head.vis, vis_tail),
            (head.ident.unwrap(), ident_tail),
            (head.ty, ty_tail),
        )
    };

    // TODO: Do I need to collect? Does iterator work?
    let idents: Vec<_> = std::iter::once(&ident_head)
        .chain(ident_tail.iter())
        .cloned()
        .collect();

    let rev_idents: Vec<_> = idents.iter().cloned().rev().collect();

    let element = input.ident;
    let offsets = format_ident!("{element}SoaOffsets");
    let fields = format_ident!("{element}SoaFields");
    let fields_mut = format_ident!("{element}SoaFieldsMut");
    let raw = format_ident!("{element}SoaRaw");

    let implementation = quote! {
        impl ::soapy_shared::Soapy for #element {
            type SoaRaw = #raw;
        }

        struct #offsets {
            #(#ident_tail: usize,)*
        }

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

            unsafe fn realloc(
                self,
                old_layout: ::std::alloc::Layout,
                new_layout: ::std::alloc::Layout,
            ) -> *mut u8 {
                let old_ptr = self.#ident_head.as_ptr() as *mut u8;
                ::std::alloc::realloc(old_ptr, old_layout, new_layout.size())
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

        impl ::soapy_shared::SoaRaw for #raw {
            type Item = #element;
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

            unsafe fn resize(self, old_capacity: usize, new_capacity: usize, len: usize) -> Self {
                assert!(len <= old_capacity);
                assert!(len <= new_capacity);
                match old_capacity.cmp(new_capacity) {
                    ::std::cmp::Ordering::Equal => self,

                    // Grow
                    ::std::cmp::Ordering::Less => {
                        let (new_layout, new_offsets) = Self::layout_and_offsets(new_capacity);
                        match old_capacity {
                            // No previous allocation
                            0 => {
                                let ptr = ::std::alloc::alloc(new_layout);
                                assert_ne!(ptr as *const u8, ::std::ptr::null());
                                Self::with_offsets(ptr, new_offsets)
                            }

                            // Grow allocation
                            _ => {
                                let (old_layout, old_offsets) = Self::layout_and_offsets(old_capacity);
                                // Grow allocation first
                                let ptr = self.realloc(old_layout, new_layout);
                                assert_ne!(ptr as *const u8, ::std::ptr::null());
                                // Pointer may have moved, can't reuse self
                                let old = Self::with_offsets(ptr, old_offsets);
                                let new = Self::with_offsets(ptr, new_offsets);
                                // Copy do destination in reverse order to avoid
                                // overwriting data
                                #(::std::ptr::copy(old.#rev_idents, new.#rev_idents, len);)*
                                new
                            }
                        }
                    }

                    // Shrink
                    ::std::cmp::Ordering::Greater => {
                        let (old_layout, _) = Self::layout_and_offsets(old_capacity);
                        match new_capacity {
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
                                let dst = Self::with_offsets(self.#ident_head, new_offsets);
                                #(::std::ptr::copy(self.#idents, dst.#idents, len);)*
                                let ptr = self.realloc(old_layout, new_layout);
                                assert_ne!(ptr as *const u8, ::std::ptr::null());
                                // Pointer may have moved, can't reuse dst
                                Self::with_offsets(ptr, new_offsets)
                            }
                        }
                    }
                }
            }

            unsafe fn dealloc(&mut self, capacity: usize) {
                let (layout, _) = Self::layout_and_offsets(capacity);
                ::std::alloc::dealloc(self.#ident_head.as_ptr() as *mut u8, layout);
            }

            unsafe fn copy(&mut self, src: usize, dst: usize, count: usize) {
                #(
                let #idents = self.#idents.as_ptr();
                ::std::ptr::copy(#idents.add(src), #idents.add(dst), count);
                )*
            }

            unsafe fn set(&mut self, index: usize, element: Self::Item) {
                #(self.#idents.as_ptr().add(index).write(element.#idents);)*
            }

            unsafe fn get(&self, index: usize) -> #element {
                #element {
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
    };

    implementation.into()
}

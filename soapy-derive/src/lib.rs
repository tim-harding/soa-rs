use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, Data, DeriveInput, Fields};

#[proc_macro_derive(Soa)]
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
            ident_tail.push(field.ident);
            ty_tail.push(field.ty);
        }
        (
            (head.vis, vis_tail),
            (head.ident, ident_tail),
            (head.ty, ty_tail),
        )
    };

    let mut_ident_head = ident_head
        .clone()
        .map(|ident| format_ident!("{}_mut", ident));
    let mut_ident_tail: Vec<_> = ident_tail
        .iter()
        .map(|ident| ident.clone().map(|ident| format_ident!("{}_mut", ident)))
        .collect();

    let ident = input.ident;
    let soa_ident = format_ident!("{}Soa", ident);
    let offsets_ident = format_ident!("{}SoaOffsets", ident);
    let into_iter_ident = format_ident!("{}SoaIntoIter", ident);

    let soa_doc = format!("A growable array of [`{ident}`]");

    let implementation = quote! {
        #[doc = #soa_doc]
        #vis struct #soa_ident {
            len: usize,
            cap: usize,
            #ident_head: ::soapy_shared::Unique<#ty_head>,
            #(#ident_tail: ::soapy_shared::Unique<#ty_tail>,)*
        }

        /// Byte offsets for the array for each field array in the SOA
        /// allocation.
        struct #offsets_ident {
            #(#ident_tail: usize,)*
        }

        impl #soa_ident {
            /// Constructs a new, empty container. The container will not
            /// allocate until elements are pushed onto it.
            pub const fn new() -> Self {
                Self {
                    len: 0,
                    cap: 0,
                    #ident_head: ::soapy_shared::Unique::dangling(),
                    #(#ident_tail: ::soapy_shared::Unique::dangling(),)*
                }
            }

            fn resize(&mut self, cap: usize) {
                let (layout, offsets) = Self::layout_and_offsets(cap);

                let ptr = if self.cap == 0 {
                    unsafe { ::std::alloc::alloc(layout) }
                } else {
                    let (old_layout, _) = Self::layout_and_offsets(self.cap);
                    let old_ptr = self.#ident_head.as_ptr() as *mut u8;
                    unsafe { ::std::alloc::realloc(old_ptr, old_layout, layout.size()) }
                };

                assert_ne!(ptr as *const u8, ::std::ptr::null());
                self.#ident_head = unsafe { ::soapy_shared::Unique::new(ptr) };
                #(self.#ident_tail = unsafe { ::soapy_shared::Unique::new(ptr.add(offsets.#ident_tail)) };)*
                self.cap = cap;
            }

            fn grow(&mut self) {
                let cap = if self.cap == 0 { 4 } else { self.cap * 2 };
                self.resize(cap);
            }

            fn layout_and_offsets(cap: usize) -> (::std::alloc::Layout, #offsets_ident) {
                use ::std::alloc::Layout;
                let layout = Layout::array::<#ty_head>(cap).unwrap();
                #(let (layout, #ident_tail) = layout.extend(Layout::array::<#ty_tail>(cap).unwrap()).unwrap();)*
                let offsets = #offsets_ident {
                    #(#ident_tail,)*
                };
                (layout, offsets)
            }

            pub fn push(&mut self, value: #ident) {
                if self.len == self.cap {
                    self.grow();
                }

                unsafe {
                    self.#ident_head.as_ptr().add(self.len).write(value.#ident_head);
                    #(self.#ident_tail.as_ptr().add(self.len).write(value.#ident_tail);)*
                }

                self.len += 1;
            }

            pub fn pop(&mut self) -> Option<#ident> {
                if self.len == 0 {
                    None
                } else {
                    self.len -= 1;
                    Some(unsafe {
                        #ident {
                            #ident_head: self.#ident_head.as_ptr().add(self.len).read(),
                            #(#ident_tail: self.#ident_tail.as_ptr().add(self.len).read(),)*
                        }
                    })
                }
            }

            pub fn insert(&mut self, index: usize, value: #ident) {
                assert!(index <= self.len, "index out of bounds");
                if self.cap == self.len {
                    self.grow();
                }
                self.len += 1;
                unsafe {
                    let #ident_head = self.#ident_head.as_ptr();
                    #(let #ident_tail = self.#ident_tail.as_ptr();)*
                    ::std::ptr::copy(#ident_head.add(index), #ident_head.add(index + 1), self.len - index);
                    #(::std::ptr::copy(#ident_tail.add(index), #ident_tail.add(index + 1), self.len - index);)*
                    #ident_head.add(index).write(value.#ident_head);
                    #(#ident_tail.add(index).write(value.#ident_tail);)*
                }
            }

            pub fn remove(&mut self, index: usize) -> #ident {
                assert!(index <= self.len, "index out of bounds");
                self.len -= 1;
                unsafe {
                    let #ident_head = self.#ident_head.as_ptr();
                    #(let #ident_tail = self.#ident_tail.as_ptr();)*
                    let out = #ident {
                        #ident_head: #ident_head.add(index).read(),
                        #(#ident_tail: #ident_tail.add(index).read(),)*
                    };
                    ::std::ptr::copy(#ident_head.add(index + 1), #ident_head.add(index), self.len - index);
                    #(::std::ptr::copy(#ident_tail.add(index + 1), #ident_tail.add(index), self.len - index);)*
                    out
                }
            }

            #vis_head fn #ident_head(&self) -> &[#ty_head] {
                unsafe { ::std::slice::from_raw_parts(self.#ident_head.as_ptr(), self.len) }
            }

            #(
            #vis_tail fn #ident_tail(&self) -> &[#ty_tail] {
                unsafe { ::std::slice::from_raw_parts(self.#ident_tail.as_ptr(), self.len) }
            }
            )*

            #vis_head fn #mut_ident_head(&mut self) -> &mut [#ty_head] {
                unsafe { ::std::slice::from_raw_parts_mut(self.#ident_head.as_ptr(), self.len) }
            }

            #(
            #vis_tail fn #mut_ident_tail(&mut self) -> &mut [#ty_tail] {
                unsafe { ::std::slice::from_raw_parts_mut(self.#ident_tail.as_ptr(), self.len) }
            }
            )*
        }

        impl Drop for #soa_ident {
            fn drop(&mut self) {
                if self.cap > 0 {
                    while let Some(_) = self.pop() {}
                    let (layout, _) = Self::layout_and_offsets(self.cap);
                    unsafe {
                        ::std::alloc::dealloc(self.#ident_head.as_ptr() as *mut u8, layout);
                    }
                }
            }
        }

        #vis struct #into_iter_ident {
            buf: ::soapy_shared::Unique<u64>,
            cap: usize,
            #ident_head: *const #ty_head,
            end: *const #ty_head,
            #(#ident_tail: *const #ty_tail,)*
        }

        impl Iterator for #into_iter_ident {
            type Item = #ident;

            fn next(&mut self) -> Option<Self::Item> {
                if self.#ident_head == self.end {
                    None
                } else {
                    unsafe {
                        let out = #ident {
                            #ident_head: self.#ident_head.read(),
                            #(#ident_tail: self.#ident_tail.read(),)*
                        };
                        self.#ident_head = self.#ident_head.offset(1);
                        #(self.#ident_tail = self.#ident_tail.offset(1);)*
                        Some(out)
                    }
                }
            }

            fn size_hint(&self) -> (usize, Option<usize>) {
                let len = (self.end as usize - self.#ident_head as usize) / ::std::mem::size_of::<u64>();
                (len, Some(len))
            }
        }

        impl IntoIterator for #soa_ident {
            type Item = #ident;

            type IntoIter = #into_iter_ident;

            fn into_iter(self) -> Self::IntoIter {
                let soa = ::std::mem::ManuallyDrop::new(self);
                unsafe {
                    #into_iter_ident {
                        buf: soa.#ident_head,
                        cap: soa.cap,
                        #ident_head: soa.#ident_head.as_ptr(),
                        end: soa.#ident_head.as_ptr().add(soa.len),
                        #(#ident_tail: soa.#ident_tail.as_ptr(),)*
                    }
                }
            }
        }

        impl Drop for #into_iter_ident {
            fn drop(&mut self) {
                if self.cap > 0 {
                    for _ in &mut *self {}
                    let (layout, _) = #soa_ident::layout_and_offsets(self.cap);
                    unsafe {
                        ::std::alloc::dealloc(self.buf.as_ptr() as *mut u8, layout);
                    }
                }
            }
        }
    };

    implementation.into()
}

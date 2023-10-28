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

    let implementation = quote! {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        struct Unique<T> {
            ptr: ::std::ptr::NonNull<T>,
            _owns_t: ::std::marker::PhantomData<T>,
        }

        unsafe impl<T: ::std::marker::Send> ::std::marker::Send for Unique<T> {}
        unsafe impl<T: ::std::marker::Sync> ::std::marker::Sync for Unique<T> {}

        impl<T> Unique<T> {
            pub const fn dangling() -> Self {
                Self {
                    ptr: ::std::ptr::NonNull::dangling(),
                    _owns_t: ::std::marker::PhantomData,
                }
            }

            /// SAFETY: Ensure that T is non-null
            pub const unsafe fn new(ptr: *mut u8) -> Self {
                Self {
                    ptr: unsafe { ::std::ptr::NonNull::new_unchecked(ptr as *mut T) },
                    _owns_t: ::std::marker::PhantomData,
                }
            }
        }

        #vis struct Soa {
            len: usize,
            cap: usize,
            #ident_head: Unique<#ty_head>,
            #(#ident_tail: Unique<#ty_tail>,)*
        }

        struct Offsets {
            #(#ident_tail: usize,)*
        }

        impl Offsets {
            pub const fn new() -> Self {
                Self {
                    #(#ident_tail: 0,)*
                }
            }
        }

        impl Soa {
            pub const fn new() -> Self {
                Self {
                    len: 0,
                    cap: 0,
                    #ident_head: Unique::dangling(),
                    #(#ident_tail: Unique::dangling(),)*
                }
            }

            fn resize(&mut self, cap: usize) {
                let (layout, offsets) = Self::layout_and_offsets(cap);

                let ptr = if self.cap == 0 {
                    unsafe { ::std::alloc::alloc(layout) }
                } else {
                    let (old_layout, _) = Self::layout_and_offsets(self.cap);
                    let old_ptr = self.#ident_head.ptr.as_ptr() as *mut u8;
                    unsafe { ::std::alloc::realloc(old_ptr, old_layout, layout.size()) }
                };

                assert_ne!(ptr as *const u8, ::std::ptr::null());
                self.#ident_head = unsafe { Unique::new(ptr) };
                #(self.#ident_tail = unsafe { Unique::new(ptr.add(offsets.#ident_tail)) };)*
                self.cap = cap;
            }

            fn grow(&mut self) {
                let cap = if self.cap == 0 { 4 } else { self.cap * 2 };
                self.resize(cap);
            }

            fn layout_and_offsets(cap: usize) -> (::std::alloc::Layout, Offsets) {
                use ::std::alloc::Layout;
                let layout = Layout::array::<#ty_head>(cap).unwrap();
                #(let (layout, #ident_tail) = layout.extend(Layout::array::<#ty_tail>(cap).unwrap()).unwrap();)*
                let offsets = Offsets {
                    #(#ident_tail,)*
                };
                (layout, offsets)
            }

            pub fn push(&mut self, el: El) {
                if self.len == self.cap {
                    self.grow();
                }

                unsafe {
                    self.#ident_head.ptr.as_ptr().add(self.len).write(el.#ident_head);
                    #(self.#ident_tail.ptr.as_ptr().add(self.len).write(el.#ident_tail);)*
                }

                self.len += 1;
            }

            pub fn pop(&mut self) -> Option<El> {
                if self.len == 0 {
                    None
                } else {
                    self.len -= 1;
                    Some(unsafe {
                        El {
                            #ident_head: self.#ident_head.ptr.as_ptr().add(self.len).read(),
                            #(#ident_tail: self.#ident_tail.ptr.as_ptr().add(self.len).read(),)*
                        }
                    })
                }
            }

            pub fn insert(&mut self, index: usize, el: El) {
                assert!(index <= self.len, "index out of bounds");
                if self.cap == self.len {
                    self.grow();
                }
                self.len += 1;
                unsafe {
                    let #ident_head = self.#ident_head.ptr.as_ptr();
                    #(let #ident_tail = self.#ident_tail.ptr.as_ptr();)*
                    ::std::ptr::copy(#ident_head.add(index), #ident_head.add(index + 1), self.len - index);
                    #(::std::ptr::copy(#ident_tail.add(index), #ident_tail.add(index + 1), self.len - index);)*
                    #ident_head.add(index).write(el.#ident_head);
                    #(#ident_tail.add(index).write(el.#ident_tail);)*
                }
            }

            pub fn remove(&mut self, index: usize) -> El {
                assert!(index <= self.len, "index out of bounds");
                self.len -= 1;
                unsafe {
                    let #ident_head = self.#ident_head.ptr.as_ptr();
                    #(let #ident_tail = self.#ident_tail.ptr.as_ptr();)*
                    let out = El {
                        #ident_head: #ident_head.add(index).read(),
                        #(#ident_tail: #ident_tail.add(index).read(),)*
                    };
                    ::std::ptr::copy(#ident_head.add(index + 1), #ident_head.add(index), self.len - index);
                    #(::std::ptr::copy(#ident_tail.add(index + 1), #ident_tail.add(index), self.len - index);)*
                    out
                }
            }

            #vis_head fn #ident_head(&self) -> &[#ty_head] {
                unsafe { ::std::slice::from_raw_parts(self.#ident_head.ptr.as_ptr(), self.len) }
            }

            #(
            #vis_tail fn #ident_tail(&self) -> &[#ty_tail] {
                unsafe { ::std::slice::from_raw_parts(self.#ident_tail.ptr.as_ptr(), self.len) }
            }
            )*

            #vis_head fn #mut_ident_head(&mut self) -> &mut [#ty_head] {
                unsafe { ::std::slice::from_raw_parts_mut(self.#ident_head.ptr.as_ptr(), self.len) }
            }

            #(
            #vis_tail fn #mut_ident_tail(&mut self) -> &mut [#ty_tail] {
                unsafe { ::std::slice::from_raw_parts_mut(self.#ident_tail.ptr.as_ptr(), self.len) }
            }
            )*
        }

        impl Drop for Soa {
            fn drop(&mut self) {
                if self.cap > 0 {
                    while let Some(_) = self.pop() {}
                    let (layout, _) = Self::layout_and_offsets(self.cap);
                    unsafe {
                        ::std::alloc::dealloc(self.#ident_head.ptr.as_ptr() as *mut u8, layout);
                    }
                }
            }
        }

        #vis struct SoaIntoIter {
            buf: Unique<u64>,
            cap: usize,
            #ident_head: *const #ty_head,
            end: *const #ty_head,
            #(#ident_tail: *const #ty_tail,)*
        }

        impl Iterator for SoaIntoIter {
            type Item = El;

            fn next(&mut self) -> Option<Self::Item> {
                if self.#ident_head == self.end {
                    None
                } else {
                    unsafe {
                        let out = El {
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

        impl IntoIterator for Soa {
            type Item = El;

            type IntoIter = SoaIntoIter;

            fn into_iter(self) -> Self::IntoIter {
                let soa = ::std::mem::ManuallyDrop::new(self);
                unsafe {
                    SoaIntoIter {
                        buf: soa.#ident_head,
                        cap: soa.cap,
                        #ident_head: soa.#ident_head.ptr.as_ptr(),
                        end: soa.#ident_head.ptr.as_ptr().add(soa.len),
                        #(#ident_tail: soa.#ident_tail.ptr.as_ptr(),)*
                    }
                }
            }
        }

        impl Drop for SoaIntoIter {
            fn drop(&mut self) {
                if self.cap > 0 {
                    for _ in &mut *self {}
                    let (layout, _) = Soa::layout_and_offsets(self.cap);
                    unsafe {
                        ::std::alloc::dealloc(self.buf.ptr.as_ptr() as *mut u8, layout);
                    }
                }
            }
        }
    };

    implementation.into()
}

use std::{
    alloc::{self, Layout},
    marker::PhantomData,
    mem::{self, ManuallyDrop},
    ptr::NonNull,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct El {
    foo: u64,
    bar: u8,
    baz: [u32; 2],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct Unique<T> {
    ptr: NonNull<T>,
    _owns_t: PhantomData<T>,
}

unsafe impl<T: Send> Send for Unique<T> {}
unsafe impl<T: Sync> Sync for Unique<T> {}

impl<T> Unique<T> {
    pub const fn dangling() -> Self {
        Self {
            ptr: NonNull::dangling(),
            _owns_t: PhantomData,
        }
    }

    /// SAFETY: Ensure that T is non-null
    pub const unsafe fn new(ptr: *mut u8) -> Self {
        Self {
            ptr: unsafe { NonNull::new_unchecked(ptr as *mut T) },
            _owns_t: PhantomData,
        }
    }
}

#[macro_export]
macro_rules! soa {
    // Naming: head ident, head type, tail ident, tail type
    ($hdi:ident : $hdt:ty, $($tli:ident : $tlt:ty),*) => {
        #[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
        pub struct El {
            $hdi: $hdt,
            $($tli : $tlt,)*
        }

        pub struct Soa {
            len: usize,
            cap: usize,
            $hdi: crate::Unique<$hdt>,
            $($tli: crate::Unique<$tlt>,)*
        }

        pub struct SoaMuts<'a> {
            soa: &'a mut Soa,
        }

        struct Offsets {
            $($tli: usize,)*
        }

        impl Offsets {
            pub const fn new() -> Self {
                Self {
                    $($tli: 0,)*
                }
            }
        }

        impl Soa {
            pub const fn new() -> Self {
                Self {
                    len: 0,
                    cap: 0,
                    $hdi: crate::Unique::dangling(),
                    $($tli: crate::Unique::dangling(),)*
                }
            }

            fn resize(&mut self, cap: usize) {
                let (layout, offsets) = Self::layout_and_offsets(cap);

                let ptr = if self.cap == 0 {
                    unsafe { std::alloc::alloc(layout) }
                } else {
                    let (old_layout, _) = Self::layout_and_offsets(self.cap);
                    let old_ptr = self.$hdi.ptr.as_ptr() as *mut u8;
                    unsafe { std::alloc::realloc(old_ptr, old_layout, layout.size()) }
                };

                assert_ne!(ptr as *const u8, std::ptr::null());
                self.$hdi = unsafe { crate::Unique::new(ptr) };
                $(self.$tli = unsafe { crate::Unique::new(ptr.add(offsets.$tli)) };)*
                self.cap = cap;
            }

            fn grow(&mut self) {
                let cap = if self.cap == 0 { 4 } else { self.cap * 2 };
                self.resize(cap);
            }

            fn layout_and_offsets(cap: usize) -> (std::alloc::Layout, Offsets) {
                let layout = std::alloc::Layout::array::<$hdt>(cap).unwrap();
                $(let (layout, $tli) = layout.extend(std::alloc::Layout::array::<$tlt>(cap).unwrap()).unwrap();)*
                let offsets = Offsets {
                    $($tli,)*
                };
                (layout, offsets)
            }

            pub fn push(&mut self, el: El) {
                if self.len == self.cap {
                    self.grow();
                }

                unsafe {
                    self.$hdi.ptr.as_ptr().add(self.len).write(el.$hdi);
                    $(self.$tli.ptr.as_ptr().add(self.len).write(el.$tli);)*
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
                            $hdi: self.$hdi.ptr.as_ptr().add(self.len).read(),
                            $($tli: self.$tli.ptr.as_ptr().add(self.len).read(),)*
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
                    let $hdi = self.$hdi.ptr.as_ptr();
                    $(let $tli = self.$tli.ptr.as_ptr();)*
                    std::ptr::copy($hdi.add(index), $hdi.add(index + 1), self.len - index);
                    $(std::ptr::copy($tli.add(index), $tli.add(index + 1), self.len - index);)*
                    $hdi.add(index).write(el.$hdi);
                    $($tli.add(index).write(el.$tli);)*
                }
            }

            pub fn remove(&mut self, index: usize) -> El {
                assert!(index <= self.len, "index out of bounds");
                self.len -= 1;
                unsafe {
                    let $hdi = self.$hdi.ptr.as_ptr();
                    $(let $tli = self.$tli.ptr.as_ptr();)*
                    let out = El {
                        $hdi: $hdi.add(index).read(),
                        $($tli: $tli.add(index).read(),)*
                    };
                    std::ptr::copy($hdi.add(index + 1), $hdi.add(index), self.len - index);
                    $(std::ptr::copy($tli.add(index + 1), $tli.add(index), self.len - index);)*
                    out
                }
            }

            pub fn $hdi(&self) -> &[$hdt] {
                unsafe { std::slice::from_raw_parts(self.$hdi.ptr.as_ptr(), self.len) }
            }

            $(
            pub fn $tli(&self) -> &[$tlt] {
                unsafe { std::slice::from_raw_parts(self.$tli.ptr.as_ptr(), self.len) }
            }
            )*

            pub fn muts(&mut self) -> SoaMuts {
                SoaMuts {
                    soa: self,
                }
            }
        }

        impl Drop for Soa {
            fn drop(&mut self) {
                if self.cap > 0 {
                    while let Some(_) = self.pop() {}
                    let (layout, _) = Self::layout_and_offsets(self.cap);
                    unsafe {
                        std::alloc::dealloc(self.$hdi.ptr.as_ptr() as *mut u8, layout);
                    }
                }
            }
        }

        impl<'a> SoaMuts<'a> {
            pub fn $hdi(&mut self) -> &mut [$hdt] {
                unsafe { std::slice::from_raw_parts_mut(self.soa.$hdi.ptr.as_ptr(), self.soa.len) }
            }

            $(
            pub fn $tli(&mut self) -> &mut [$tlt] {
                unsafe { std::slice::from_raw_parts_mut(self.soa.$tli.ptr.as_ptr(), self.soa.len) }
            }
            )*
        }

        pub struct SoaIntoIter {
            buf: crate::Unique<u64>,
            cap: usize,
            $hdi: *const $hdt,
            end: *const $hdt,
            $($tli: *const $tlt,)*
        }

        impl Iterator for SoaIntoIter {
            type Item = El;

            fn next(&mut self) -> Option<Self::Item> {
                if self.$hdi == self.end {
                    None
                } else {
                    unsafe {
                        let out = El {
                            $hdi: self.$hdi.read(),
                            $($tli: self.$tli.read(),)*
                        };
                        self.$hdi = self.$hdi.offset(1);
                        $(self.$tli = self.$tli.offset(1);)*
                        Some(out)
                    }
                }
            }

            fn size_hint(&self) -> (usize, Option<usize>) {
                let len = (self.end as usize - self.$hdi as usize) / std::mem::size_of::<u64>();
                (len, Some(len))
            }
        }

        impl IntoIterator for Soa {
            type Item = El;

            type IntoIter = SoaIntoIter;

            fn into_iter(self) -> Self::IntoIter {
                let soa = std::mem::ManuallyDrop::new(self);
                unsafe {
                    SoaIntoIter {
                        buf: soa.$hdi,
                        cap: soa.cap,
                        $hdi: soa.$hdi.ptr.as_ptr(),
                        end: soa.$hdi.ptr.as_ptr().add(soa.len),
                        $($tli: soa.$tli.ptr.as_ptr(),)*
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
                        std::alloc::dealloc(self.buf.ptr.as_ptr() as *mut u8, layout);
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    soa! {foo: u64, bar: u8, baz: [u32; 2]}

    const A: El = El {
        foo: 20,
        bar: 10,
        baz: [6, 4],
    };

    const B: El = El {
        foo: 10,
        bar: 5,
        baz: [3, 2],
    };

    const ZERO: El = El {
        foo: 0,
        bar: 0,
        baz: [0, 0],
    };

    fn soa() -> Soa {
        let mut soa = Soa::new();
        soa.push(A);
        soa.push(B);
        soa
    }

    #[test]
    fn push_and_pop() {
        let mut soa = soa();
        assert_eq!(soa.pop(), Some(B));
        assert_eq!(soa.pop(), Some(A));
        assert_eq!(soa.pop(), None);
    }

    #[test]
    fn iterators() {
        let soa = soa();
        assert_eq!(soa.foo(), &[20, 10]);
        assert_eq!(soa.bar(), &[10, 5]);
        assert_eq!(soa.baz(), &[[6, 4], [3, 2]]);
    }

    #[test]
    fn insert_and_remove() {
        let mut soa = soa();
        soa.insert(1, ZERO);
        assert_eq!(soa.foo(), &[20, 0, 10]);
        assert_eq!(soa.bar(), &[10, 0, 5]);
        assert_eq!(soa.baz(), &[[6, 4], [0, 0], [3, 2]]);
        assert_eq!(soa.remove(1), ZERO);
        assert_eq!(soa.foo(), &[20, 10]);
        assert_eq!(soa.bar(), &[10, 5]);
        assert_eq!(soa.baz(), &[[6, 4], [3, 2]]);
    }

    #[test]
    fn into_iter() {
        {
            let mut soa = soa().into_iter();
            assert_eq!(soa.next(), Some(A));
            assert_eq!(soa.next(), Some(B));
            assert_eq!(soa.next(), None);
        }
    }
}

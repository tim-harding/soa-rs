use std::{
    alloc::{self, Layout},
    marker::PhantomData,
    ptr::NonNull,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct El {
    foo: u64,
    bar: u8,
    baz: [u32; 2],
}

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

pub struct Soa {
    foo: Unique<u64>,
    bar: Unique<u8>,
    baz: Unique<[u32; 2]>,
    len: usize,
    cap: usize,
}

impl Soa {
    pub const fn new() -> Self {
        Self {
            foo: Unique::dangling(),
            bar: Unique::dangling(),
            baz: Unique::dangling(),
            len: 0,
            cap: 0,
        }
    }

    fn resize(&mut self, cap: usize) {
        let (layout, offset1, offset2) = Self::layout_and_offsets(cap);

        let ptr = if self.cap == 0 {
            unsafe { alloc::alloc(layout) }
        } else {
            let (old_layout, _, _) = Self::layout_and_offsets(self.cap);
            let old_ptr = self.foo.ptr.as_ptr() as *mut u8;
            unsafe { alloc::realloc(old_ptr, old_layout, layout.size()) }
        };

        assert_ne!(ptr as *const u8, std::ptr::null());
        self.foo = unsafe { Unique::new(ptr) };
        self.bar = unsafe { Unique::new(ptr.add(offset1)) };
        self.baz = unsafe { Unique::new(ptr.add(offset2)) };
        self.cap = cap;
    }

    fn grow(&mut self) {
        let cap = if self.cap == 0 { 4 } else { self.cap * 2 };
        self.resize(cap);
    }

    fn layout_and_offsets(cap: usize) -> (Layout, usize, usize) {
        let layout = Layout::array::<u64>(cap).unwrap();
        let (layout, offset1) = layout.extend(Layout::array::<u8>(cap).unwrap()).unwrap();
        let (layout, offset2) = layout
            .extend(Layout::array::<[u32; 2]>(cap).unwrap())
            .unwrap();
        (layout, offset1, offset2)
    }

    pub fn push(&mut self, el: El) {
        if self.len == self.cap {
            self.grow();
        }

        unsafe {
            self.foo.ptr.as_ptr().add(self.len).write(el.foo);
            self.bar.ptr.as_ptr().add(self.len).write(el.bar);
            self.baz.ptr.as_ptr().add(self.len).write(el.baz);
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
                    foo: self.foo.ptr.as_ptr().add(self.len).read(),
                    bar: self.bar.ptr.as_ptr().add(self.len).read(),
                    baz: self.baz.ptr.as_ptr().add(self.len).read(),
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
            let foo = self.foo.ptr.as_ptr();
            let bar = self.bar.ptr.as_ptr();
            let baz = self.baz.ptr.as_ptr();
            std::ptr::copy(foo.add(index), foo.add(index + 1), self.len - index);
            std::ptr::copy(bar.add(index), bar.add(index + 1), self.len - index);
            std::ptr::copy(baz.add(index), baz.add(index + 1), self.len - index);
            foo.add(index).write(el.foo);
            bar.add(index).write(el.bar);
            baz.add(index).write(el.baz);
        }
    }

    pub fn remove(&mut self, index: usize) -> El {
        assert!(index <= self.len, "index out of bounds");
        self.len -= 1;
        unsafe {
            let foo = self.foo.ptr.as_ptr();
            let bar = self.bar.ptr.as_ptr();
            let baz = self.baz.ptr.as_ptr();
            let out = El {
                foo: foo.add(index).read(),
                bar: bar.add(index).read(),
                baz: baz.add(index).read(),
            };
            std::ptr::copy(foo.add(index + 1), foo.add(index), self.len - index);
            std::ptr::copy(bar.add(index + 1), bar.add(index), self.len - index);
            std::ptr::copy(baz.add(index + 1), baz.add(index), self.len - index);
            out
        }
    }

    pub fn foo(&self) -> &[u64] {
        unsafe { std::slice::from_raw_parts(self.foo.ptr.as_ptr(), self.len) }
    }

    pub fn foo_mut(&mut self) -> &mut [u64] {
        unsafe { std::slice::from_raw_parts_mut(self.foo.ptr.as_ptr(), self.len) }
    }

    pub fn bar(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts(self.bar.ptr.as_ptr(), self.len) }
    }

    pub fn bar_mut(&mut self) -> &mut [u8] {
        unsafe { std::slice::from_raw_parts_mut(self.bar.ptr.as_ptr(), self.len) }
    }

    pub fn baz(&self) -> &[[u32; 2]] {
        unsafe { std::slice::from_raw_parts(self.baz.ptr.as_ptr(), self.len) }
    }

    pub fn baz_mut(&mut self) -> &mut [[u32; 2]] {
        unsafe { std::slice::from_raw_parts_mut(self.baz.ptr.as_ptr(), self.len) }
    }
}

impl Drop for Soa {
    fn drop(&mut self) {
        if self.cap > 0 {
            while let Some(_) = self.pop() {}
            let (layout, _, _) = Self::layout_and_offsets(self.cap);
            unsafe {
                alloc::dealloc(self.foo.ptr.as_ptr() as *mut u8, layout);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}

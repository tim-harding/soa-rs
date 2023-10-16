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
        let layout = Layout::array::<u64>(cap).unwrap();
        let (layout, offset1) = layout.extend(Layout::array::<u8>(cap).unwrap()).unwrap();
        let (layout, offset2) = layout
            .extend(Layout::array::<[u32; 2]>(cap).unwrap())
            .unwrap();

        let ptr = if self.cap == 0 {
            unsafe { alloc::alloc(layout) }
        } else {
            let old_layout = Layout::array::<u64>(self.cap)
                .unwrap()
                .extend(Layout::array::<u8>(self.cap).unwrap())
                .unwrap()
                .0
                .extend(Layout::array::<[u32; 2]>(self.cap).unwrap())
                .unwrap()
                .0;
            let old_ptr = self.foo.ptr.as_ptr() as *mut u8;
            unsafe { alloc::realloc(old_ptr, old_layout, layout.size()) }
        };

        assert_ne!(ptr as *const u8, std::ptr::null());
        self.foo = unsafe { Unique::new(ptr) };
        self.bar = unsafe { Unique::new(ptr.add(offset1)) };
        self.baz = unsafe { Unique::new(ptr.add(offset2)) };
        self.cap = cap;
    }

    pub fn push(&mut self, el: El) {
        if self.len == self.cap {
            let cap = if self.cap == 0 { 4 } else { self.cap * 2 };
            self.resize(cap);
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn push_and_pop() {
        let a = El {
            foo: 20,
            bar: 10,
            baz: [6, 4],
        };
        let b = El {
            foo: 10,
            bar: 5,
            baz: [3, 2],
        };
        let mut soa = Soa::new();
        soa.push(a);
        soa.push(b);
        assert_eq!(soa.pop(), Some(b));
        assert_eq!(soa.pop(), Some(a));
        assert_eq!(soa.pop(), None);
    }
}

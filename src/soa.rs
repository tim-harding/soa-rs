// TODO: ZSTs

use soapy_shared::{SoaRaw, Soapy};

pub struct Soa<T>
where
    T: Soapy,
{
    len: usize,
    capacity: usize,
    raw: T::SoaRaw,
}

impl<T> Soa<T>
where
    T: Soapy,
{
    pub fn new() -> Self {
        Self {
            len: 0,
            capacity: 0,
            raw: T::SoaRaw::new(),
        }
    }

    pub fn push(&mut self, element: T) {
        self.maybe_grow();
        unsafe {
            self.raw.set(self.len, element);
        }
        self.len += 1;
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.len == 0 {
            None
        } else {
            self.len -= 1;
            Some(unsafe { self.raw.get(self.len) })
        }
    }

    pub fn insert(&mut self, index: usize, element: T) {
        assert!(index <= self.len, "index out of bounds");
        self.maybe_grow();
        unsafe {
            self.raw.copy(index, index + 1, self.len - index);
            self.raw.set(index, element);
        }
        self.len += 1;
    }

    pub fn remove(&mut self, index: usize) -> T {
        assert!(index < self.len, "index out of bounds");
        self.len -= 1;
        unsafe {
            let out = self.raw.get(index);
            self.raw.copy(index + 1, index, self.len - index);
            out
        }
    }

    fn maybe_grow(&mut self) {
        if self.len < self.capacity {
            return;
        }
        let new_capacity = match self.capacity {
            0 => 4,
            cap => cap * 2,
        };
        unsafe {
            self.raw.grow(self.capacity, new_capacity, self.len);
        }
        self.capacity = new_capacity;
    }
}

impl<T> Drop for Soa<T>
where
    T: Soapy,
{
    fn drop(&mut self) {
        if self.capacity == 0 {
            return;
        }
        while let Some(_) = self.pop() {}
        unsafe { self.raw.dealloc(self.capacity) };
    }
}

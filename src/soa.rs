use soapy_shared::{SoaRaw, Soapy};
use std::mem::{size_of, ManuallyDrop};

pub struct Soa<T>
where
    T: Soapy,
{
    len: usize,
    cap: usize,
    raw: T::SoaRaw,
}

impl<T> Soa<T>
where
    T: Soapy,
{
    pub fn new() -> Self {
        Self {
            len: 0,
            cap: if size_of::<T>() == 0 { usize::MAX } else { 0 },
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
        if self.len < self.cap {
            return;
        }
        let new_capacity = match self.cap {
            0 => 4,
            cap => cap * 2,
        };
        unsafe {
            self.raw.grow(self.cap, new_capacity, self.len);
        }
        self.cap = new_capacity;
    }
}

impl<T> Drop for Soa<T>
where
    T: Soapy,
{
    fn drop(&mut self) {
        while let Some(_) = self.pop() {}
        unsafe { self.raw.dealloc(self.cap) };
    }
}

impl<T> IntoIterator for Soa<T>
where
    T: Soapy,
{
    type Item = T;

    type IntoIter = IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        // Make sure not to drop self and free the buffer
        let soa = ManuallyDrop::new(self);

        let len = soa.len;
        let cap = soa.cap;
        let raw = soa.raw;

        IntoIter {
            start: 0,
            end: len,
            raw,
            cap,
        }
    }
}

pub struct IntoIter<T>
where
    T: Soapy,
{
    raw: T::SoaRaw,
    cap: usize,
    start: usize,
    end: usize,
}

impl<T> Iterator for IntoIter<T>
where
    T: Soapy,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.start >= self.end {
            None
        } else {
            let out = unsafe { self.raw.get(self.start) };
            self.start += 1;
            Some(out)
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.end - self.start;
        (len, Some(len))
    }
}

impl<T> Drop for IntoIter<T>
where
    T: Soapy,
{
    fn drop(&mut self) {
        while let Some(_) = self.next() {}
        unsafe { self.raw.dealloc(self.cap) };
    }
}

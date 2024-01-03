use soapy_shared::{RawSoa, Soapy};
use std::mem::{size_of, ManuallyDrop};

pub struct Soa<T>
where
    T: Soapy,
{
    len: usize,
    cap: usize,
    raw: T::RawSoa,
}

impl<T> Soa<T>
where
    T: Soapy,
{
    const INIT_CAP: usize = if size_of::<T>() == 0 { usize::MAX } else { 0 };

    pub fn new() -> Self {
        Self {
            len: 0,
            cap: Self::INIT_CAP,
            raw: T::RawSoa::dangling(),
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        match capacity {
            0 => Self::new(),
            capacity => Self {
                len: 0,
                cap: capacity,
                raw: unsafe { T::RawSoa::alloc(capacity) },
            },
        }
    }

    /// Decomposes a `Soa<T>` into its raw components.
    ///
    /// Returns the raw pointer to the underlying data, the length of the vector (in
    /// elements), and the allocated capacity of the data (in elements). These
    /// are the same arguments in the same order as the arguments to
    /// [`Soa::from_raw_parts`].
    ///
    /// After calling this function, the caller is responsible for the memory
    /// previously managed by the `Soa`. The only way to do this is to convert the
    /// raw pointer, length, and capacity back into a Vec with the
    /// [`Soa::from_raw_parts`] function, allowing the destructor to perform the cleanup.
    pub fn into_raw_parts(self) -> (*mut u8, usize, usize) {
        let Self { len, cap, raw } = self;
        (raw.as_ptr(), len, cap)
    }

    /// Creates a `Soa<T>` from a pointer, a length, and a capacity.
    ///
    /// # Safety
    ///
    /// This is highly unsafe due to the number of invariants that aren't
    /// checked. Given that many of these invariants are private implementation
    /// details of [`RawSoa`], it is better not to uphold them manually. Rather,
    /// it only valid to call this method with the output of a previous call to
    /// [`Soa::into_raw_parts`].
    pub unsafe fn from_raw_parts(ptr: *mut u8, length: usize, capacity: usize) -> Self {
        Self {
            len: length,
            cap: capacity,
            raw: T::RawSoa::from_parts(ptr, capacity),
        }
    }

    pub fn capacity(&self) -> usize {
        self.cap
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
        if self.len < self.cap || size_of::<T>() == 0 {
            return;
        }

        match self.cap {
            0 => {
                const INIT_CAP: usize = 4;
                self.cap = INIT_CAP;
                unsafe {
                    self.raw = T::RawSoa::alloc(INIT_CAP);
                }
            }
            old_cap => {
                self.cap = old_cap * 2;
                unsafe {
                    self.raw.realloc_grow(old_cap, self.cap, self.len);
                }
            }
        }
    }
}

impl<T> Drop for Soa<T>
where
    T: Soapy,
{
    fn drop(&mut self) {
        while let Some(_) = self.pop() {}
        dealloc(&mut self.raw, self.cap);
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
    raw: T::RawSoa,
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
        dealloc(&mut self.raw, self.cap);
    }
}

fn dealloc<T>(raw: &mut impl RawSoa<T>, old_capacity: usize) {
    if size_of::<T>() > 0 && old_capacity > 0 {
        unsafe {
            raw.dealloc(old_capacity);
        }
    }
}

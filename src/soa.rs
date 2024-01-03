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
    const SMALL_CAPACITY: usize = 4;

    /// Constructs a new, empty `Soa<T>`.
    ///
    /// The container will not allocate until elements are pushed onto it.
    pub fn new() -> Self {
        Self {
            len: 0,
            cap: if size_of::<T>() == 0 { usize::MAX } else { 0 },
            raw: T::RawSoa::dangling(),
        }
    }

    /// Construct a new, empty `Soa<T>` with the specified capacity.
    ///
    /// The container will be able to hold `capacity` elements without
    /// reallocating. If the `capacity` is 0, the container will not allocate.
    /// Note that although the returned vector has the minimum capacity
    /// specified, the vector will have a zero length.
    pub fn with_capacity(capacity: usize) -> Self {
        match capacity {
            0 => Self::new(),
            capacity => {
                if size_of::<T>() == 0 {
                    Self {
                        len: 0,
                        cap: usize::MAX,
                        raw: T::RawSoa::dangling(),
                    }
                } else {
                    Self {
                        len: 0,
                        cap: capacity,
                        raw: unsafe { T::RawSoa::alloc(capacity) },
                    }
                }
            }
        }
    }

    /// Returns the number of elements in the vector, also referred to as its
    /// length.
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns the total number of elements the container can hold without
    /// reallocating.
    pub fn capacity(&self) -> usize {
        self.cap
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

    /// Appends an element to the back of a collection.
    pub fn push(&mut self, element: T) {
        self.maybe_grow();
        unsafe {
            self.raw.set(self.len, element);
        }
        self.len += 1;
    }

    /// Removes the last element from a vector and returns it, or [`None`] if it
    /// is empty.
    pub fn pop(&mut self) -> Option<T> {
        if self.len == 0 {
            None
        } else {
            self.len -= 1;
            Some(unsafe { self.raw.get(self.len) })
        }
    }

    /// Inserts an element at position `index`, shifting all elements after it
    /// to the right.
    ///
    /// # Panics
    ///
    /// Panics if `index > len`
    pub fn insert(&mut self, index: usize, element: T) {
        assert!(index <= self.len, "index out of bounds");
        self.maybe_grow();
        unsafe {
            self.raw.copy(index, index + 1, self.len - index);
            self.raw.set(index, element);
        }
        self.len += 1;
    }

    /// Removes and returns the element at position index within the vector,
    /// shifting all elements after it to the left.
    pub fn remove(&mut self, index: usize) -> T {
        assert!(index < self.len, "index out of bounds");
        self.len -= 1;
        unsafe {
            let out = self.raw.get(index);
            self.raw.copy(index + 1, index, self.len - index);
            out
        }
    }

    /// Reserves capacity for at least additional more elements to be inserted
    /// in the given `Soa<T>`. The collection may reserve more space to
    /// speculatively avoid frequent reallocations. After calling reserve,
    /// capacity will be greater than or equal to `self.len() + additional`. Does
    /// nothing if capacity is already sufficient.
    pub fn reserve(&mut self, additional: usize) {
        if additional == 0 {
            return;
        }
        let new_cap = (self.len + additional)
            // Ensure exponential growth
            .max(self.cap * 2)
            .max(Self::SMALL_CAPACITY);
        self.grow(new_cap);
    }

    /// Reserves the minimum capacity for at least additional more elements to
    /// be inserted in the given `Soa<T>`. Unlike [`Soa::reserve`], this will not
    /// deliberately over-allocate to speculatively avoid frequent allocations.
    /// After calling `reserve_exact`, capacity will be greater than or equal to
    /// self.len() + additional. Does nothing if the capacity is already
    /// sufficient.
    pub fn reserve_exact(&mut self, additional: usize) {
        if additional == 0 {
            return;
        }
        let new_cap = (additional + self.len).max(self.cap);
        self.grow(new_cap);
    }

    /// Grows the allocated capacity if `len == cap`
    fn maybe_grow(&mut self) {
        if self.len < self.cap {
            return;
        }
        let new_cap = match self.cap {
            0 => Self::SMALL_CAPACITY,
            old_cap => old_cap * 2,
        };
        self.grow(new_cap);
    }

    fn grow(&mut self, new_cap: usize) {
        debug_assert!(new_cap > self.cap);
        match self.cap {
            0 => {
                self.raw = unsafe { T::RawSoa::alloc(new_cap) };
            }
            old_cap => unsafe {
                self.raw.realloc_grow(old_cap, new_cap, self.len);
            },
        }
        self.cap = new_cap;
    }
}

impl<T> Drop for Soa<T>
where
    T: Soapy,
{
    fn drop(&mut self) {
        while let Some(_) = self.pop() {}
        drop_raw(&mut self.raw, self.cap);
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
        drop_raw(&mut self.raw, self.cap);
    }
}

fn drop_raw<T>(raw: &mut impl RawSoa<T>, old_capacity: usize) {
    if size_of::<T>() > 0 && old_capacity > 0 {
        unsafe {
            raw.dealloc(old_capacity);
        }
    }
}

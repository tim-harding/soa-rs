use crate::{index::SoaIndex, IntoIter, Iter, IterMut};
use soapy_shared::{RawSoa, Soapy};
use std::{
    cmp::Ordering,
    fmt::{self, Formatter},
    marker::PhantomData,
    mem::{size_of, ManuallyDrop},
    ops::{ControlFlow, Deref},
};

/// A growable array type that stores the values for each field of `T`
/// contiguously.
pub struct Soa<T>
where
    T: Soapy,
{
    pub(crate) len: usize,
    pub(crate) cap: usize,
    pub(crate) raw: T::RawSoa,
}

unsafe impl<T> Send for Soa<T> where T: Send + Soapy {}
unsafe impl<T> Sync for Soa<T> where T: Sync + Soapy {}

impl<T> Soa<T>
where
    T: Soapy,
{
    /// The capacity of the initial allocation. This is an optimization to avoid
    /// excessive reallocation for small array sizes.
    const SMALL_CAPACITY: usize = 4;

    /// Constructs a new, empty `Soa<T>`.
    ///
    /// The container will not allocate until elements are pushed onto it.
    ///
    /// # Examples
    /// ```
    /// # use soapy::{Soa, Soapy};
    /// # #[derive(Soapy)]
    /// # struct Foo;
    /// let mut soa: Soa<Foo> = Soa::new();
    /// ```
    pub fn new() -> Self {
        Self {
            len: 0,
            cap: if size_of::<T>() == 0 { usize::MAX } else { 0 },
            raw: T::RawSoa::dangling(),
        }
    }

    /// Construct a new, empty `Soa<T>` with at least the specified capacity.
    ///
    /// The container will be able to hold `capacity` elements without
    /// reallocating. If the `capacity` is 0, the container will not allocate.
    /// Note that although the returned vector has the minimum capacity
    /// specified, the vector will have a zero length. The capacity will be as
    /// specified unless `T` is zero-sized, in which case the capacity will be
    /// `usize::MAX`.
    ///
    /// # Examples
    /// ```
    /// # use soapy::{Soa, Soapy};
    /// #[derive(Soapy)]
    /// struct Foo(u8, u8);
    ///
    /// let mut soa = Soa::with_capacity(10);
    /// assert_eq!(soa.len(), 0);
    /// assert_eq!(soa.capacity(), 10);
    ///
    /// // These pushes do not reallocate...
    /// for i in 0..10 {
    ///     soa.push(Foo(i, i));
    /// }
    /// assert_eq!(soa.len(), 10);
    /// assert_eq!(soa.capacity(), 10);
    ///
    /// // ...but this one does
    /// soa.push(Foo(11, 11));
    /// assert_eq!(soa.len(), 11);
    /// assert_eq!(soa.capacity(), 20);
    ///
    /// #[derive(Soapy)]
    /// struct Bar;
    ///
    /// // A SOA of a zero-sized type always over-allocates
    /// let soa: Soa<Bar> = Soa::with_capacity(10);
    /// assert_eq!(soa.capacity(), usize::MAX);
    /// ```
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
    ///
    /// # Examples
    ///
    /// ```
    /// # use soapy::{Soa, Soapy, soa};
    /// # #[derive(Soapy)]
    /// # struct Foo(usize);
    /// let soa = soa![Foo(1), Foo(2), Foo(3)];
    /// assert_eq!(soa.len(), 3);
    /// ```
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns true if the container contains no elements.
    ///
    /// # Examples
    ///
    /// ```
    /// # use soapy::{Soa, Soapy};
    /// # #[derive(Soapy)]
    /// # struct Foo(usize);
    /// let mut soa = Soa::new();
    /// assert!(soa.is_empty());
    /// soa.push(Foo(1));
    /// assert!(!soa.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Returns the total number of elements the container can hold without
    /// reallocating.
    ///
    /// # Examples
    ///
    /// ```
    /// # use soapy::{Soa, Soapy};
    /// # #[derive(Soapy)]
    /// # struct Foo(usize);
    /// let mut soa = Soa::new();
    /// for i in 0..42 {
    ///     assert!(soa.capacity() >= i);
    ///     soa.push(Foo(i));
    /// }
    /// ```
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
    ///
    /// # Examples
    ///
    /// ```
    /// # use soapy::{Soa, Soapy, soa};
    /// # #[derive(Soapy, Debug, PartialEq)]
    /// # struct Foo(usize);
    /// let soa = soa![Foo(1), Foo(2)];
    /// let (ptr, len, cap) = soa.into_raw_parts();
    /// let rebuilt = unsafe { Soa::from_raw_parts(ptr, len, cap) };
    /// assert_eq!(rebuilt, [Foo(1), Foo(2)]);
    /// ```
    pub fn into_raw_parts(self) -> (*mut u8, usize, usize) {
        let me = ManuallyDrop::new(self);
        (me.raw.as_ptr(), me.len, me.cap)
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
    ///
    /// # Examples
    ///
    /// ```
    /// # use soapy::{Soa, Soapy, soa};
    /// # #[derive(Soapy, Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
    /// # struct Foo(usize);
    /// let mut soa = soa![Foo(1), Foo(2)];
    /// soa.push(Foo(3));
    /// assert_eq!(soa, [Foo(1), Foo(2), Foo(3)]);
    /// ```
    pub fn push(&mut self, element: T) {
        self.maybe_grow();
        unsafe {
            self.raw.set(self.len, element);
        }
        self.len += 1;
    }

    /// Removes the last element from a vector and returns it, or [`None`] if it
    /// is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// # use soapy::{Soa, Soapy, soa};
    /// # #[derive(Soapy, Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
    /// # struct Foo(usize);
    /// let mut soa = soa![Foo(1), Foo(2), Foo(3)];
    /// assert_eq!(soa.pop(), Some(Foo(3)));
    /// assert_eq!(soa, [Foo(1), Foo(2)]);
    /// ```
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
    ///
    /// # Examples
    ///
    /// ```
    /// # use soapy::{Soa, Soapy, soa};
    /// # #[derive(Soapy, Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
    /// # struct Foo(usize);
    /// let mut soa = soa![Foo(1), Foo(2), Foo(3)];
    /// soa.insert(1, Foo(4));
    /// assert_eq!(soa, [Foo(1), Foo(4), Foo(2), Foo(3)]);
    /// soa.insert(4, Foo(5));
    /// assert_eq!(soa, [Foo(1), Foo(4), Foo(2), Foo(3), Foo(5)]);
    /// ```
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
    ///
    /// # Examples
    ///
    /// ```
    /// # use soapy::{Soa, Soapy, soa};
    /// # #[derive(Soapy, Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
    /// # struct Foo(usize);
    /// let mut soa = soa![Foo(1), Foo(2), Foo(3)];
    /// assert_eq!(soa.remove(1), Foo(2));
    /// assert_eq!(soa, [Foo(1), Foo(3)])
    /// ```
    pub fn remove(&mut self, index: usize) -> T {
        assert!(index < self.len, "index out of bounds");
        self.len -= 1;
        let out = unsafe { self.raw.get(index) };
        unsafe {
            self.raw.copy(index + 1, index, self.len - index);
        }
        out
    }

    /// Reserves capacity for at least additional more elements to be inserted
    /// in the given `Soa<T>`. The collection may reserve more space to
    /// speculatively avoid frequent reallocations. After calling reserve,
    /// capacity will be greater than or equal to `self.len() + additional`.
    /// Does nothing if capacity is already sufficient.
    ///
    /// # Examples
    ///
    /// ```
    /// # use soapy::{Soa, Soapy, soa};
    /// # #[derive(Soapy, Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
    /// # struct Foo(usize);
    /// let mut soa = soa![Foo(1)];
    /// soa.reserve(10);
    /// assert!(soa.capacity() >= 11);
    /// ```
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
    /// be inserted in the given `Soa<T>`. Unlike [`Soa::reserve`], this will
    /// not deliberately over-allocate to speculatively avoid frequent
    /// allocations. After calling `reserve_exact`, capacity will be equal to
    /// self.len() + additional, or else `usize::MAX` if `T` is zero-sized. Does
    /// nothing if the capacity is already sufficient.
    ///
    /// # Examples
    ///
    /// ```
    /// # use soapy::{Soa, Soapy, soa};
    /// # #[derive(Soapy, Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
    /// # struct Foo(usize);
    /// let mut soa = soa![Foo(1)];
    /// soa.reserve(10);
    /// assert!(soa.capacity() == 11);
    /// ```
    pub fn reserve_exact(&mut self, additional: usize) {
        if additional == 0 {
            return;
        }
        let new_cap = (additional + self.len).max(self.cap);
        self.grow(new_cap);
    }

    /// Shrinks the capacity of the container as much as possible.
    ///
    /// # Examples
    ///
    /// ```
    /// # use soapy::{Soa, Soapy, soa};
    /// # #[derive(Soapy, Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
    /// # struct Foo(usize);
    /// let mut soa = Soa::with_capacity(10);
    /// soa.extend([Foo(1), Foo(2), Foo(3)]);
    /// assert_eq!(soa.capacity(), 10);
    /// soa.shrink_to_fit();
    /// assert_eq!(soa.capacity(), 3);
    /// ```
    pub fn shrink_to_fit(&mut self) {
        self.shrink(self.len);
    }

    /// Shrinks the capacity of the vector with a lower bound.
    ///
    /// The capacity will remain at least as large as both the length and the
    /// supplied value. If the current capacity is less than the lower limit,
    /// this is a no-op.
    ///
    /// # Examples
    ///
    /// ```
    /// # use soapy::{Soa, Soapy, soa};
    /// # #[derive(Soapy, Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
    /// # struct Foo(usize);
    /// let mut soa = Soa::with_capacity(10);
    /// soa.extend([Foo(1), Foo(2), Foo(3)]);
    /// assert_eq!(soa.capacity(), 10);
    /// soa.shrink_to(4);
    /// assert_eq!(soa.capacity(), 4);
    /// soa.shrink_to(0);
    /// assert_eq!(soa.capacity(), 3);
    pub fn shrink_to(&mut self, min_capacity: usize) {
        let new_cap = self.len.max(min_capacity);
        if new_cap < self.cap {
            self.shrink(new_cap);
        }
    }

    /// Shortens the vector, keeping the first len elements and dropping the rest.
    ///
    /// If len is greater or equal to the vector’s current length, this has no
    /// effect. Note that this method has no effect on the allocated capacity of
    /// the vector.
    ///
    /// # Examples
    ///
    /// Truncating a five-element SOA to two elements:
    /// ```
    /// # use soapy::{Soa, Soapy, soa};
    /// # #[derive(Soapy, Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
    /// # struct Foo(usize);
    /// let mut soa = soa![Foo(1), Foo(2), Foo(3), Foo(4), Foo(5)];
    /// soa.truncate(2);
    /// assert_eq!(soa, [Foo(1), Foo(2)]);
    /// ```
    ///
    /// No truncation occurs when `len` is greater than the SOA's current
    /// length:
    /// ```
    /// # use soapy::{Soa, Soapy, soa};
    /// # #[derive(Soapy, Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
    /// # struct Foo(usize);
    /// let mut soa = soa![Foo(1), Foo(2), Foo(3)];
    /// soa.truncate(8);
    /// assert_eq!(soa, [Foo(1), Foo(2), Foo(3)]);
    /// ```
    ///
    /// Truncating with `len == 0` is equivalent to [`Soa::clear`].
    /// ```
    /// # use soapy::{Soa, Soapy, soa};
    /// # #[derive(Soapy, Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
    /// # struct Foo(usize);
    /// let mut soa = soa![Foo(1), Foo(2), Foo(3)];
    /// soa.truncate(0);
    /// assert_eq!(soa, []);
    /// ```
    pub fn truncate(&mut self, len: usize) {
        while len < self.len {
            self.pop();
        }
    }

    /// Removes an element from the vector and returns it.
    ///
    /// The removed element is replaced by the last element of the vector. This
    /// does not preserve ordering, but is O(1). If you need to preserve the
    /// element order, use remove instead.
    ///
    /// # Panics
    ///
    /// Panics if index is out of bounds.
    ///
    /// # Examples
    ///
    /// ```
    /// # use soapy::{Soa, Soapy, soa};
    /// # #[derive(Soapy, Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
    /// # struct Foo(usize);
    /// let mut soa = soa![Foo(0), Foo(1), Foo(2), Foo(3)];
    ///
    /// assert_eq!(soa.swap_remove(1), Foo(1));
    /// assert_eq!(soa, [Foo(0), Foo(3), Foo(2)]);
    ///
    /// assert_eq!(soa.swap_remove(0), Foo(0));
    /// assert_eq!(soa, [Foo(2), Foo(3)])
    /// ```
    pub fn swap_remove(&mut self, index: usize) -> T {
        let out = unsafe { self.raw.get(index) };
        let last = unsafe { self.raw.get(self.len - 1) };
        unsafe {
            self.raw.set(index, last);
        }
        self.len -= 1;
        out
    }

    /// Moves all the elements of other into self, leaving other empty.
    pub fn append(&mut self, other: &mut Self) {
        for i in 0..other.len {
            let element = unsafe { other.raw.get(i) };
            self.push(element);
        }
        other.len = 0;
    }

    /// Returns an iterator over the elements.
    ///
    /// The iterator yields all items from start to end.
    pub fn iter(&self) -> Iter<T> {
        Iter {
            raw: self.raw,
            start: 0,
            end: self.len,
            _marker: PhantomData,
        }
    }

    /// Returns an iterator over the elements that allows modifying each value.
    ///
    /// The iterator yields all items from start to end.
    pub fn iter_mut(&mut self) -> IterMut<T> {
        IterMut {
            raw: self.raw,
            start: 0,
            end: self.len,
            _marker: PhantomData,
        }
    }

    pub fn try_fold<F, B>(&self, init: B, mut f: F) -> B
    where
        F: FnMut(B, &T) -> ControlFlow<B, B>,
    {
        let mut acc = init;
        for i in 0..self.len {
            // SAFETY:
            // Okay to construct an element and take its reference, so long as
            // we don't run its destructor.
            let element = ManuallyDrop::new(unsafe { self.raw.get(i) });
            let result = f(acc, &element);
            match result {
                ControlFlow::Continue(b) => acc = b,
                ControlFlow::Break(b) => return b,
            }
        }
        acc
    }

    pub fn try_fold_zip<F, B, O>(&self, other: &Soa<O>, init: B, mut f: F) -> B
    where
        O: Soapy,
        F: FnMut(B, &T, &O) -> ControlFlow<B, B>,
    {
        let mut acc = init;
        let len = self.len.min(other.len);
        for i in 0..len {
            // SAFETY:
            // Okay to construct an element and take its reference, so long as
            // we don't run its destructor.
            let a = ManuallyDrop::new(unsafe { self.raw.get(i) });
            let b = ManuallyDrop::new(unsafe { other.raw.get(i) });
            let result = f(acc, &a, &b);
            match result {
                ControlFlow::Continue(b) => acc = b,
                ControlFlow::Break(b) => return b,
            }
        }
        acc
    }

    /// Calls a closure on each element of the collection.
    pub fn for_each<F>(&self, mut f: F)
    where
        F: FnMut(&T),
    {
        self.try_fold((), |_, item| {
            f(item);
            ControlFlow::Continue(())
        })
    }

    /// Calls a closure on each element of the collection.
    pub fn for_each_zip<F, O>(&self, other: &Soa<O>, mut f: F)
    where
        O: Soapy,
        F: FnMut(&T, &O),
    {
        self.try_fold_zip(other, (), |_, a, b| {
            f(a, b);
            ControlFlow::Continue(())
        })
    }

    /// Clears the vector, removing all values.
    ///
    /// Note that this method has no effect on the allocated capacity of the
    /// vector.
    pub fn clear(&mut self) {
        while self.pop().is_some() {}
    }

    /// Returns a reference to an element or subslice depending on the type of
    /// index.
    pub fn get<I>(&self, index: I) -> Option<I::Output<'_>>
    where
        I: SoaIndex<T>,
    {
        index.get(self)
    }

    /// Returns a clone of the element at the given index.
    ///
    /// # Panics
    ///
    /// Panics if `index >= len`.
    pub fn nth_cloned(&self, index: usize) -> T
    where
        T: Clone,
    {
        if index >= self.len {
            panic!("index out of bounds");
        }
        let el = ManuallyDrop::new(unsafe { self.raw.get(index) });
        el.deref().clone()
    }

    /// Returns a reference to the element at the given index.
    ///
    /// # Panics
    ///
    /// Panics if `index >= len`.
    pub fn nth(&self, index: usize) -> <T::RawSoa as RawSoa<T>>::Ref<'_> {
        if index >= self.len {
            panic!("index out of bounds");
        }
        unsafe { self.raw.get_ref(index) }
    }

    /// Returns slices for each of the SoA fields.
    pub fn slices(&self) -> <T::RawSoa as RawSoa<T>>::Slices<'_> {
        unsafe { self.raw.slices(0, self.len) }
    }

    /// Returns a mutable reference to an element or subslice depending on the
    /// type of index.
    pub fn get_mut<I>(&mut self, index: I) -> Option<I::OutputMut<'_>>
    where
        I: SoaIndex<T>,
    {
        index.get_mut(self)
    }

    /// Returns a reference to the element at the given index.
    ///
    /// # Panics
    ///
    /// Panics if `index >= len`.
    pub fn nth_mut(&mut self, index: usize) -> <T::RawSoa as RawSoa<T>>::RefMut<'_> {
        if index >= self.len {
            panic!("index out of bounds");
        }
        unsafe { self.raw.get_mut(index) }
    }

    /// Returns mutable slices for each of the SoA fields.
    pub fn slices_mut(&mut self) -> <T::RawSoa as RawSoa<T>>::SlicesMut<'_> {
        unsafe { self.raw.slices_mut(0, self.len) }
    }

    /// Swaps the position of two elements.
    ///
    /// # Panics
    ///
    /// Panics if `a` or `b` is out of bounds.
    pub fn swap(&mut self, a: usize, b: usize) {
        if a >= self.len || b >= self.len {
            panic!("index out of bounds");
        }

        unsafe {
            let tmp = self.raw.get(a);
            self.raw.copy(b, a, 1);
            self.raw.set(b, tmp);
        }
    }

    /// Grows the allocated capacity if `len == cap`.
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

    // Shrinks the allocated capacity.
    fn shrink(&mut self, new_cap: usize) {
        debug_assert!(new_cap <= self.cap);
        if self.cap == 0 || new_cap == self.cap || size_of::<T>() == 0 {
            return;
        }

        if new_cap == 0 {
            debug_assert!(self.cap > 0);
            unsafe {
                self.raw.dealloc(self.cap);
            }
            self.raw = T::RawSoa::dangling();
        } else {
            debug_assert!(new_cap < self.cap);
            debug_assert!(self.len <= new_cap);
            unsafe {
                self.raw.realloc_shrink(self.cap, new_cap, self.len);
            }
        }

        self.cap = new_cap;
    }

    /// Grows the allocated capacity.
    fn grow(&mut self, new_cap: usize) {
        debug_assert!(size_of::<T>() > 0);
        debug_assert!(new_cap > self.cap);

        if self.cap == 0 {
            debug_assert!(new_cap > 0);
            self.raw = unsafe { T::RawSoa::alloc(new_cap) };
        } else {
            debug_assert!(self.len <= self.cap);
            unsafe {
                self.raw.realloc_grow(self.cap, new_cap, self.len);
            }
        }

        self.cap = new_cap;
    }
}

impl<T> Drop for Soa<T>
where
    T: Soapy,
{
    fn drop(&mut self) {
        while self.pop().is_some() {}
        if size_of::<T>() > 0 && self.cap > 0 {
            unsafe {
                self.raw.dealloc(self.cap);
            }
        }
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

impl<T> Clone for Soa<T>
where
    T: Soapy + Clone,
{
    fn clone(&self) -> Self {
        let mut out = Self::with_capacity(self.len());
        self.for_each(|el| {
            out.push(el.clone());
        });
        out
    }

    fn clone_from(&mut self, source: &Self) {
        self.clear();
        if self.cap < source.len {
            self.reserve(source.len);
        }
        source.for_each(|el| {
            self.push(el.clone());
        });
    }
}

impl<T> Extend<T> for Soa<T>
where
    T: Soapy,
{
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        for item in iter {
            self.push(item);
        }
    }
}

impl<T> FromIterator<T> for Soa<T>
where
    T: Soapy,
{
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let iter = iter.into_iter();
        let (hint_min, hint_max) = iter.size_hint();
        let cap = hint_max.unwrap_or(hint_min);
        let mut out = Self::with_capacity(cap);
        for item in iter {
            out.push(item);
        }
        out
    }
}

impl<T, const N: usize> From<[T; N]> for Soa<T>
where
    T: Soapy,
{
    /// Allocate a `Soa<T>` and move `value`'s items into it.
    fn from(value: [T; N]) -> Self {
        value.into_iter().collect()
    }
}

impl<T, const N: usize> From<&[T; N]> for Soa<T>
where
    T: Soapy + Clone,
{
    /// Allocate a `Soa<T>` and fill it by cloning `value`'s items.
    fn from(value: &[T; N]) -> Self {
        value.iter().cloned().collect()
    }
}

impl<T, const N: usize> From<&mut [T; N]> for Soa<T>
where
    T: Soapy + Clone,
{
    /// Allocate a `Soa<T>` and fill it by cloning `value`'s items.
    fn from(value: &mut [T; N]) -> Self {
        value.iter().cloned().collect()
    }
}

impl<T> From<&[T]> for Soa<T>
where
    T: Soapy + Clone,
{
    /// Allocate a `Soa<T>` and fill it by cloning `value`'s items.
    fn from(value: &[T]) -> Self {
        value.iter().cloned().collect()
    }
}

impl<T> From<&mut [T]> for Soa<T>
where
    T: Soapy + Clone,
{
    /// Allocate a `Soa<T>` and fill it by cloning `value`'s items.
    fn from(value: &mut [T]) -> Self {
        value.iter().cloned().collect()
    }
}

impl<T> PartialEq for Soa<T>
where
    T: Soapy + PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        if self.len != other.len {
            return false;
        }

        self.try_fold_zip(other, true, |_, a, b| {
            if a == b {
                ControlFlow::Continue(true)
            } else {
                ControlFlow::Break(false)
            }
        })
    }
}

impl<T, R> PartialEq<R> for Soa<T>
where
    T: Soapy + PartialEq,
    R: AsRef<[T]>,
{
    fn eq(&self, other: &R) -> bool {
        let other = other.as_ref();
        if self.len() != other.len() {
            return false;
        }

        let mut iter = other.into_iter();
        self.try_fold(true, |_, a| {
            let b = iter.next();
            // SAFETY:
            // We already checked that the lengths are the same
            let b = unsafe { b.unwrap_unchecked() };
            if a == b {
                ControlFlow::Continue(true)
            } else {
                ControlFlow::Break(false)
            }
        })
    }
}

impl<T> Eq for Soa<T> where T: Soapy + Eq {}

impl<T> fmt::Debug for Soa<T>
where
    T: Soapy + fmt::Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut list = f.debug_list();
        self.for_each(|item| {
            list.entry(&item);
        });
        list.finish()
    }
}

impl<T> PartialOrd for Soa<T>
where
    T: Soapy + PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let element_wise = self.try_fold_zip(other, Some(Ordering::Equal), |_, a, b| {
            match a.partial_cmp(b) {
                ord @ (None | Some(Ordering::Less | Ordering::Greater)) => ControlFlow::Break(ord),
                Some(Ordering::Equal) => ControlFlow::Continue(Some(Ordering::Equal)),
            }
        });
        match element_wise {
            ord @ (None | Some(Ordering::Less | Ordering::Greater)) => ord,
            Some(Ordering::Equal) => Some(self.len.cmp(&other.len)),
        }
    }
}

impl<T> Ord for Soa<T>
where
    T: Soapy + Ord,
{
    fn cmp(&self, other: &Self) -> Ordering {
        let element_wise = self.try_fold_zip(other, Ordering::Equal, |_, a, b| match a.cmp(b) {
            ord @ (Ordering::Greater | Ordering::Less) => ControlFlow::Break(ord),
            Ordering::Equal => ControlFlow::Continue(Ordering::Equal),
        });
        match element_wise {
            ord @ (Ordering::Less | Ordering::Greater) => ord,
            Ordering::Equal => self.len.cmp(&other.len),
        }
    }
}

impl<T> Default for Soa<T>
where
    T: Soapy,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<T> std::hash::Hash for Soa<T>
where
    T: Soapy + std::hash::Hash,
{
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.len.hash(state);
        self.for_each(|item| item.hash(state));
    }
}

use crate::{
    iter_raw::IterRaw, AsMutSlice, AsSlice, IntoIter, Iter, IterMut, Slice, SliceMut, SliceRef,
    SoaRaw, Soars,
};
use std::{
    borrow::{Borrow, BorrowMut},
    cmp::Ordering,
    fmt::{self, Debug, Formatter},
    hash::{Hash, Hasher},
    marker::PhantomData,
    mem::{needs_drop, size_of, ManuallyDrop},
    ops::{Deref, DerefMut},
    ptr::NonNull,
};

/// A growable array type that stores the values for each field of `T`
/// contiguously.
///
/// The design for SoA aligns closely with [`Vec`]:
/// - Overallocates capacity to provide O(1) amortized insertion
/// - Does not allocate until elements are added
/// - Never deallocates memory unless explicitly requested
/// - Uses `usize::MAX` as the capacity for zero-sized types
///
/// See the top-level [`soa_rs`] docs for usage examples.
///
/// [`soa_rs`]: crate
pub struct Soa<T>
where
    T: Soars,
{
    pub(crate) cap: usize,
    pub(crate) slice: Slice<T, ()>,
    pub(crate) len: usize,
}

impl<T> Soa<T>
where
    T: Soars,
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
    /// # use soa_rs::{Soa, Soars};
    /// # #[derive(Soars, Copy, Clone)]
    /// # #[soa_derive(Debug, PartialEq)]
    /// # struct Foo;
    /// let mut soa = Soa::<Foo>::new();
    /// ```
    pub fn new() -> Self {
        Self {
            cap: if size_of::<T>() == 0 { usize::MAX } else { 0 },
            slice: Slice::empty(),
            len: 0,
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
    /// # use soa_rs::{Soa, Soars};
    /// #[derive(Soars)]
    /// # #[soa_derive(Debug, PartialEq)]
    /// struct Foo(u8, u8);
    ///
    /// let mut soa = Soa::<Foo>::with_capacity(10);
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
    /// #[derive(Soars, Copy, Clone)]
    /// # #[soa_derive(Debug, PartialEq)]
    /// struct Bar;
    ///
    /// // A SOA of a zero-sized type always over-allocates
    /// let soa = Soa::<Bar>::with_capacity(10);
    /// assert_eq!(soa.capacity(), usize::MAX);
    /// ```
    pub fn with_capacity(capacity: usize) -> Self {
        match capacity {
            0 => Self::new(),
            capacity => {
                if size_of::<T>() == 0 {
                    Self {
                        cap: usize::MAX,
                        slice: Slice::empty(),
                        len: 0,
                    }
                } else {
                    Self {
                        cap: capacity,
                        // SAFETY:
                        // - T is nonzero sized
                        // - capacity is nonzero
                        slice: Slice::with_raw(unsafe { T::Raw::alloc(capacity) }),
                        len: 0,
                    }
                }
            }
        }
    }

    /// Constructs a new `Soa<T>` with the given first element.
    ///
    /// This is mainly useful to get around type inference limitations in some
    /// situations, namely macros. Type inference can struggle sometimes due to
    /// dereferencing to an associated type of `T`, which causes Rust to get
    /// confused about whether, for example, `push`ing and element should coerce
    /// `self` to the argument's type.
    ///
    /// # Examples
    ///
    /// ```
    /// # use soa_rs::{Soa, Soars, soa};
    /// # #[derive(Soars, Debug, PartialEq)]
    /// # #[soa_derive(Debug, PartialEq)]
    /// # struct Foo(usize);
    /// let soa = Soa::with(Foo(10));
    /// assert_eq!(soa, soa![Foo(10)]);
    /// ```
    pub fn with(element: T) -> Self {
        let mut out = Self::new();
        out.push(element);
        out
    }

    /// Returns the total number of elements the container can hold without
    /// reallocating.
    ///
    /// # Examples
    ///
    /// ```
    /// # use soa_rs::{Soa, Soars};
    /// # #[derive(Soars)]
    /// # #[soa_derive(Debug, PartialEq)]
    /// # struct Foo(usize);
    /// let mut soa = Soa::<Foo>::new();
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
    /// # use soa_rs::{Soa, Soars, soa};
    /// # #[derive(Soars, Debug, PartialEq)]
    /// # #[soa_derive(Debug, PartialEq)]
    /// # struct Foo(usize);
    /// let soa = soa![Foo(1), Foo(2)];
    /// let (ptr, len, cap) = soa.into_raw_parts();
    /// let rebuilt = unsafe { Soa::<Foo>::from_raw_parts(ptr, len, cap) };
    /// assert_eq!(rebuilt, soa![Foo(1), Foo(2)]);
    /// ```
    pub fn into_raw_parts(self) -> (NonNull<u8>, usize, usize) {
        let me = ManuallyDrop::new(self);
        (me.raw().into_parts(), me.len, me.cap)
    }

    /// Creates a `Soa<T>` from a pointer, a length, and a capacity.
    ///
    /// # Safety
    ///
    /// This is highly unsafe due to the number of invariants that aren't
    /// checked. Given that many of these invariants are private implementation
    /// details of [`SoaRaw`], it is better not to uphold them manually. Rather,
    /// it only valid to call this method with the output of a previous call to
    /// [`Soa::into_raw_parts`].
    pub unsafe fn from_raw_parts(ptr: NonNull<u8>, length: usize, capacity: usize) -> Self { unsafe {
        Self {
            cap: capacity,
            slice: Slice::with_raw(T::Raw::from_parts(ptr, capacity)),
            len: length,
        }
    }}

    /// Appends an element to the back of a collection.
    ///
    /// # Examples
    ///
    /// ```
    /// # use soa_rs::{Soa, Soars, soa};
    /// # #[derive(Soars, Debug, PartialEq)]
    /// # #[soa_derive(Debug, PartialEq)]
    /// # struct Foo(usize);
    /// let mut soa = soa![Foo(1), Foo(2)];
    /// soa.push(Foo(3));
    /// assert_eq!(soa, soa![Foo(1), Foo(2), Foo(3)]);
    /// ```
    pub fn push(&mut self, element: T) {
        self.maybe_grow();
        // SAFETY: After maybe_grow, the allocated capacity is greater than len
        unsafe {
            self.raw().offset(self.len).set(element);
        }
        self.len += 1;
    }

    /// Removes the last element from a vector and returns it, or [`None`] if it
    /// is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// # use soa_rs::{Soa, Soars, soa};
    /// # #[derive(Soars, Debug, PartialEq)]
    /// # #[soa_derive(Debug, PartialEq)]
    /// # struct Foo(usize);
    /// let mut soa = soa![Foo(1), Foo(2), Foo(3)];
    /// assert_eq!(soa.pop(), Some(Foo(3)));
    /// assert_eq!(soa, soa![Foo(1), Foo(2)]);
    /// ```
    pub fn pop(&mut self) -> Option<T> {
        if self.len == 0 {
            None
        } else {
            self.len -= 1;
            // SAFETY: len points to at least one initialized item
            Some(unsafe { self.raw().offset(self.len).get() })
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
    /// # use soa_rs::{Soa, Soars, soa};
    /// # #[derive(Soars, Debug, PartialEq)]
    /// # #[soa_derive(Debug, PartialEq)]
    /// # struct Foo(usize);
    /// let mut soa = soa![Foo(1), Foo(2), Foo(3)];
    /// soa.insert(1, Foo(4));
    /// assert_eq!(soa, soa![Foo(1), Foo(4), Foo(2), Foo(3)]);
    /// soa.insert(4, Foo(5));
    /// assert_eq!(soa, soa![Foo(1), Foo(4), Foo(2), Foo(3), Foo(5)]);
    /// ```
    pub fn insert(&mut self, index: usize, element: T) {
        assert!(index <= self.len, "index out of bounds");
        self.maybe_grow();
        // SAFETY: After the bounds check and maybe_grow, index is an
        // initialized item and index+1 is allocated
        unsafe {
            let ith = self.raw().offset(index);
            ith.copy_to(ith.offset(1), self.len - index);
            ith.set(element);
        }
        self.len += 1;
    }

    /// Removes and returns the element at position index within the vector,
    /// shifting all elements after it to the left.
    ///
    /// # Examples
    ///
    /// ```
    /// # use soa_rs::{Soa, Soars, soa};
    /// # #[derive(Soars, Debug, PartialEq)]
    /// # #[soa_derive(Debug, PartialEq)]
    /// # struct Foo(usize);
    /// let mut soa = soa![Foo(1), Foo(2), Foo(3)];
    /// assert_eq!(soa.remove(1), Foo(2));
    /// assert_eq!(soa, soa![Foo(1), Foo(3)])
    /// ```
    pub fn remove(&mut self, index: usize) -> T {
        assert!(index < self.len, "index out of bounds");
        self.len -= 1;
        // SAFETY: After the bounds check, we know ith item is initialized
        let ith = unsafe { self.raw().offset(index) };
        let out = unsafe { ith.get() };
        // SAFETY: There are len-index initialized elements to shift back
        unsafe {
            ith.offset(1).copy_to(ith, self.len - index);
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
    /// # use soa_rs::{Soa, Soars, soa};
    /// # #[derive(Soars, Debug, PartialEq)]
    /// # #[soa_derive(Debug, PartialEq)]
    /// # struct Foo(usize);
    /// let mut soa = soa![Foo(1)];
    /// soa.reserve(10);
    /// assert!(soa.capacity() >= 11);
    /// ```
    pub fn reserve(&mut self, additional: usize) {
        let new_len = self.len + additional;
        if new_len > self.cap {
            let new_cap = new_len
                // Ensure exponential growth
                .max(self.cap * 2)
                .max(Self::SMALL_CAPACITY);
            self.grow(new_cap);
        }
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
    /// # use soa_rs::{Soa, Soars, soa};
    /// # #[derive(Soars, Debug, PartialEq)]
    /// # #[soa_derive(Debug, PartialEq)]
    /// # struct Foo(usize);
    /// let mut soa = soa![Foo(1)];
    /// soa.reserve(10);
    /// assert!(soa.capacity() == 11);
    /// ```
    pub fn reserve_exact(&mut self, additional: usize) {
        let new_len = additional + self.len;
        if new_len > self.cap {
            self.grow(new_len);
        }
    }

    /// Shrinks the capacity of the container as much as possible.
    ///
    /// # Examples
    ///
    /// ```
    /// # use soa_rs::{Soa, Soars, soa};
    /// # #[derive(Soars, Debug, PartialEq)]
    /// # #[soa_derive(Debug, PartialEq)]
    /// # struct Foo(usize);
    /// let mut soa = Soa::<Foo>::with_capacity(10);
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
    /// # use soa_rs::{Soa, Soars, soa};
    /// # #[derive(Soars, Debug, PartialEq)]
    /// # #[soa_derive(Debug, PartialEq)]
    /// # struct Foo(usize);
    /// let mut soa = Soa::<Foo>::with_capacity(10);
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
    /// # use soa_rs::{Soa, Soars, soa};
    /// # #[derive(Soars, Debug, PartialEq)]
    /// # #[soa_derive(Debug, PartialEq)]
    /// # struct Foo(usize);
    /// let mut soa = soa![Foo(1), Foo(2), Foo(3), Foo(4), Foo(5)];
    /// soa.truncate(2);
    /// assert_eq!(soa, soa![Foo(1), Foo(2)]);
    /// ```
    ///
    /// No truncation occurs when `len` is greater than the SOA's current
    /// length:
    /// ```
    /// # use soa_rs::{Soa, Soars, soa};
    /// # #[derive(Soars, Debug, PartialEq)]
    /// # #[soa_derive(Debug, PartialEq)]
    /// # struct Foo(usize);
    /// let mut soa = soa![Foo(1), Foo(2), Foo(3)];
    /// soa.truncate(8);
    /// assert_eq!(soa, soa![Foo(1), Foo(2), Foo(3)]);
    /// ```
    ///
    /// Truncating with `len == 0` is equivalent to [`Soa::clear`].
    /// ```
    /// # use soa_rs::{Soa, Soars, soa};
    /// # #[derive(Soars, Debug, PartialEq)]
    /// # #[soa_derive(Debug, PartialEq)]
    /// # struct Foo(usize);
    /// let mut soa = soa![Foo(1), Foo(2), Foo(3)];
    /// soa.truncate(0);
    /// assert_eq!(soa, soa![]);
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
    /// # use soa_rs::{Soa, Soars, soa};
    /// # #[derive(Soars, Debug, PartialEq)]
    /// # #[soa_derive(Debug, PartialEq)]
    /// # struct Foo(usize);
    /// let mut soa = soa![Foo(0), Foo(1), Foo(2), Foo(3)];
    ///
    /// assert_eq!(soa.swap_remove(1), Foo(1));
    /// assert_eq!(soa, soa![Foo(0), Foo(3), Foo(2)]);
    ///
    /// assert_eq!(soa.swap_remove(0), Foo(0));
    /// assert_eq!(soa, soa![Foo(2), Foo(3)])
    /// ```
    pub fn swap_remove(&mut self, index: usize) -> T {
        if index >= self.len {
            panic!("index out of bounds")
        }
        self.len -= 1;
        // SAFETY: index and len-1 are initialized elements
        let to_remove = unsafe { self.raw().offset(index) };
        let last = unsafe { self.raw().offset(self.len) };
        let out = unsafe { to_remove.get() };
        unsafe {
            last.copy_to(to_remove, 1);
        }
        out
    }

    /// Moves all the elements of other into self, leaving other empty.
    ///
    /// # Examples
    ///
    /// ```
    /// # use soa_rs::{Soa, Soars, soa};
    /// # #[derive(Soars, Debug, PartialEq)]
    /// # #[soa_derive(Debug, PartialEq)]
    /// # struct Foo(usize);
    /// let mut soa1  = soa![Foo(1), Foo(2), Foo(3)];
    /// let mut soa2 = soa![Foo(4), Foo(5), Foo(6)];
    /// soa1.append(&mut soa2);
    /// assert_eq!(soa1, soa![Foo(1), Foo(2), Foo(3), Foo(4), Foo(5), Foo(6)]);
    /// assert_eq!(soa2, soa![]);
    /// ```
    pub fn append(&mut self, other: &mut Self) {
        self.reserve(other.len);
        for i in 0..other.len {
            // SAFETY: i is in bounds
            let element = unsafe { other.raw().offset(i).get() };
            self.push(element);
        }
        other.clear();
    }

    /// Clears the vector, removing all values.
    ///
    /// Note that this method has no effect on the allocated capacity of the
    /// vector.
    ///
    /// # Examples
    ///
    /// ```
    /// # use soa_rs::{Soa, Soars, soa};
    /// # #[derive(Soars, Debug, PartialEq)]
    /// # #[soa_derive(Debug, PartialEq)]
    /// # struct Foo(usize);
    /// let mut soa = soa![Foo(1), Foo(2)];
    /// soa.clear();
    /// assert!(soa.is_empty());
    /// ```
    pub fn clear(&mut self) {
        while self.pop().is_some() {}
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
            // SAFETY: We asserted the preconditions
            unsafe {
                self.raw().dealloc(self.cap);
            }
            self.raw = T::Raw::dangling();
        } else {
            debug_assert!(new_cap < self.cap);
            debug_assert!(self.len <= new_cap);
            // SAFETY: We asserted the preconditions
            unsafe {
                self.raw = self.raw().realloc_shrink(self.cap, new_cap, self.len);
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
            // SAFETY: We asserted the preconditions
            self.raw = unsafe { T::Raw::alloc(new_cap) };
        } else {
            debug_assert!(self.len <= self.cap);
            // SAFETY: We asserted the preconditions
            unsafe {
                self.raw = self.raw().realloc_grow(self.cap, new_cap, self.len);
            }
        }

        self.cap = new_cap;
    }
}

impl<T> Drop for Soa<T>
where
    T: Soars,
{
    fn drop(&mut self) {
        if needs_drop::<T>() {
            while self.pop().is_some() {}
        }

        if size_of::<T>() > 0 && self.cap > 0 {
            // SAFETY: We asserted the preconditions
            unsafe {
                self.raw().dealloc(self.cap);
            }
        }
    }
}

impl<T> IntoIterator for Soa<T>
where
    T: Soars,
{
    type Item = T;

    type IntoIter = IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        let soa = ManuallyDrop::new(self);
        IntoIter {
            iter_raw: IterRaw {
                slice: soa.slice,
                len: soa.len,
                adapter: PhantomData,
            },
            ptr: soa.raw().into_parts(),
            cap: soa.cap,
        }
    }
}

impl<'a, T> IntoIterator for &'a Soa<T>
where
    T: Soars,
{
    type Item = T::Ref<'a>;

    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.deref().into_iter()
    }
}

impl<'a, T> IntoIterator for &'a mut Soa<T>
where
    T: Soars,
{
    type Item = T::RefMut<'a>;

    type IntoIter = IterMut<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.deref_mut().into_iter()
    }
}

// NOTE: Copy is the required bound because calling Clone::clone on a
// stack-allocated element is unsound in the presence of interior mutability
// unless the fields are written back, which we also can't do because of &self.
impl<T> Clone for Soa<T>
where
    T: Soars + Copy,
{
    fn clone(&self) -> Self {
        let mut out = Self::with_capacity(self.len);
        for i in 0..self.len {
            // SAFETY: i is in-bounds
            let el = unsafe { self.raw.offset(i).get() };
            out.push(el);
        }
        out
    }

    fn clone_from(&mut self, source: &Self) {
        self.clear();
        self.reserve_exact(source.len);
        for i in 0..source.len {
            // SAFETY: i is in-bounds
            let el = unsafe { source.raw.offset(i).get() };
            self.push(el);
        }
    }
}

impl<T> Extend<T> for Soa<T>
where
    T: Soars,
{
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        for item in iter {
            self.push(item);
        }
    }
}

impl<T> FromIterator<T> for Soa<T>
where
    T: Soars,
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
    T: Soars,
{
    /// Allocate a `Soa<T>` and move `value`'s items into it.
    fn from(value: [T; N]) -> Self {
        value.into_iter().collect()
    }
}

impl<T, const N: usize> From<&[T; N]> for Soa<T>
where
    T: Soars + Clone,
{
    /// Allocate a `Soa<T>` and fill it by cloning `value`'s items.
    fn from(value: &[T; N]) -> Self {
        value.iter().cloned().collect()
    }
}

impl<T, const N: usize> From<&mut [T; N]> for Soa<T>
where
    T: Soars + Clone,
{
    /// Allocate a `Soa<T>` and fill it by cloning `value`'s items.
    fn from(value: &mut [T; N]) -> Self {
        value.iter().cloned().collect()
    }
}

impl<T> From<&[T]> for Soa<T>
where
    T: Soars + Clone,
{
    /// Allocate a `Soa<T>` and fill it by cloning `value`'s items.
    fn from(value: &[T]) -> Self {
        value.iter().cloned().collect()
    }
}

impl<T> From<&mut [T]> for Soa<T>
where
    T: Soars + Clone,
{
    /// Allocate a `Soa<T>` and fill it by cloning `value`'s items.
    fn from(value: &mut [T]) -> Self {
        value.iter().cloned().collect()
    }
}

impl<T> Debug for Soa<T>
where
    T: Soars,
    for<'a> T::Ref<'a>: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.as_slice().fmt(f)
    }
}

impl<T> PartialOrd for Soa<T>
where
    T: Soars,
    for<'a> T::Ref<'a>: PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.as_slice().partial_cmp(&other.as_slice())
    }
}

impl<T> Ord for Soa<T>
where
    T: Soars,
    for<'a> T::Ref<'a>: Ord,
{
    fn cmp(&self, other: &Self) -> Ordering {
        self.as_slice().cmp(&other.as_slice())
    }
}

impl<T> Hash for Soa<T>
where
    T: Soars,
    for<'a> T::Ref<'a>: Hash,
{
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.as_slice().hash(state)
    }
}

impl<T> Default for Soa<T>
where
    T: Soars,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<T> AsRef<Slice<T>> for Soa<T>
where
    T: Soars,
{
    fn as_ref(&self) -> &Slice<T> {
        // SAFETY:
        // - len is valid for the slice
        // - The lifetime is bound to self
        unsafe { self.slice.as_unsized(self.len) }
    }
}

impl<T> AsMut<Slice<T>> for Soa<T>
where
    T: Soars,
{
    fn as_mut(&mut self) -> &mut Slice<T> {
        // SAFETY:
        // - len is valid for the slice
        // - The lifetime is bound to self
        unsafe { self.slice.as_unsized_mut(self.len) }
    }
}

impl<T> AsRef<Self> for Soa<T>
where
    T: Soars,
{
    fn as_ref(&self) -> &Self {
        self
    }
}

impl<T> AsMut<Self> for Soa<T>
where
    T: Soars,
{
    fn as_mut(&mut self) -> &mut Self {
        self
    }
}

impl<T> Deref for Soa<T>
where
    T: Soars,
{
    type Target = Slice<T>;

    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl<T> DerefMut for Soa<T>
where
    T: Soars,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut()
    }
}

impl<T> Borrow<Slice<T>> for Soa<T>
where
    T: Soars,
{
    fn borrow(&self) -> &Slice<T> {
        self.as_ref()
    }
}

impl<T> BorrowMut<Slice<T>> for Soa<T>
where
    T: Soars,
{
    fn borrow_mut(&mut self) -> &mut Slice<T> {
        self.as_mut()
    }
}

impl<T, R> PartialEq<R> for Soa<T>
where
    T: Soars,
    R: AsSlice<Item = T> + ?Sized,
    for<'a> T::Ref<'a>: PartialEq,
{
    fn eq(&self, other: &R) -> bool {
        self.as_slice() == other.as_slice()
    }
}

impl<T> Eq for Soa<T>
where
    T: Soars,
    for<'a> T::Ref<'a>: Eq,
{
}

impl<T> AsSlice for Soa<T>
where
    T: Soars,
{
    type Item = T;

    fn as_slice(&self) -> SliceRef<'_, Self::Item> {
        // SAFETY:
        // - len is valid for this slice
        // - The returned lifetime is bound to self
        unsafe { SliceRef::from_slice(self.slice, self.len) }
    }
}

impl<T> AsMutSlice for Soa<T>
where
    T: Soars,
{
    fn as_mut_slice(&mut self) -> crate::SliceMut<'_, Self::Item> {
        // SAFETY:
        // - len is valid for this slice
        // - The returned lifetime is bound to self
        unsafe { SliceMut::from_slice(self.slice, self.len) }
    }
}

use crate::{slice_raw::SliceRaw, IntoIter, Iter, IterMut};
use soapy_shared::{SoaRaw, Soapy};
use std::{
    cmp::Ordering,
    fmt::{self, Formatter},
    marker::PhantomData,
    mem::{size_of, ManuallyDrop},
    ops::{ControlFlow, Deref, DerefMut},
};

/// A growable array type that stores the values for each field of `T`
/// contiguously.
pub struct Soa<T>
where
    T: Soapy,
{
    pub(crate) cap: usize,
    pub(crate) slice: SliceRaw<T>,
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
    /// let mut soa = Soa::<Foo>::new();
    /// ```
    pub fn new() -> Self {
        Self {
            cap: if size_of::<T>() == 0 { usize::MAX } else { 0 },
            slice: SliceRaw::empty(),
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
    /// #[derive(Soapy)]
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
                        slice: SliceRaw::empty(),
                    }
                } else {
                    Self {
                        cap: capacity,
                        slice: unsafe { SliceRaw::from_raw_parts(T::Raw::alloc(capacity), 0) },
                    }
                }
            }
        }
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
    /// # use soapy::{Soa, Soapy, soa};
    /// # #[derive(Soapy, Debug, PartialEq)]
    /// # struct Foo(usize);
    /// let soa = soa![Foo(1), Foo(2)];
    /// let (ptr, len, cap) = soa.into_raw_parts();
    /// let rebuilt = unsafe { Soa::from_raw_parts(ptr, len, cap) };
    /// assert_eq!(rebuilt, [Foo(1), Foo(2)]);
    /// ```
    pub fn into_raw_parts(self) -> (T::Raw, usize, usize) {
        let me = ManuallyDrop::new(self);
        (me.slice.0.raw, me.slice.0.len, me.cap)
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
    pub unsafe fn from_raw_parts(raw: T::Raw, length: usize, capacity: usize) -> Self {
        Self {
            cap: capacity,
            slice: unsafe { SliceRaw::from_raw_parts(raw, length) },
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
            self.slice.0.raw.set(self.slice.0.len, element);
        }
        self.slice.0.len += 1;
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
        if self.slice.0.len == 0 {
            None
        } else {
            self.slice.0.len -= 1;
            Some(unsafe { self.slice.0.raw.get(self.slice.0.len) })
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
        assert!(index <= self.slice.0.len, "index out of bounds");
        self.maybe_grow();
        unsafe {
            self.slice
                .0
                .raw
                .copy(index, index + 1, self.slice.0.len - index);
            self.slice.0.raw.set(index, element);
        }
        self.slice.0.len += 1;
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
        assert!(index < self.slice.0.len, "index out of bounds");
        self.slice.0.len -= 1;
        let out = unsafe { self.slice.0.raw.get(index) };
        unsafe {
            self.slice
                .0
                .raw
                .copy(index + 1, index, self.slice.0.len - index);
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
        let new_cap = (self.slice.0.len + additional)
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
        let new_cap = (additional + self.slice.0.len).max(self.cap);
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
    /// let mut soa = Soa::<Foo>::with_capacity(10);
    /// soa.extend([Foo(1), Foo(2), Foo(3)]);
    /// assert_eq!(soa.capacity(), 10);
    /// soa.shrink_to_fit();
    /// assert_eq!(soa.capacity(), 3);
    /// ```
    pub fn shrink_to_fit(&mut self) {
        self.shrink(self.slice.0.len);
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
    /// let mut soa = Soa::<Foo>::with_capacity(10);
    /// soa.extend([Foo(1), Foo(2), Foo(3)]);
    /// assert_eq!(soa.capacity(), 10);
    /// soa.shrink_to(4);
    /// assert_eq!(soa.capacity(), 4);
    /// soa.shrink_to(0);
    /// assert_eq!(soa.capacity(), 3);
    pub fn shrink_to(&mut self, min_capacity: usize) {
        let new_cap = self.slice.0.len.max(min_capacity);
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
        while len < self.slice.0.len {
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
        let out = unsafe { self.slice.0.raw.get(index) };
        let last = unsafe { self.slice.0.raw.get(self.slice.0.len - 1) };
        unsafe {
            self.slice.0.raw.set(index, last);
        }
        self.slice.0.len -= 1;
        out
    }

    /// Moves all the elements of other into self, leaving other empty.
    ///
    /// # Examples
    ///
    /// ```
    /// # use soapy::{Soa, Soapy, soa};
    /// # #[derive(Soapy, Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
    /// # struct Foo(usize);
    /// let mut soa1  = soa![Foo(1), Foo(2), Foo(3)];
    /// let mut soa2 = soa![Foo(4), Foo(5), Foo(6)];
    /// soa1.append(&mut soa2);
    /// assert_eq!(soa1, [Foo(1), Foo(2), Foo(3), Foo(4), Foo(5), Foo(6)]);
    /// assert_eq!(soa2, []);
    /// ```
    pub fn append(&mut self, other: &mut Self) {
        for i in 0..other.slice.0.len {
            let element = unsafe { other.slice.0.raw.get(i) };
            self.push(element);
        }
        other.slice.0.len = 0;
    }

    /// Returns an iterator over the elements.
    ///
    /// The iterator yields all items from start to end.
    ///
    /// # Examples
    ///
    /// ```
    /// # use soapy::{Soa, Soapy, soa, WithRef};
    /// # use std::fmt;
    /// # #[derive(Soapy, Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
    /// # struct Foo(usize);
    /// # impl<'a> fmt::Debug for FooSoaRef<'a> {
    /// #     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    /// #         self.with_ref(|me| me.fmt(f))
    /// #     }
    /// # }
    /// # impl<'a> PartialEq<FooSoaRef<'a>> for FooSoaRef<'a> {
    /// #     fn eq(&self, other: &FooSoaRef) -> bool {
    /// #         self.with_ref(|me| other.with_ref(|other| me == other))
    /// #     }
    /// # }
    /// let soa = soa![Foo(1), Foo(2), Foo(4)];
    /// let mut iter = soa.iter();
    /// assert_eq!(iter.next(), Some(FooSoaRef(&1)));
    /// assert_eq!(iter.next(), Some(FooSoaRef(&2)));
    /// assert_eq!(iter.next(), Some(FooSoaRef(&4)));
    /// assert_eq!(iter.next(), None);
    /// ```
    pub fn iter(&self) -> Iter<T> {
        Iter {
            raw: self.slice.0.raw,
            start: 0,
            end: self.slice.0.len,
            _marker: PhantomData,
        }
    }

    /// Returns an iterator over the elements that allows modifying each value.
    ///
    /// The iterator yields all items from start to end.
    ///
    /// # Examples
    ///
    /// ```
    /// # use soapy::{Soa, Soapy, soa, WithRef};
    /// # use std::fmt;
    /// # #[derive(Soapy, Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
    /// # struct Foo(usize);
    /// let mut soa = soa![Foo(1), Foo(2), Foo(4)];
    /// for elem in soa.iter_mut() {
    ///     *elem.0 *= 2;
    /// }
    /// assert_eq!(soa, [Foo(2), Foo(4), Foo(8)]);
    /// ```
    pub fn iter_mut(&mut self) -> IterMut<T> {
        IterMut {
            raw: self.slice.0.raw,
            start: 0,
            end: self.slice.0.len,
            _marker: PhantomData,
        }
    }

    /// An internal iteration version of [`Iterator::try_fold`].
    ///
    /// Internal iteration is useful whenever you need to work with the elements
    /// of `Soa` as `T`, rather than as [`Soapy::Ref`]. This can be the case if
    /// you want to take advantage of traits or methods that are only
    /// implemented for `T`. You can also use [`WithRef`] on the elements of
    /// [`Iter`] or [`IterMut`].
    ///
    /// `try_fold` takes two arguments: an initial value, and a closure with two
    /// arguments: an ‘accumulator’, and an element. The closure either returns
    /// [`Continue`], with the value that the accumulator should have for the
    /// next iteration, or it returns [`Break`], with a value that is returned
    /// to the caller immediately (short-circuiting).
    ///
    /// The initial value is the value the accumulator will have on the first
    /// call. If applying the closure succeeded against every element of the
    /// iterator, `try_fold` returns the final accumulator.
    ///
    /// # Examples
    ///
    /// Basic usage:
    /// ```
    /// # use soapy::{Soa, Soapy, soa};
    /// # use std::ops::{Add, ControlFlow};
    /// # #[derive(Soapy, Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
    /// # struct Foo(u8);
    /// # impl Add for Foo {
    /// #     type Output = Foo;
    /// #
    /// #     fn add(self, other: Self) -> Self::Output {
    /// #         Self(self.0 + other.0)
    /// #     }
    /// # }
    /// let soa = soa![Foo(1), Foo(2), Foo(3)];
    /// let sum = soa.try_fold(Foo(0), |acc, &foo| ControlFlow::Continue(acc + foo));
    /// assert_eq!(sum, Foo(6));
    /// ```
    ///
    /// Short circuiting:
    /// ```
    /// # use soapy::{Soa, Soapy, soa};
    /// # use std::ops::{Add, ControlFlow};
    /// # #[derive(Soapy, Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
    /// # struct Foo(u8);
    /// let soa = soa![Foo(1), Foo(2), Foo(3), Foo(4)];
    /// let index_of = |needle| soa.try_fold(0, |i, &foo| {
    ///     if foo == needle {
    ///         ControlFlow::Break(i)
    ///     } else {
    ///         ControlFlow::Continue(i + 1)
    ///     }
    /// });
    /// assert_eq!(index_of(Foo(2)), 1);
    /// assert_eq!(index_of(Foo(4)), 3);
    /// ```
    ///
    /// [`Continue`]: ControlFlow::Continue
    /// [`Break`]: ControlFlow::Break
    /// [`WithRef`]: crate::WithRef
    pub fn try_fold<F, B>(&self, init: B, mut f: F) -> B
    where
        F: FnMut(B, &T) -> ControlFlow<B, B>,
    {
        let mut acc = init;
        for i in 0..self.slice.0.len {
            // SAFETY:
            // Okay to construct an element and take its reference, so long as
            // we don't run its destructor.
            let element = ManuallyDrop::new(unsafe { self.slice.0.raw.get(i) });
            let result = f(acc, &element);
            match result {
                ControlFlow::Continue(b) => acc = b,
                ControlFlow::Break(b) => return b,
            }
        }
        acc
    }

    /// Internal iteration over two `Soa` that applies a function to each pair
    /// of elements.
    ///
    /// This function is similar to calling [`Iterator::try_fold`] on a [`Zip`].
    /// It will walk each collection and call the provided function with each
    /// pair of elements, short-circuiting when either container's elements are
    /// exhausted or when the provided function returns [`Break`].
    ///
    /// Internal iteration is useful whenever you need to iterate the elements
    /// of `Soa<T>` as `T`, rather than as [`Soapy::Ref`]. This can be the case
    /// if you want to take advantage of traits or methods that are only
    /// implemented for `T`. You can also use [`WithRef`] on the items of
    /// [`Iter`] or [`IterMut`] for similar effect.
    ///
    /// `try_fold_zip` takes two arguments: an initial value, and a closure with
    /// three arguments: an ‘accumulator’, and a pair of elements. The closure
    /// either returns [`Continue`], with the value that the accumulator should
    /// have for the next iteration, or it returns [`Break`], with a value that
    /// is returned to the caller immediately (short-circuiting).
    ///
    /// The initial value is the value the accumulator will have on the first
    /// call. If applying the closure succeeded against every element of the
    /// iterator, `try_fold` returns the final accumulator.
    ///
    /// See also [`try_fold`].
    ///
    /// # Examples
    ///
    /// ```
    /// # use soapy::{Soa, Soapy, soa};
    /// # use std::ops::{Add, ControlFlow};
    /// # #[derive(Soapy, Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
    /// # struct Foo(u8);
    /// let soa1 = soa![Foo(1), Foo(2)];
    /// let soa2 = soa![Foo(3), Foo(4), Foo(5)];
    /// let sums = Soa::try_fold_zip(&soa1, &soa2, vec![], |mut acc, &a, &b| {
    ///     acc.push(a.0 + b.0);
    ///     ControlFlow::Continue(acc)
    /// });
    /// assert_eq!(sums, vec![4, 6]);
    /// ```
    ///
    /// [`try_fold`]: Soa::try_fold
    /// [`Zip`]: std::iter::Zip
    /// [`Continue`]: ControlFlow::Continue
    /// [`Break`]: ControlFlow::Break
    /// [`WithRef`]: crate::WithRef
    pub fn try_fold_zip<F, B>(&self, other: &Self, init: B, mut f: F) -> B
    where
        F: FnMut(B, &T, &T) -> ControlFlow<B, B>,
    {
        let mut acc = init;
        let len = self.slice.0.len.min(other.slice.0.len);
        for i in 0..len {
            // SAFETY:
            // Okay to construct an element and take its reference, so long as
            // we don't run its destructor.
            let a = ManuallyDrop::new(unsafe { self.slice.0.raw.get(i) });
            let b = ManuallyDrop::new(unsafe { other.slice.0.raw.get(i) });
            let result = f(acc, &a, &b);
            match result {
                ControlFlow::Continue(b) => acc = b,
                ControlFlow::Break(b) => return b,
            }
        }
        acc
    }

    /// Calls a closure on each element of the collection.
    ///
    /// This is an internal iteration version of [`Iterator::for_each`] It is
    /// equivalent to a for loop over the collection, although break and
    /// continue are not possible from a closure.
    ///
    /// Internal iteration is useful whenever you need to iterate the elements
    /// of `Soa<T>` as `T`, rather than as [`Soapy::Ref`]. This can be the case
    /// if you want to take advantage of traits or methods that are only
    /// implemented for `T`. You can also use [`WithRef`] on the items of
    /// [`Iter`] or [`IterMut`] for similar effect.
    ///
    /// # Examples
    ///
    /// ```
    /// # use soapy::{Soa, Soapy, soa};
    /// # #[derive(Soapy, Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
    /// # struct Foo(String);
    /// # impl Foo {
    /// #     fn new(s: &str) -> Self {
    /// #         Self(s.to_string())
    /// #     }
    /// # }
    /// # impl std::ops::Deref for Foo {
    /// #     type Target = str;
    /// #     fn deref(&self) -> &Self::Target {
    /// #         &self.0
    /// #     }
    /// # }
    /// let soa = soa![Foo::new("Hello "), Foo::new("for_each")];
    /// let mut msg = String::new();
    /// soa.for_each(|item| msg.push_str(item));
    /// assert_eq!(msg, "Hello for_each");
    /// ```
    ///
    /// [`WithRef`]: crate::WithRef
    pub fn for_each<F>(&self, mut f: F)
    where
        F: FnMut(&T),
    {
        self.try_fold((), |_, item| {
            f(item);
            ControlFlow::Continue(())
        })
    }

    /// Clears the vector, removing all values.
    ///
    /// Note that this method has no effect on the allocated capacity of the
    /// vector.
    ///
    /// # Examples
    ///
    /// ```
    /// # use soapy::{Soa, Soapy, soa};
    /// # #[derive(Soapy, Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
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
        if self.slice.0.len < self.cap {
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
                self.slice.0.raw.dealloc(self.cap);
            }
            self.slice.0.raw = T::Raw::dangling();
        } else {
            debug_assert!(new_cap < self.cap);
            debug_assert!(self.slice.0.len <= new_cap);
            unsafe {
                self.slice
                    .0
                    .raw
                    .realloc_shrink(self.cap, new_cap, self.slice.0.len);
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
            self.slice.0.raw = unsafe { T::Raw::alloc(new_cap) };
        } else {
            debug_assert!(self.slice.0.len <= self.cap);
            unsafe {
                self.slice
                    .0
                    .raw
                    .realloc_grow(self.cap, new_cap, self.slice.0.len);
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
                self.slice.0.raw.dealloc(self.cap);
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
        let soa = ManuallyDrop::new(self);
        IntoIter {
            start: 0,
            end: soa.slice.0.len,
            raw: soa.slice.0.raw,
            cap: soa.cap,
        }
    }
}

impl<T> Clone for Soa<T>
where
    T: Soapy + Clone,
{
    fn clone(&self) -> Self {
        let mut out = Self::with_capacity(self.slice.0.len);
        self.for_each(|el| {
            out.push(el.clone());
        });
        out
    }

    fn clone_from(&mut self, source: &Self) {
        self.clear();
        if self.cap < source.slice.0.len {
            self.reserve(source.slice.0.len);
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
        self.deref().eq(other.deref())
    }
}

impl<T, R> PartialEq<R> for Soa<T>
where
    T: Soapy + PartialEq,
    R: AsRef<[T]>,
{
    fn eq(&self, other: &R) -> bool {
        self.deref().eq(other)
    }
}

impl<T> Eq for Soa<T> where T: Soapy + Eq {}

impl<T> fmt::Debug for Soa<T>
where
    T: Soapy + fmt::Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.deref().fmt(f)
    }
}

impl<T> PartialOrd for Soa<T>
where
    T: Soapy + PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.deref().partial_cmp(other)
    }
}

impl<T> Ord for Soa<T>
where
    T: Soapy + Ord,
{
    fn cmp(&self, other: &Self) -> Ordering {
        self.deref().cmp(other)
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

impl<T> AsRef<SliceRaw<T>> for Soa<T>
where
    T: Soapy,
{
    fn as_ref(&self) -> &SliceRaw<T> {
        &self.slice
    }
}

impl<T> Deref for Soa<T>
where
    T: Soapy,
{
    type Target = SliceRaw<T>;

    fn deref(&self) -> &Self::Target {
        &self.slice
    }
}

impl<T> DerefMut for Soa<T>
where
    T: Soapy,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.slice
    }
}

use crate::{
    AsSlice, Iter, Slice, SoaClone, SoaRaw, Soars, Vec, chunks_exact::ChunksExact,
    index::SoaIndex, iter_raw::IterRaw,
};
use core::{
    borrow::Borrow,
    cmp::Ordering,
    fmt::{self, Debug, Formatter},
    hash::{Hash, Hasher},
    marker::PhantomData,
    ops::Deref,
};

/// An immutably borrowed [`Slice`].
///
/// A `SliceRef` is a thin wrapper over a [`Slice`] that applies the same
/// borrowing rules as an immutable reference. It is semantically equivalent to
/// `&Slice`.
pub struct SliceRef<'a, T>
where
    T: 'a + Soars,
{
    pub(crate) slice: Slice<T, ()>,
    pub(crate) len: usize,
    pub(crate) marker: PhantomData<&'a T>,
}

impl<T> SliceRef<'_, T>
where
    T: Soars,
{
    /// Creates a new [`SliceRef`] from the given slice and length. Intended for
    /// use in proc macro code, not user code.
    ///
    /// # Safety
    ///
    /// The provided slice and its length must be compatible. Since the slice
    /// passed in has no intrinsic lifetime, care must be taken to ensure that
    /// the lifetime of [`SliceRef`] is valid.
    #[doc(hidden)]
    pub unsafe fn from_slice(slice: Slice<T, ()>, len: usize) -> Self {
        Self {
            slice,
            len,
            marker: PhantomData,
        }
    }
}

/// These methods shadow their [`Slice`] counterparts, which are reached by
/// [`Deref`] and would otherwise shorten the lifetime to the borrow of `self`
/// rather than the lifetime of the data.
impl<'a, T> SliceRef<'a, T>
where
    T: Soars,
{
    /// Returns a reference to an element or subslice, or [`None`] if out of
    /// bounds. See [`Slice::get`].
    #[inline]
    pub fn get<I>(self, index: I) -> Option<I::Output<'a>>
    where
        I: SoaIndex<T>,
    {
        index.get(self)
    }

    /// Returns a reference to an element or subslice. See [`Slice::idx`].
    ///
    /// # Panics
    ///
    /// Panics if the index is out of bounds.
    #[inline]
    pub fn idx<I>(self, index: I) -> I::Output<'a>
    where
        I: SoaIndex<T>,
    {
        self.get(index).expect("index out of bounds")
    }

    /// Returns an iterator over the elements. See [`Slice::iter`].
    pub fn iter(self) -> Iter<'a, T> {
        self.into_iter()
    }

    /// Divides one slice into two at an index. See [`Slice::split_at`].
    ///
    /// # Panics
    ///
    /// Panics if `mid > len`.
    ///
    /// # Examples
    ///
    /// A batching iterator, which requires that the halves keep the lifetime of
    /// the data rather than that of the borrow:
    ///
    /// ```
    /// # use soa_rs::{Soars, SliceRef, AsSlice, soa};
    /// # #[derive(Soars, Debug, PartialEq)]
    /// # #[soa_derive(Debug, PartialEq)]
    /// # struct Foo(usize);
    /// struct Batches<'a, T: Soars> {
    ///     batch_size: usize,
    ///     data: SliceRef<'a, T>,
    /// }
    ///
    /// impl<'a, T: Soars> Iterator for Batches<'a, T> {
    ///     type Item = SliceRef<'a, T>;
    ///
    ///     fn next(&mut self) -> Option<Self::Item> {
    ///         let (batch, rest) = self.data.split_at(self.batch_size.min(self.data.len()));
    ///         self.data = rest;
    ///         (!batch.is_empty()).then_some(batch)
    ///     }
    /// }
    ///
    /// let soa = soa![Foo(1), Foo(2), Foo(3), Foo(4), Foo(5)];
    /// let batches: Vec<_> = Batches { batch_size: 2, data: soa.as_slice() }.collect();
    /// assert_eq!(batches, [soa![Foo(1), Foo(2)], soa![Foo(3), Foo(4)], soa![Foo(5)]]);
    /// ```
    #[must_use]
    pub fn split_at(self, mid: usize) -> (Self, Self) {
        match self.split_at_checked(mid) {
            Some(pair) => pair,
            None => panic!("mid > len"),
        }
    }

    /// Divides one slice into two at an index, returning [`None`] if the slice
    /// is too short. See [`Slice::split_at_checked`].
    #[must_use]
    pub fn split_at_checked(self, mid: usize) -> Option<(Self, Self)> {
        // Don't use `bool::then_some` here because constructing an invalid
        // slice is unsound, even if it is not accessed
        // SAFETY: The above bounds check
        (mid <= self.len()).then(|| unsafe { self.split_at_unchecked(mid) })
    }

    /// Divides one slice into two at an index without doing bounds checking.
    /// See [`Slice::split_at_unchecked`].
    ///
    /// # Safety
    ///
    /// Calling this method with an out-of-bounds index is undefined behavior,
    /// even if the resulting slice is not used.
    #[must_use]
    pub unsafe fn split_at_unchecked(self, mid: usize) -> (Self, Self) {
        // SAFETY:
        // - The caller guarantees that mid is in bounds
        // - Both halves inherit the lifetime of self
        let l = unsafe { Self::from_slice(self.slice, mid) };
        let raw = unsafe { self.raw().offset(mid) };
        let r = unsafe { Self::from_slice(Slice::with_raw(raw), self.len() - mid) };
        (l, r)
    }

    /// Returns the first element, or [`None`] if empty. See [`Slice::first`].
    pub fn first(self) -> Option<T::Ref<'a>> {
        self.get(0)
    }

    /// Returns the last element, or [`None`] if empty. See [`Slice::last`].
    pub fn last(self) -> Option<T::Ref<'a>> {
        self.get(self.len().checked_sub(1)?)
    }

    /// Returns an iterator over `chunk_size` elements at a time. See
    /// [`Slice::chunks_exact`].
    ///
    /// # Panics
    ///
    /// Panics if `chunk_size` is 0.
    pub fn chunks_exact(self, chunk_size: usize) -> ChunksExact<'a, T> {
        ChunksExact::new(self, chunk_size)
    }

    /// Returns a collection of slices for each field. See [`Slice::slices`].
    pub fn slices(self) -> T::Slices<'a> {
        // SAFETY:
        // - The returned lifetime is that of self
        // - len elements are allocated and initialized
        unsafe { self.raw().slices(self.len()) }
    }
}

impl<T> Clone for SliceRef<'_, T>
where
    T: Soars,
{
    fn clone(&self) -> Self {
        *self
    }
}

impl<'a, T> Copy for SliceRef<'a, T> where T: 'a + Soars {}

impl<T> AsRef<Slice<T>> for SliceRef<'_, T>
where
    T: Soars,
{
    fn as_ref(&self) -> &Slice<T> {
        // SAFETY:
        // - len is valid
        // - The returned lifetime is bound to self
        unsafe { self.slice.as_unsized(self.len) }
    }
}

impl<T> Borrow<Slice<T>> for SliceRef<'_, T>
where
    T: Soars,
{
    fn borrow(&self) -> &Slice<T> {
        self.as_ref()
    }
}

impl<T> Deref for SliceRef<'_, T>
where
    T: Soars,
{
    type Target = Slice<T>;

    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl<'a, T> IntoIterator for SliceRef<'a, T>
where
    T: Soars,
{
    type Item = T::Ref<'a>;
    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        Iter {
            iter_raw: IterRaw {
                slice: Slice::with_raw(self.raw()),
                len: self.len(),
                adapter: PhantomData,
            },
            _marker: PhantomData,
        }
    }
}

impl<T> Debug for SliceRef<'_, T>
where
    T: Soars,
    for<'b> T::Ref<'b>: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.as_ref().fmt(f)
    }
}

impl<T> PartialOrd for SliceRef<'_, T>
where
    T: Soars,
    for<'b> T::Ref<'b>: PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.as_ref().partial_cmp(other.as_ref())
    }
}

impl<T> Ord for SliceRef<'_, T>
where
    T: Soars + Ord,
    for<'b> T::Ref<'b>: Ord,
{
    fn cmp(&self, other: &Self) -> Ordering {
        self.as_ref().cmp(other.as_ref())
    }
}

impl<T> Hash for SliceRef<'_, T>
where
    T: Soars,
    for<'b> T::Ref<'b>: Hash,
{
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.as_ref().hash(state)
    }
}

impl<T, R> PartialEq<R> for SliceRef<'_, T>
where
    T: Soars,
    R: AsSlice<Item = T> + ?Sized,
    for<'a> T::Ref<'a>: PartialEq,
{
    fn eq(&self, other: &R) -> bool {
        self.as_ref() == other.as_slice().as_ref()
    }
}

impl<T> Eq for SliceRef<'_, T>
where
    T: Soars,
    for<'a> T::Ref<'a>: Eq,
{
}

impl<T> From<SliceRef<'_, T>> for Vec<T>
where
    T: SoaClone,
{
    /// Allocate a `Vec<T>` and fill it by cloning `value`'s items.
    fn from(value: SliceRef<T>) -> Self {
        value.as_ref().into()
    }
}

impl<T> AsSlice for SliceRef<'_, T>
where
    T: Soars,
{
    type Item = T;

    fn as_slice(&self) -> SliceRef<'_, Self::Item> {
        *self
    }
}

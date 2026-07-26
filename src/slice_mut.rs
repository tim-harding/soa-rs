use crate::{
    AsMutSlice, AsSlice, IterMut, Slice, SliceRef, SoaClone, SoaRaw, Soars, Vec, index::SoaIndex,
    iter_raw::IterRaw,
};
use core::{
    borrow::{Borrow, BorrowMut},
    cmp::Ordering,
    fmt::{self, Debug, Formatter},
    hash::{Hash, Hasher},
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

/// An mutably borrowed [`Slice`].
///
/// A `SliceMut` is a thin wrapper over a [`Slice`] that applies the same
/// borrowing rules as a mutable reference. It is semantically equivalent to
/// `&mut Slice`.
pub struct SliceMut<'a, T>
where
    T: 'a + Soars,
{
    pub(crate) slice: Slice<T, ()>,
    pub(crate) len: usize,
    pub(crate) marker: PhantomData<&'a mut T>,
}

impl<T> SliceMut<'_, T>
where
    T: Soars,
{
    /// Creates a new [`SliceMut`] from the given slice and length. Intended for
    /// use in proc macro code, not user code.
    ///
    /// # Safety
    ///
    /// The provided slice and its length must be compatible. Since the slice
    /// passed in has no intrinsic lifetime, care must be taken to ensure that
    /// the lifetime of [`SliceMut`] is valid.
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
/// rather than the lifetime of the data. They consume `self` because that is
/// the only way to hand out the full lifetime without aliasing. To reborrow
/// for a shorter lifetime instead, go through [`as_mut_slice`].
///
/// [`as_mut_slice`]: AsMutSlice::as_mut_slice
impl<'a, T> SliceMut<'a, T>
where
    T: Soars,
{
    /// Returns a mutable reference to an element or subslice, or [`None`] if
    /// out of bounds. See [`Slice::get_mut`].
    #[inline]
    pub fn get_mut<I>(self, index: I) -> Option<I::OutputMut<'a>>
    where
        I: SoaIndex<T>,
    {
        index.get_mut(self)
    }

    /// Returns a mutable reference to an element or subslice. See
    /// [`Slice::idx_mut`].
    ///
    /// # Panics
    ///
    /// Panics if the index is out of bounds.
    #[inline]
    pub fn idx_mut<I>(self, index: I) -> I::OutputMut<'a>
    where
        I: SoaIndex<T>,
    {
        self.get_mut(index).expect("index out of bounds")
    }

    /// Divides one mutable slice into two at an index. See
    /// [`Slice::split_at_mut`].
    ///
    /// # Panics
    ///
    /// Panics if `mid > len`.
    ///
    /// # Examples
    ///
    /// A draining batcher, which requires that the halves keep the lifetime of
    /// the data rather than that of the borrow:
    ///
    /// ```
    /// # use soa_rs::{Soars, SliceMut, AsMutSlice, soa};
    /// # #[derive(Soars, Debug, PartialEq)]
    /// # #[soa_derive(Debug, PartialEq)]
    /// # struct Foo(usize);
    /// let mut soa = soa![Foo(1), Foo(2), Foo(3), Foo(4)];
    /// let mut rest = soa.as_mut_slice();
    /// while !rest.is_empty() {
    ///     let mid = 2.min(rest.len());
    ///     let (batch, next) = rest.split_at_mut(mid);
    ///     for mut elem in batch {
    ///         *elem.0 *= 10;
    ///     }
    ///     rest = next;
    /// }
    /// assert_eq!(soa, soa![Foo(10), Foo(20), Foo(30), Foo(40)]);
    /// ```
    #[must_use]
    pub fn split_at_mut(self, mid: usize) -> (Self, Self) {
        match self.split_at_mut_checked(mid) {
            Some(pair) => pair,
            None => panic!("mid > len"),
        }
    }

    /// Divides one mutable slice into two at an index, returning [`None`] if
    /// the slice is too short. See [`Slice::split_at_mut_checked`].
    #[must_use]
    pub fn split_at_mut_checked(self, mid: usize) -> Option<(Self, Self)> {
        // Don't use `bool::then_some` here because constructing an invalid
        // slice is unsound, even if it is not accessed
        // SAFETY: The above bounds check
        (mid <= self.len()).then(|| unsafe { self.split_at_mut_unchecked(mid) })
    }

    /// Divides one mutable slice into two at an index without doing bounds
    /// checking. See [`Slice::split_at_mut_unchecked`].
    ///
    /// # Safety
    ///
    /// Calling this method with an out-of-bounds index is undefined behavior,
    /// even if the resulting slice is not used.
    #[must_use]
    pub unsafe fn split_at_mut_unchecked(self, mid: usize) -> (Self, Self) {
        // SAFETY:
        // - The caller guarantees that mid is in bounds
        // - The halves are disjoint and self is consumed, so they don't alias
        // - Both halves inherit the lifetime of self
        let l = unsafe { Self::from_slice(self.slice, mid) };
        let raw = unsafe { self.raw().offset(mid) };
        let r = unsafe { Self::from_slice(Slice::with_raw(raw), self.len() - mid) };
        (l, r)
    }

}

impl<T> AsRef<Slice<T>> for SliceMut<'_, T>
where
    T: Soars,
{
    fn as_ref(&self) -> &Slice<T> {
        self.deref()
    }
}

impl<T> AsMut<Slice<T>> for SliceMut<'_, T>
where
    T: Soars,
{
    fn as_mut(&mut self) -> &mut Slice<T> {
        self.deref_mut()
    }
}

impl<T> Borrow<Slice<T>> for SliceMut<'_, T>
where
    T: Soars,
{
    fn borrow(&self) -> &Slice<T> {
        self.as_ref()
    }
}

impl<T> BorrowMut<Slice<T>> for SliceMut<'_, T>
where
    T: Soars,
{
    fn borrow_mut(&mut self) -> &mut Slice<T> {
        self.as_mut()
    }
}

impl<T> Deref for SliceMut<'_, T>
where
    T: Soars,
{
    type Target = Slice<T>;

    fn deref(&self) -> &Self::Target {
        // SAFETY:
        // - len is valid
        // - The lifetime is bound to self
        unsafe { self.slice.as_unsized(self.len) }
    }
}

impl<T> DerefMut for SliceMut<'_, T>
where
    T: Soars,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        // SAFETY:
        // - len is valid
        // - The lifetime is bound to self
        unsafe { self.slice.as_unsized_mut(self.len) }
    }
}

impl<'a, T> IntoIterator for SliceMut<'a, T>
where
    T: Soars,
{
    type Item = T::RefMut<'a>;
    type IntoIter = IterMut<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        IterMut {
            iter_raw: IterRaw {
                slice: Slice::with_raw(self.raw()),
                len: self.len(),
                adapter: PhantomData,
            },
            _marker: PhantomData,
        }
    }
}

impl<T> Debug for SliceMut<'_, T>
where
    T: Soars,
    for<'b> T::Ref<'b>: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.as_ref().fmt(f)
    }
}

impl<T> PartialOrd for SliceMut<'_, T>
where
    T: Soars,
    for<'b> T::Ref<'b>: PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.as_ref().partial_cmp(other.as_ref())
    }
}

impl<T> Ord for SliceMut<'_, T>
where
    T: Soars,
    for<'b> T::Ref<'b>: Ord,
{
    fn cmp(&self, other: &Self) -> Ordering {
        self.as_ref().cmp(other.as_ref())
    }
}

impl<T> Hash for SliceMut<'_, T>
where
    T: Soars,
    for<'b> T::Ref<'b>: Hash,
{
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.as_ref().hash(state)
    }
}

impl<T, R> PartialEq<R> for SliceMut<'_, T>
where
    T: Soars,
    R: AsSlice<Item = T> + ?Sized,
    for<'a> T::Ref<'a>: PartialEq,
{
    fn eq(&self, other: &R) -> bool {
        self.as_ref() == other.as_slice().as_ref()
    }
}

impl<T> Eq for SliceMut<'_, T>
where
    T: Soars,
    for<'a> T::Ref<'a>: Eq,
{
}

impl<T> From<SliceMut<'_, T>> for Vec<T>
where
    T: SoaClone,
{
    /// Allocate a `Vec<T>` and fill it by cloning `value`'s items.
    fn from(value: SliceMut<T>) -> Self {
        value.as_ref().into()
    }
}

impl<T> AsSlice for SliceMut<'_, T>
where
    T: Soars,
{
    type Item = T;

    fn as_slice(&self) -> SliceRef<'_, Self::Item> {
        // SAFETY:
        // - len is valid
        // - The lifetime is bound to self
        unsafe { SliceRef::from_slice(self.slice, self.len) }
    }
}

impl<T> AsMutSlice for SliceMut<'_, T>
where
    T: Soars,
{
    fn as_mut_slice(&mut self) -> SliceMut<'_, Self::Item> {
        // SAFETY:
        // - len is valid
        // - The lifetime is bound to self
        unsafe { Self::from_slice(self.slice, self.len) }
    }
}

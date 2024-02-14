use crate::{
    index::SoaIndex, iter_raw::IterRaw, soa_ref::RefMut, Iter, IterMut, Ref, SoaRaw, Soapy, WithRef,
};
use std::{
    cmp::Ordering,
    fmt::{self, Debug, Formatter},
    hash::{Hash, Hasher},
    marker::PhantomData,
    mem::transmute,
    ops::{ControlFlow, Deref, DerefMut},
};

/// A dynamically-sized view into the contents of a [`Soa`].
///
/// [`Slice`] and [`Soa`] have the same relationship as `[T]` and [`Vec`]. The
/// related types [`SliceRef`] and [`SliceMut`] are equivalent to `&[T]` and
/// `&mut [T]`.
///
/// This struct provides most of the implementation for [`Soa`], [`SliceRef`],
/// and [`SliceMut`] via [`Deref`] impls. It is not usually constructed directly
/// but instead used through one of these other types. The [`SliceRef`] and
/// [`SliceMut`] wrappers attach lifetimes and ensure the same borrowing rules
/// as `&` and `&mut`.
///
/// While [`Vec`] can return `&[T]` for all its slice methods, returning
/// `&Slice` is not always possible. That is why [`SliceRef`] and [`SliceMut`]
/// are necessary. While fat pointers allow packing length information as slice
/// metadata, this is insufficient for SoA slices, which require multiple
/// pointers alongside the length. Therefore, SoA slice references cannot be
/// created on the stack and returned like normal slices can.
///
/// [`Soa`]: crate::Soa
/// [`SliceRef`]: crate::SliceRef
/// [`SliceMut`]: crate::SliceMut
pub struct Slice<T>
where
    T: Soapy,
{
    pub(crate) raw: T::Raw,
    pub(crate) len: usize,
}

unsafe impl<T> Send for Slice<T> where T: Send + Soapy {}
unsafe impl<T> Sync for Slice<T> where T: Sync + Soapy {}

impl<T> Slice<T>
where
    T: Soapy,
{
    /// Constructs a new, empty `Slice<T>`.
    ///
    /// # Examples
    /// ```
    /// # use soapy::{Slice, Soapy};
    /// # #[derive(Soapy)]
    /// # struct Foo;
    /// let mut slice = Slice::<Foo>::empty();
    /// ```
    pub fn empty() -> Self {
        Self {
            len: 0,
            raw: <T::Raw as SoaRaw>::dangling(),
        }
    }

    /// Returns the number of elements in the slice, also referred to as its
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
    pub const fn len(&self) -> usize {
        self.len
    }

    /// Returns true if the slice contains no elements.
    ///
    /// # Examples
    ///
    /// ```
    /// # use soapy::{Soa, Soapy};
    /// # #[derive(Soapy)]
    /// # struct Foo(usize);
    /// let mut soa = Soa::<Foo>::new();
    /// assert!(soa.is_empty());
    /// soa.push(Foo(1));
    /// assert!(!soa.is_empty());
    /// ```
    pub const fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Creates a slice from a [`SoaRaw`] and a length.
    ///
    /// # Safety
    ///
    /// This is highly unsafe due to the number of invariants that aren't
    /// checked. Given that many of these invariants are private implementation
    /// details, it is better not to uphold them manually. Instead, use the
    /// methods of, for example, a [`Soa`] to get a handle to a slice.
    ///
    /// [`Soa`]: crate::Soa
    #[doc(hidden)]
    pub const unsafe fn from_raw_parts(raw: T::Raw, length: usize) -> Self {
        Self { len: length, raw }
    }

    /// Gets the [`SoaRaw`] the slice uses.
    ///
    /// Used by the [`Soapy`] derive macro, but generally not intended for use
    /// by end users.
    #[doc(hidden)]
    pub const fn raw(&self) -> T::Raw {
        self.raw
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
    /// let soa = soa![Foo(1), Foo(2), Foo(4)];
    /// let mut iter = soa.iter();
    /// assert_eq!(iter.next().unwrap(), Foo(1));
    /// assert_eq!(iter.next().unwrap(), Foo(2));
    /// assert_eq!(iter.next().unwrap(), Foo(4));
    /// assert_eq!(iter.next(), None);
    /// ```
    pub const fn iter(&self) -> Iter<T> {
        Iter {
            iter_raw: IterRaw {
                slice: Slice {
                    raw: self.raw,
                    len: self.len,
                },
                adapter: PhantomData,
            },
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
    /// for mut elem in soa.iter_mut() {
    ///     *elem.0 *= 2;
    /// }
    /// assert_eq!(soa, [Foo(2), Foo(4), Foo(8)]);
    /// ```
    pub fn iter_mut(&mut self) -> IterMut<T> {
        IterMut {
            iter_raw: IterRaw {
                slice: Slice {
                    raw: self.raw,
                    len: self.len,
                },
                adapter: PhantomData,
            },
            _marker: PhantomData,
        }
    }

    /// Returns a reference to an element or subslice depending on the type of
    /// index.
    ///
    /// - If given a position, returns a reference to the element at that
    /// position or None if out of bounds.
    ///
    /// - If given a range, returns the subslice corresponding to that range, or
    /// None if out of bounds.
    ///
    /// # Examples
    ///
    /// ```
    /// # use std::fmt;
    /// # use soapy::{Soa, Soapy, soa, WithRef, Slice};
    /// # #[derive(Soapy, Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
    /// # struct Foo(usize);
    /// let soa = soa![Foo(10), Foo(40), Foo(30), Foo(20)];
    /// assert_eq!(soa.get(1).unwrap(), Foo(40));
    /// assert_eq!(soa.get(4), None);
    /// assert_eq!(soa.get(..).unwrap(), [Foo(10), Foo(40), Foo(30), Foo(20)]);
    /// assert_eq!(soa.get(..2).unwrap(), [Foo(10), Foo(40)]);
    /// assert_eq!(soa.get(..=2).unwrap(), [Foo(10), Foo(40), Foo(30)]);
    /// assert_eq!(soa.get(2..).unwrap(), [Foo(30), Foo(20)]);
    /// assert_eq!(soa.get(1..3).unwrap(), [Foo(40), Foo(30)]);
    /// assert_eq!(soa.get(1..=3).unwrap(), [Foo(40), Foo(30), Foo(20)]);
    /// assert_eq!(soa.get(2..5), None);
    /// ```
    pub fn get<I>(&self, index: I) -> Option<I::Output<'_>>
    where
        I: SoaIndex<T>,
    {
        index.get(self)
    }

    /// Returns a mutable reference to an element or subslice depending on the
    /// type of index (see [`get`]) or `None` if the index is out of bounds.
    ///
    /// # Examples
    ///
    /// ```
    /// # use soapy::{Soa, Soapy, soa};
    /// # #[derive(Soapy, Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
    /// # struct Foo(usize);
    /// let mut soa = soa![Foo(1), Foo(2), Foo(3)];
    /// if let Some(mut elem) = soa.get_mut(1) {
    ///     *elem.0 = 42;
    /// }
    /// assert_eq!(soa, [Foo(1), Foo(42), Foo(3)]);
    /// ```
    ///
    /// [`get`]: Slice::get
    pub fn get_mut<I>(&mut self, index: I) -> Option<I::OutputMut<'_>>
    where
        I: SoaIndex<T>,
    {
        index.get_mut(self)
    }

    /// Returns a reference to the element at the given index.
    ///
    /// This is similar to [`Index`], which is not implementable for this type.
    /// See [`get`] for a non-panicking version.
    ///
    /// # Panics
    ///
    /// Panics if the index is out-of-bounds, which is whenever
    /// [`SoaIndex::get`] returns [`None`].
    ///
    /// # Examples
    ///
    /// ```
    /// # use std::fmt;
    /// # use soapy::{Soa, Soapy, soa, WithRef};
    /// # #[derive(Soapy, Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
    /// # struct Foo(usize);
    /// let soa = soa![Foo(10), Foo(40), Foo(30), Foo(90)];
    /// assert_eq!(soa.idx(3), Foo(90));
    /// assert_eq!(soa.idx(1..3), [Foo(40), Foo(30)]);
    /// ```
    ///
    /// [`Index`]: std::ops::Index
    /// [`get`]: Slice::get
    pub fn idx<I>(&self, index: I) -> I::Output<'_>
    where
        I: SoaIndex<T>,
    {
        self.get(index).expect("index out of bounds")
    }

    /// Returns a mutable reference to the element at the given index.
    ///
    /// This is similar to [`IndexMut`], which is not implementable for this
    /// type. See [`get_mut`] for a non-panicking version.
    ///
    /// # Panics
    ///
    /// Panics if the index is out-of-bounds, which is whenever
    /// [`SoaIndex::get_mut`] returns [`None`].
    ///
    /// # Examples
    ///
    /// ```
    /// # use std::fmt;
    /// # use soapy::{Soa, Soapy, soa, WithRef};
    /// # #[derive(Soapy, Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
    /// # struct Foo(usize);
    /// let mut soa = soa![Foo(10), Foo(20), Foo(30)];
    /// *soa.idx_mut(1).0 = 42;
    /// assert_eq!(soa, [Foo(10), Foo(42), Foo(30)]);
    /// ```
    ///
    /// [`IndexMut`]: std::ops::Index
    /// [`get_mut`]: Slice::get_mut
    pub fn idx_mut<I>(&mut self, index: I) -> I::OutputMut<'_>
    where
        I: SoaIndex<T>,
    {
        self.get_mut(index).expect("index out of bounds")
    }

    /// Swaps the position of two elements.
    ///
    /// # Arguments
    ///
    /// - `a`: The index of the first element
    /// - `b`: The index of the second element
    ///
    /// # Panics
    ///
    /// Panics if `a` or `b` is out of bounds.
    ///
    /// # Examples
    ///
    /// ```
    /// # use soapy::{Soa, Soapy, soa};
    /// # #[derive(Soapy, Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
    /// # struct Foo(usize);
    /// let mut soa = soa![Foo(0), Foo(1), Foo(2), Foo(3), Foo(4)];
    /// soa.swap(2, 4);
    /// assert_eq!(soa, [Foo(0), Foo(1), Foo(4), Foo(3), Foo(2)]);
    /// ```
    pub fn swap(&mut self, a: usize, b: usize) {
        if a >= self.len || b >= self.len {
            panic!("index out of bounds");
        }

        unsafe {
            let a = self.raw.offset(a);
            let b = self.raw.offset(b);
            let tmp = a.get();
            b.copy_to(a, 1);
            b.set(tmp);
        }
    }

    /// Returns the first element of the slice, or None if empty.
    ///
    /// # Examples
    ///
    /// ```
    /// # use soapy::{Soa, Soapy, soa};
    /// # #[derive(Soapy, Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
    /// # struct Foo(usize);
    /// let soa = soa![Foo(10), Foo(40), Foo(30)];
    /// assert_eq!(soa.first().unwrap(), Foo(10));
    ///
    /// let soa = Soa::<Foo>::new();
    /// assert_eq!(soa.first(), None);
    /// ```
    pub fn first(&self) -> Option<Ref<'_, T>> {
        self.get(0)
    }

    /// Returns a mutable reference to the first element of the slice, or None if empty.
    ///
    /// # Examples
    ///
    /// ```
    /// # use soapy::{Soa, Soapy, soa};
    /// # #[derive(Soapy, Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
    /// # struct Foo(usize);
    /// let mut soa = soa![Foo(0), Foo(1), Foo(2)];
    /// if let Some(mut first) = soa.first_mut() {
    ///     *first.0 = 5;
    /// }
    /// assert_eq!(soa, [Foo(5), Foo(1), Foo(2)]);
    /// ```
    pub fn first_mut(&mut self) -> Option<RefMut<'_, T>> {
        self.get_mut(0)
    }

    /// Returns the last element of the slice, or None if empty.
    ///
    /// # Examples
    ///
    /// ```
    /// # use soapy::{Soa, Soapy, soa};
    /// # #[derive(Soapy, Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
    /// # struct Foo(usize);
    /// let soa = soa![Foo(10), Foo(40), Foo(30)];
    /// assert_eq!(soa.last().unwrap(), Foo(30));
    ///
    /// let soa = Soa::<Foo>::new();
    /// assert_eq!(soa.last(), None);
    /// ```
    pub fn last(&self) -> Option<Ref<'_, T>> {
        self.get(self.len().saturating_sub(1))
    }

    /// Returns a mutable reference to the last element of the slice, or None if empty.
    ///
    /// # Examples
    ///
    /// ```
    /// # use soapy::{Soa, Soapy, soa};
    /// # #[derive(Soapy, Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
    /// # struct Foo(usize);
    /// let mut soa = soa![Foo(0), Foo(1), Foo(2)];
    /// if let Some(mut last) = soa.last_mut() {
    ///     *last.0 = 5;
    /// }
    /// assert_eq!(soa, [Foo(0), Foo(1), Foo(5)]);
    /// ```
    pub fn last_mut(&mut self) -> Option<RefMut<'_, T>> {
        self.get_mut(self.len().saturating_sub(1))
    }
}

impl<'a, T> IntoIterator for &'a Slice<T>
where
    T: Soapy,
{
    type Item = Ref<'a, T>;
    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, T> IntoIterator for &'a mut Slice<T>
where
    T: Soapy,
{
    type Item = RefMut<'a, T>;
    type IntoIter = IterMut<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<T> PartialEq for Slice<T>
where
    T: Soapy + PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        if self.len != other.len {
            return false;
        }
        self.iter().zip(other.iter()).all(|(me, them)| me == them)
    }
}

impl<T> Eq for Slice<T> where T: Soapy + Eq {}

impl<T> PartialEq<[T]> for Slice<T>
where
    T: Soapy + PartialEq,
{
    fn eq(&self, other: &[T]) -> bool {
        if self.len() != other.len() {
            return false;
        }
        self.iter()
            .zip(other.iter())
            .all(|(me, them)| me.with_ref(|me| me == them))
    }
}

impl<T> PartialEq<Vec<T>> for Slice<T>
where
    T: Soapy + PartialEq,
{
    fn eq(&self, other: &Vec<T>) -> bool {
        self.eq(other.as_slice())
    }
}

impl<T> PartialEq<&[T]> for Slice<T>
where
    T: Soapy + PartialEq,
{
    fn eq(&self, other: &&[T]) -> bool {
        self.eq(*other)
    }
}

impl<T> PartialEq<&mut [T]> for Slice<T>
where
    T: Soapy + PartialEq,
{
    fn eq(&self, other: &&mut [T]) -> bool {
        self.eq(*other)
    }
}

impl<T, const N: usize> PartialEq<[T; N]> for Slice<T>
where
    T: Soapy + PartialEq,
{
    fn eq(&self, other: &[T; N]) -> bool {
        self.eq(other.as_slice())
    }
}

impl<T, const N: usize> PartialEq<&[T; N]> for Slice<T>
where
    T: Soapy + PartialEq,
{
    fn eq(&self, other: &&[T; N]) -> bool {
        self.eq(*other)
    }
}

impl<T, const N: usize> PartialEq<&mut [T; N]> for Slice<T>
where
    T: Soapy + PartialEq,
{
    fn eq(&self, other: &&mut [T; N]) -> bool {
        self.eq(*other)
    }
}

macro_rules! partial_eq_reflect {
    ($t:ty) => {
        impl<T> PartialEq<Slice<T>> for $t
        where
            T: Soapy + PartialEq,
        {
            fn eq(&self, other: &Slice<T>) -> bool {
                other.eq(self)
            }
        }
    };
}

macro_rules! partial_eq_reflect_array {
    ($t:ty) => {
        impl<T, const N: usize> PartialEq<Slice<T>> for $t
        where
            T: Soapy + PartialEq,
        {
            fn eq(&self, other: &Slice<T>) -> bool {
                other.eq(self)
            }
        }
    };
}

partial_eq_reflect!(Vec<T>);
partial_eq_reflect!([T]);
partial_eq_reflect!(&[T]);
partial_eq_reflect!(&mut [T]);
partial_eq_reflect_array!([T; N]);
partial_eq_reflect_array!(&[T; N]);
partial_eq_reflect_array!(&mut [T; N]);

impl<T> Debug for Slice<T>
where
    T: Soapy + Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut list = f.debug_list();
        self.iter().for_each(|item| {
            item.with_ref(|item| list.entry(&item));
        });
        list.finish()
    }
}

impl<T> PartialOrd for Slice<T>
where
    T: Soapy + PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match self
            .iter()
            .zip(other.iter())
            .try_fold(Ordering::Equal, |_, (a, b)| match a.partial_cmp(&b) {
                ord @ (None | Some(Ordering::Less | Ordering::Greater)) => ControlFlow::Break(ord),
                Some(Ordering::Equal) => ControlFlow::Continue(self.len.cmp(&other.len)),
            }) {
            ControlFlow::Continue(ord) => Some(ord),
            ControlFlow::Break(ord) => ord,
        }
    }
}

impl<T> Ord for Slice<T>
where
    T: Soapy + Ord,
{
    fn cmp(&self, other: &Self) -> Ordering {
        match self
            .iter()
            .zip(other.iter())
            .try_fold(Ordering::Equal, |_, (a, b)| match a.cmp(&b) {
                ord @ (Ordering::Greater | Ordering::Less) => ControlFlow::Break(ord),
                Ordering::Equal => ControlFlow::Continue(self.len.cmp(&other.len)),
            }) {
            ControlFlow::Continue(ord) | ControlFlow::Break(ord) => ord,
        }
    }
}

impl<T> Default for Slice<T>
where
    T: Soapy,
{
    fn default() -> Self {
        Self::empty()
    }
}

impl<T> Hash for Slice<T>
where
    T: Soapy + Hash,
{
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.len.hash(state);
        for el in self.iter() {
            el.with_ref(|el| el.hash(state))
        }
    }
}

impl<T> Clone for Slice<T>
where
    T: Soapy,
{
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for Slice<T> where T: Soapy {}

impl<T> Deref for Slice<T>
where
    T: Soapy,
{
    type Target = T::Deref;

    fn deref(&self) -> &Self::Target {
        // SAFETY:
        // T::Deref is #[repr(transparent)] of Slice
        unsafe { transmute(self) }
    }
}

impl<T> DerefMut for Slice<T>
where
    T: Soapy,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        // SAFETY:
        // T::Deref is #[repr(transparent)] of Slice
        unsafe { transmute(self) }
    }
}

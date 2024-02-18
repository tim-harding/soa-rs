use crate::{
    chunks_exact::ChunksExact, dst::Dst, index::SoaIndex, iter_raw::IterRaw, soa_ref::RefMut, Iter,
    IterMut, Ref, SliceMut, SliceRef, Soa, SoaDeref, SoaRaw, Soapy, WithRef,
};
use std::{
    cmp::Ordering,
    fmt::{self, Debug, Formatter},
    hash::{Hash, Hasher},
    marker::PhantomData,
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
#[repr(transparent)]
pub struct Slice<T: Soapy, D: ?Sized = [()]>(pub(crate) Dst<T::Raw, D>);

impl<T> Slice<T, ()>
where
    T: Soapy,
{
    /// Constructs a new, empty `Slice<T>`.
    pub(crate) fn empty() -> Self {
        Self(Dst(<T::Raw as SoaRaw>::dangling(), ()))
    }

    pub(crate) fn with_raw(raw: T::Raw) -> Self {
        Self(Dst(raw, ()))
    }

    pub(crate) unsafe fn as_unsized_mut<'a>(&mut self, len: usize) -> &'a mut Slice<T> {
        std::mem::transmute(self.0.as_unsized_mut(len))
    }

    pub(crate) unsafe fn as_unsized<'a>(&self, len: usize) -> &'a Slice<T> {
        std::mem::transmute(self.0.as_unsized(len))
    }
}

impl<T, D: ?Sized> Slice<T, D>
where
    T: Soapy,
{
    /// Gets the [`SoaRaw`] the slice uses.
    ///
    /// Used by the [`Soapy`] derive macro, but generally not intended for use
    /// by end users.
    #[doc(hidden)]
    pub const fn raw(&self) -> T::Raw {
        self.0 .0
    }

    /// Sets the [`SoaRaw`] the slice uses.
    #[doc(hidden)]
    pub(crate) fn set_raw(&mut self, raw: T::Raw) {
        self.0 .0 = raw;
    }
}

impl<T> Slice<T>
where
    T: Soapy,
{
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
        self.0.len()
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

    /// Returns an iterator over the elements.
    ///
    /// The iterator yields all items from start to end.
    ///
    /// # Examples
    ///
    /// ```
    /// # use soapy::{Soa, Soapy, soa, WithRef};
    /// # use std::fmt;
    /// # #[derive(Soapy, Debug, PartialEq)]
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
                slice: unsafe { self.as_sized() },
                len: self.len(),
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
    /// # #[derive(Soapy, Debug, PartialEq)]
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
                slice: unsafe { self.as_sized() },
                len: self.len(),
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
    /// # #[derive(Soapy, Debug, PartialEq)]
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
    /// # #[derive(Soapy, Debug, PartialEq)]
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
    /// # #[derive(Soapy, Debug, PartialEq)]
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
    /// # #[derive(Soapy, Debug, PartialEq)]
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
    /// # #[derive(Soapy, Debug, PartialEq)]
    /// # struct Foo(usize);
    /// let mut soa = soa![Foo(0), Foo(1), Foo(2), Foo(3), Foo(4)];
    /// soa.swap(2, 4);
    /// assert_eq!(soa, [Foo(0), Foo(1), Foo(4), Foo(3), Foo(2)]);
    /// ```
    pub fn swap(&mut self, a: usize, b: usize) {
        if a >= self.len() || b >= self.len() {
            panic!("index out of bounds");
        }

        unsafe {
            let a = self.raw().offset(a);
            let b = self.raw().offset(b);
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
    /// # #[derive(Soapy, Debug, PartialEq)]
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
    /// # #[derive(Soapy, Debug, PartialEq)]
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
    /// # #[derive(Soapy, Debug, PartialEq)]
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
    /// # #[derive(Soapy, Debug, PartialEq)]
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

    /// Returns an iterator over `chunk_size` elements of the slice at a time,
    /// starting at the beginning of the slice.
    ///
    /// The chunks are slices and do not overlap. If `chunk_size` does not divide
    /// the length of the slice, then the last up to `chunk_size-1` elements will
    /// be omitted and can be retrieved from the [`remainder`] function of the
    /// iterator.
    ///
    /// Due to each chunk having exactly `chunk_size` elements, the compiler can
    /// often optimize the resulting code better than in the case of chunks.
    ///
    /// [`remainder`]: ChunksExact::remainder
    ///
    /// # Examples
    ///
    /// ```
    /// # use soapy::{Soa, Soapy, soa};
    /// # #[derive(Soapy, Debug, PartialEq)]
    /// # struct Foo(char);
    /// let soa = soa![Foo('l'), Foo('o'), Foo('r'), Foo('e'), Foo('m')];
    /// let mut iter = soa.chunks_exact(2);
    /// assert_eq!(iter.next().unwrap(), [Foo('l'), Foo('o')]);
    /// assert_eq!(iter.next().unwrap(), [Foo('r'), Foo('e')]);
    /// assert!(iter.next().is_none());
    /// assert_eq!(iter.remainder(), [Foo('m')]);
    /// ```
    pub fn chunks_exact(&self, chunk_size: usize) -> ChunksExact<'_, T> {
        if chunk_size == 0 {
            panic!("chunk size must be nonzero")
        }

        ChunksExact::new(self, chunk_size)
    }

    pub(crate) const unsafe fn as_sized(&self) -> Slice<T, ()> {
        Slice(unsafe { self.0.as_sized() })
    }
}

impl<T> Clone for Slice<T, ()>
where
    T: Soapy,
{
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for Slice<T, ()> where T: Soapy {}

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

impl<T, U> PartialEq<Slice<U>> for Slice<T>
where
    T: Soapy + PartialEq<U>,
    U: Soapy,
{
    fn eq(&self, other: &Slice<U>) -> bool {
        self.len() == other.len() && self.iter().zip(other.iter()).all(|(me, them)| me == them)
    }
}

impl<T> Eq for Slice<T> where T: Soapy + Eq {}

impl<T, U> PartialEq<Slice<U>> for [T]
where
    T: PartialEq<U>,
    U: Soapy,
{
    fn eq(&self, other: &Slice<U>) -> bool {
        self.len() == other.len()
            && self
                .iter()
                .zip(other.iter())
                .all(|(me, them)| them.with_ref(|them| me == them))
    }
}

impl<T, U> PartialEq<[U]> for Slice<T>
where
    T: Soapy + PartialEq<U>,
{
    fn eq(&self, other: &[U]) -> bool {
        self.len() == other.len()
            && self
                .iter()
                .zip(other.iter())
                .all(|(me, them)| me.with_ref(|me| me == them))
    }
}

macro_rules! as_slice_eq {
    ($t:ty $(,$N:tt)?) => {
        impl<T, U $(,const $N: usize)?> PartialEq<$t> for Slice<U>
        where
            U: Soapy + PartialEq<T>,
        {
            fn eq(&self, other: &$t) -> bool {
                self.eq(other.as_slice())
            }
        }

        impl<T, U $(,const $N: usize)?> PartialEq<Slice<U>> for $t
        where
            T: PartialEq<U>,
            U: Soapy,
        {
            fn eq(&self, other: &Slice<U>) -> bool {
                self.as_slice().eq(other)
            }
        }
    };
}

as_slice_eq!([T; N], N);
as_slice_eq!(Vec<T>);

macro_rules! trivial_ref_eq {
    ($t:ty $(,$N:tt)?) => {
        impl<T, U $(,const $N: usize)?> PartialEq<$t> for Slice<U>
        where
            U: Soapy + PartialEq<T>,
        {
            fn eq(&self, other: &$t) -> bool {
                self.eq(*other)
            }
        }

        impl<T, U $(,const $N: usize)?> PartialEq<Slice<U>> for $t
        where
            T: PartialEq<U>,
            U: Soapy,
        {
            fn eq(&self, other: &Slice<U>) -> bool {
                (**self).eq(other)
            }
        }
    };
}

trivial_ref_eq!(&[T]);
trivial_ref_eq!(&mut [T]);
trivial_ref_eq!(&[T; N], N);
trivial_ref_eq!(&mut [T; N], N);

macro_rules! eq_for_slice_ref {
    ($t:ty) => {
        eq_for_slice_ref!($t, Vec<U>, U);
        eq_for_slice_ref!($t, [U], U);
        eq_for_slice_ref!($t, [U; N], U, const N: usize);
        eq_for_slice_ref!($t, Slice<U>, U: Soapy);
        eq_for_slice_ref!($t, SliceRef<'_, U>, U: Soapy);
        eq_for_slice_ref!($t, SliceMut<'_, U>, U: Soapy);
        eq_for_slice_ref!($t, Soa<U>, U: Soapy);
    };

    ($t:ty, $s:ty, $($b:tt)+) => {
        impl<T, $($b)+> PartialEq<$s> for $t
        where
            T: Soapy + PartialEq<U>,
        {
            fn eq(&self, other: &$s) -> bool {
                <Slice<T> as PartialEq<$s>>::eq(*self, other)
            }
        }
    };
}

eq_for_slice_ref!(&Slice<T>);
eq_for_slice_ref!(&mut Slice<T>);

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

impl<T, U> PartialOrd<Slice<U>> for Slice<T>
where
    T: Soapy + PartialOrd<U>,
    U: Soapy,
{
    fn partial_cmp(&self, other: &Slice<U>) -> Option<Ordering> {
        match self
            .iter()
            .zip(other.iter())
            .try_fold(Ordering::Equal, |_, (a, b)| match a.partial_cmp(&b) {
                ord @ (None | Some(Ordering::Less | Ordering::Greater)) => ControlFlow::Break(ord),
                Some(Ordering::Equal) => ControlFlow::Continue(self.len().cmp(&other.len())),
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
                Ordering::Equal => ControlFlow::Continue(self.len().cmp(&other.len())),
            }) {
            ControlFlow::Continue(ord) | ControlFlow::Break(ord) => ord,
        }
    }
}

impl<T> Hash for Slice<T>
where
    T: Soapy + Hash,
{
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.len().hash(state);
        for el in self.iter() {
            el.with_ref(|el| el.hash(state))
        }
    }
}

impl<T> Deref for Slice<T>
where
    T: Soapy,
{
    type Target = T::Deref;

    fn deref(&self) -> &Self::Target {
        <T::Deref as SoaDeref>::from_slice(self)
    }
}

impl<T> DerefMut for Slice<T>
where
    T: Soapy,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        <T::Deref as SoaDeref>::from_slice_mut(self)
    }
}

impl<T> AsRef<Slice<T>> for Slice<T>
where
    T: Soapy,
{
    fn as_ref(&self) -> &Self {
        self
    }
}

impl<T> AsMut<Slice<T>> for Slice<T>
where
    T: Soapy,
{
    fn as_mut(&mut self) -> &mut Self {
        self
    }
}

use crate::{index::SoaIndex, Iter, IterMut};
use soapy_shared::{SliceData, SoaRaw, Soapy};
use std::{
    cmp::Ordering,
    fmt::{self, Formatter},
    hash::{Hash, Hasher},
    marker::PhantomData,
    mem::{transmute, ManuallyDrop},
    ops::{ControlFlow, Deref, DerefMut},
};

/// A growable array type that stores the values for each field of `T`
/// contiguously.
#[repr(transparent)]
pub struct Slice<T>(pub(crate) SliceData<T::Raw>)
where
    T: Soapy;

unsafe impl<T> Send for Slice<T> where T: Send + Soapy {}
unsafe impl<T> Sync for Slice<T> where T: Sync + Soapy {}

impl<T> Slice<T>
where
    T: Soapy,
{
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
    pub fn empty() -> Self {
        Self(SliceData {
            len: 0,
            raw: <T::Raw as SoaRaw>::dangling(),
        })
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
        self.0.len
    }

    /// Returns true if the container contains no elements.
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
    pub fn is_empty(&self) -> bool {
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
    pub(crate) unsafe fn from_raw_parts(raw: T::Raw, length: usize) -> Self {
        Self(SliceData { len: length, raw })
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
    /// # #[extra_impl(Debug, PartialEq)]
    /// # struct Foo(usize);
    /// let soa = soa![Foo(1), Foo(2), Foo(4)];
    /// let mut iter = soa.iter();
    /// assert_eq!(iter.next(), Some(FooSoaRef(&1)));
    /// assert_eq!(iter.next(), Some(FooSoaRef(&2)));
    /// assert_eq!(iter.next(), Some(FooSoaRef(&4)));
    /// assert_eq!(iter.next(), None);
    /// ```
    pub fn iter(&self) -> Iter<T> {
        Iter {
            start: 0,
            end: self.0.len,
            raw: self.0.raw,
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
            start: 0,
            end: self.0.len,
            raw: self.0.raw,
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
        for i in 0..self.0.len {
            let element = ManuallyDrop::new(unsafe { self.0.raw.get(i) });
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
    /// [`try_fold`]: Slice::try_fold
    /// [`Zip`]: std::iter::Zip
    /// [`Continue`]: ControlFlow::Continue
    /// [`Break`]: ControlFlow::Break
    /// [`WithRef`]: crate::WithRef
    pub fn try_fold_zip<F, B>(&self, other: &Self, init: B, mut f: F) -> B
    where
        F: FnMut(B, &T, &T) -> ControlFlow<B, B>,
    {
        let mut acc = init;
        let len = self.0.len.min(other.0.len);
        for i in 0..len {
            let a = ManuallyDrop::new(unsafe { self.0.raw.get(i) });
            let b = ManuallyDrop::new(unsafe { other.0.raw.get(i) });
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
    /// # #[extra_impl(PartialEq, Debug)]
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
    /// if let Some(elem) = soa.get_mut(1) {
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
    /// # #[extra_impl(Debug, PartialEq)]
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
    /// # #[extra_impl(Debug, PartialEq)]
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
        if a >= self.0.len || b >= self.0.len {
            panic!("index out of bounds");
        }

        unsafe {
            let tmp = self.0.raw.get(a);
            self.0.raw.copy(b, a, 1);
            self.0.raw.set(b, tmp);
        }
    }

    /// Returns the first element of the slice, or None if empty.
    ///
    /// # Examples
    ///
    /// ```
    /// # use soapy::{Soa, Soapy, soa};
    /// # #[derive(Soapy, Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
    /// # #[extra_impl(Debug, PartialEq)]
    /// # struct Foo(usize);
    /// let soa = soa![Foo(10), Foo(40), Foo(30)];
    /// assert_eq!(soa.first(), Some(FooSoaRef(&10)));
    ///
    /// let soa = Soa::<Foo>::new();
    /// assert_eq!(soa.first(), None);
    /// ```
    pub fn first(&self) -> Option<T::Ref<'_>> {
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
    /// if let Some(first) = soa.first_mut() {
    ///     *first.0 = 5;
    /// }
    /// assert_eq!(soa, [Foo(5), Foo(1), Foo(2)]);
    /// ```
    pub fn first_mut(&mut self) -> Option<T::RefMut<'_>> {
        self.get_mut(0)
    }

    /// Returns the last element of the slice, or None if empty.
    ///
    /// # Examples
    ///
    /// ```
    /// # use soapy::{Soa, Soapy, soa};
    /// # #[derive(Soapy, Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
    /// # #[extra_impl(Debug, PartialEq)]
    /// # struct Foo(usize);
    /// let soa = soa![Foo(10), Foo(40), Foo(30)];
    /// assert_eq!(soa.last(), Some(FooSoaRef(&30)));
    ///
    /// let soa = Soa::<Foo>::new();
    /// assert_eq!(soa.last(), None);
    /// ```
    pub fn last(&self) -> Option<T::Ref<'_>> {
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
    /// if let Some(last) = soa.last_mut() {
    ///     *last.0 = 5;
    /// }
    /// assert_eq!(soa, [Foo(0), Foo(1), Foo(5)]);
    /// ```
    pub fn last_mut(&mut self) -> Option<T::RefMut<'_>> {
        self.get_mut(self.len().saturating_sub(1))
    }
}

impl<'a, T> IntoIterator for &'a Slice<T>
where
    T: Soapy,
{
    type Item = T::Ref<'a>;
    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        Iter {
            start: 0,
            end: self.0.len,
            raw: self.0.raw,
            _marker: PhantomData,
        }
    }
}

impl<'a, T> IntoIterator for &'a mut Slice<T>
where
    T: Soapy,
{
    type Item = T::RefMut<'a>;
    type IntoIter = IterMut<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        IterMut {
            start: 0,
            end: self.0.len,
            raw: self.0.raw,
            _marker: PhantomData,
        }
    }
}

impl<T> PartialEq for Slice<T>
where
    T: Soapy + PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        if self.0.len != other.0.len {
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

impl<T> Eq for Slice<T> where T: Soapy + Eq {}

impl<T> PartialEq<[T]> for Slice<T>
where
    T: Soapy + PartialEq,
{
    fn eq(&self, other: &[T]) -> bool {
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

impl<T> fmt::Debug for Slice<T>
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

impl<T> PartialOrd for Slice<T>
where
    T: Soapy + PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.try_fold_zip(other, Some(Ordering::Equal), |_, a, b| {
            match a.partial_cmp(b) {
                ord @ (None | Some(Ordering::Less | Ordering::Greater)) => ControlFlow::Break(ord),
                Some(Ordering::Equal) => ControlFlow::Continue(Some(self.0.len.cmp(&other.0.len))),
            }
        })
    }
}

impl<T> Ord for Slice<T>
where
    T: Soapy + Ord,
{
    fn cmp(&self, other: &Self) -> Ordering {
        self.try_fold_zip(other, Ordering::Equal, |_, a, b| match a.cmp(b) {
            ord @ (Ordering::Greater | Ordering::Less) => ControlFlow::Break(ord),
            Ordering::Equal => ControlFlow::Continue(self.0.len.cmp(&other.0.len)),
        })
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
        self.0.len.hash(state);
        self.for_each(|item| item.hash(state));
    }
}

impl<T> Clone for Slice<T>
where
    T: Soapy,
{
    fn clone(&self) -> Self {
        Self(self.0)
    }
}

impl<T> Copy for Slice<T> where T: Soapy {}

impl<T> Deref for Slice<T>
where
    T: Soapy,
{
    type Target = T::Deref;

    fn deref(&self) -> &Self::Target {
        unsafe { transmute(self) }
    }
}

impl<T> DerefMut for Slice<T>
where
    T: Soapy,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { transmute(self) }
    }
}

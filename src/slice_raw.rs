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
#[derive(Copy, Clone)]
#[repr(transparent)]
pub struct SliceRaw<T>(pub(crate) SliceData<T::Raw>)
where
    T: Soapy;

unsafe impl<T> Send for SliceRaw<T> where T: Send + Soapy {}
unsafe impl<T> Sync for SliceRaw<T> where T: Sync + Soapy {}

impl<T> SliceRaw<T>
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
    pub fn into_raw_parts(self) -> (T::Raw, usize) {
        (self.0.raw, self.0.len)
    }

    /// Creates a `Soa<T>` from a pointer, a length, and a capacity.
    ///
    /// # Safety
    ///
    /// This is highly unsafe due to the number of invariants that aren't
    /// checked. Given that many of these invariants are private implementation
    /// details of [`Raw`], it is better not to uphold them manually. Rather,
    /// it only valid to call this method with the output of a previous call to
    /// [`Soa::into_raw_parts`].
    pub unsafe fn from_raw_parts(raw: T::Raw, length: usize) -> Self {
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
            // SAFETY:
            // Okay to construct an element and take its reference, so long as
            // we don't run its destructor.
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
        let len = self.0.len.min(other.0.len);
        for i in 0..len {
            // SAFETY:
            // Okay to construct an element and take its reference, so long as
            // we don't run its destructor.
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
    /// # use soapy::{Soa, Soapy, soa, WithRef};
    /// # #[derive(Soapy, Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
    /// # #[extra_impl(PartialEq, Debug)]
    /// # struct Foo(usize);
    /// let soa = soa![Foo(10), Foo(40), Foo(30)];
    /// assert_eq!(soa.get(1), Some(FooSoaRef(&40)));
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
    /// [`get`]: Soa::get
    pub fn get_mut<I>(&mut self, index: I) -> Option<I::OutputMut<'_>>
    where
        I: SoaIndex<T>,
    {
        index.get_mut(self)
    }

    /// Returns a clone of the element at the given index.
    ///
    /// This is equivalent to [`index`] followed by a [`clone`]. Prefer
    /// [`nth_copied`] for types that support it.
    ///
    /// # Panics
    ///
    /// Panics if `index >= len`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use std::fmt;
    /// # use soapy::{Soa, Soapy, soa, WithRef};
    /// # #[derive(Soapy, Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
    /// # struct Foo(usize);
    /// let soa = soa![Foo(10), Foo(40), Foo(30), Foo(90)];
    /// assert_eq!(soa.nth_cloned(1), Foo(40));
    /// assert_eq!(soa.nth_cloned(3), Foo(90));
    /// ```
    ///
    /// [`index`]: std::ops::Index::index
    /// [`clone`]: std::clone::Clone::clone
    /// [`nth_copied`]: Soa::nth_copied
    pub fn nth_cloned(&self, index: usize) -> T
    where
        T: Clone,
    {
        if index >= self.0.len {
            panic!("index out of bounds");
        }
        let el = ManuallyDrop::new(unsafe { self.0.raw.get(index) });
        el.deref().clone()
    }

    /// Returns a copy of the element at the given index.
    ///
    /// This is equivalent to [`index`] except that it returns a copy rather
    /// than a reference. Prefer this over [`nth_cloned`] for types that support
    /// it.
    ///
    /// # Panics
    ///
    /// Panics if `index >= len`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use std::fmt;
    /// # use soapy::{Soa, Soapy, soa, WithRef};
    /// # #[derive(Soapy, Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
    /// # struct Foo(usize);
    /// let soa = soa![Foo(10), Foo(40), Foo(30), Foo(90)];
    /// assert_eq!(soa.nth_copied(1), Foo(40));
    /// assert_eq!(soa.nth_copied(3), Foo(90));
    /// ```
    ///
    /// [`index`]: std::ops::Index::index
    /// [`nth_cloned`]: Soa::nth_cloned
    pub fn nth_copied(&self, index: usize) -> T
    where
        T: Copy,
    {
        if index >= self.0.len {
            panic!("index out of bounds");
        }
        unsafe { self.0.raw.get(index) }
    }

    /// Returns a reference to the element at the given index.
    ///
    /// This is functionally equivalent to [`Index`], which is not implementable
    /// for this type.
    ///
    /// # Panics
    ///
    /// Panics if `index >= len`.
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
    /// assert_eq!(soa.nth(1), FooSoaRef(&40));
    /// assert_eq!(soa.nth(3), FooSoaRef(&90));
    /// ```
    ///
    /// [`Index`]: std::ops::Index
    pub fn nth(&self, index: usize) -> T::Ref<'_> {
        if index >= self.0.len {
            panic!("index out of bounds");
        }
        unsafe { self.0.raw.get_ref(index) }
    }

    /// Returns a mutable reference to the element at the given index.
    ///
    /// This is functionally equivalent to [`IndexMut`], which is not
    /// implementable for this type.
    ///
    /// # Panics
    ///
    /// Panics if `index >= len`.
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
    /// *soa.nth_mut(1).0 = 42;
    /// assert_eq!(soa, [Foo(10), Foo(42), Foo(30)]);
    /// ```
    ///
    /// [`IndexMut`]: std::ops::Index
    pub fn nth_mut(&mut self, index: usize) -> T::RefMut<'_> {
        if index >= self.0.len {
            panic!("index out of bounds");
        }
        unsafe { self.0.raw.get_mut(index) }
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

    /// Returns the first element of the slice, or None if empty
    ///
    /// # Examples
    ///
    /// ```
    /// # use soapy::{Soa, Soapy, soa};
    /// # #[derive(Soapy, Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
    /// # #[extra_impl(Debug, PartialEq)]
    /// # struct Foo(usize);
    /// let mut soa = Soa::<Foo>::new();
    /// assert_eq!(soa.first(), None);
    /// soa.push(Foo(10));
    /// soa.push(Foo(20));
    /// assert_eq!(soa.first(), Some(FooSoaRef(&10)));
    /// ```
    pub fn first(&self) -> Option<T::Ref<'_>> {
        self.get(0)
    }
}

impl<'a, T> IntoIterator for &'a SliceRaw<T>
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

impl<'a, T> IntoIterator for &'a mut SliceRaw<T>
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

impl<T> PartialEq for SliceRaw<T>
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

impl<T, R> PartialEq<R> for SliceRaw<T>
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

impl<T> Eq for SliceRaw<T> where T: Soapy + Eq {}

impl<T> fmt::Debug for SliceRaw<T>
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

impl<T> PartialOrd for SliceRaw<T>
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

impl<T> Ord for SliceRaw<T>
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

impl<T> Default for SliceRaw<T>
where
    T: Soapy,
{
    fn default() -> Self {
        Self::empty()
    }
}

impl<T> Hash for SliceRaw<T>
where
    T: Soapy + Hash,
{
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.len.hash(state);
        self.for_each(|item| item.hash(state));
    }
}

impl<T> Deref for SliceRaw<T>
where
    T: Soapy,
{
    type Target = T::Deref;

    fn deref(&self) -> &Self::Target {
        unsafe { transmute(self) }
    }
}

impl<T> DerefMut for SliceRaw<T>
where
    T: Soapy,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { transmute(self) }
    }
}

use crate::{Slice, SoaRaw, Soars};
use std::{fmt::Debug, iter::FusedIterator, marker::PhantomData};

/// Used by [`IterRaw`] to get the first element from a [`SoaRaw`] in different
/// forms.
pub trait IterRawAdapter<T>
where
    T: Soars,
{
    /// The desired form of the first element, such as a [`Ref`] or [`RefMut`].
    type Item;

    /// Gets the first element of `raw` in the desired form.
    ///
    /// # Safety
    ///
    /// The caller must ensure that `T` is zero-sized or `raw` contains at least
    /// one initialized element.
    unsafe fn item_from_raw(raw: T::Raw) -> Self::Item;
}

pub struct IterRaw<T, A>
where
    T: Soars,
    A: IterRawAdapter<T>,
{
    pub(crate) slice: Slice<T, ()>,
    pub(crate) len: usize,
    pub(crate) adapter: PhantomData<A>,
}

impl<T, A> IterRaw<T, A>
where
    T: Soars,
    A: IterRawAdapter<T>,
{
    /// Gets the remaining items as a slice.
    ///
    /// # Safety
    ///
    /// This function returns an unbounded lifetime to which the caller must
    /// apply a suitable bound that respects the aliasing rules.
    pub(crate) unsafe fn as_slice<'a>(&self) -> &'a Slice<T> {
        unsafe { self.slice.as_unsized(self.len) }
    }

    /// Gets the remaining items as a mutable slice.
    ///
    /// # Safety
    ///
    /// This function returns an unbounded lifetime to which the caller must
    /// apply a suitable bound that respects the aliasing rules.
    pub(crate) unsafe fn as_mut_slice(&mut self) -> &mut Slice<T> {
        unsafe { self.slice.as_unsized_mut(self.len) }
    }
}

impl<T, A> Clone for IterRaw<T, A>
where
    T: Soars,
    A: IterRawAdapter<T>,
{
    fn clone(&self) -> Self {
        *self
    }
}

impl<T, A> Copy for IterRaw<T, A>
where
    T: Soars,
    A: IterRawAdapter<T>,
{
}

impl<T, A> Debug for IterRaw<T, A>
where
    T: Soars,
    A: IterRawAdapter<T>,
    for<'a> T::Ref<'a>: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // SAFETY: len is valid for this slice
        unsafe { self.slice.as_unsized(self.len).fmt(f) }
    }
}

impl<T, A> Iterator for IterRaw<T, A>
where
    T: Soars,
    A: IterRawAdapter<T>,
{
    type Item = A::Item;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.len == 0 {
            None
        } else {
            let raw = self.slice.raw();
            // SAFETY: Our length is nonzero so raw points to a valid element
            let out = Some(unsafe { A::item_from_raw(raw) });
            // SAFETY: There is at least one element in raw
            self.slice.raw = unsafe { raw.offset(1) };
            self.len -= 1;
            out
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }

    fn count(self) -> usize
    where
        Self: Sized,
    {
        self.len
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        if n >= self.len {
            self.len = 0;
            None
        } else {
            let raw = self.slice.raw();
            // SAFETY: n < length so we can offset by at least n
            let raw = unsafe { raw.offset(n) };
            // SAFETY: We offset by less than length
            // so `raw` points to at least one item.
            let out = Some(unsafe { A::item_from_raw(raw) });

            // nth(n) consumes item n so we need to advance one more
            self.len -= n + 1;
            // SAFETY: n < length so we can offset by at least n+1
            // (we already offset by n)
            self.slice.raw = unsafe { raw.offset(1) };

            out
        }
    }

    fn last(self) -> Option<Self::Item>
    where
        Self: Sized,
    {
        if self.len == 0 {
            None
        } else {
            let raw = self.slice.raw();
            // SAFETY: Offsetting to one before length leaves one item to read
            Some(unsafe { A::item_from_raw(raw.offset(self.len - 1)) })
        }
    }

    fn fold<B, F>(self, init: B, mut f: F) -> B
    where
        Self: Sized,
        F: FnMut(B, Self::Item) -> B,
    {
        let Self {
            slice,
            len,
            adapter: _,
        } = self;
        let raw = slice.raw();
        if len == 0 {
            return init;
        }
        let mut acc = init;
        let mut i = 0;
        loop {
            // SAFETY: i < len so offsetting by i leaves at least one item to read
            let raw = unsafe { A::item_from_raw(raw.offset(i)) };
            acc = f(acc, raw);

            // SAFETY: see std::slice::Iter::fold
            //
            // `i` can't overflow since it'll only reach usize::MAX if the
            // slice had that length, in which case we'll break out of the loop
            // after the increment
            i = unsafe { i.unchecked_add(1) };

            if i == len {
                break;
            }
        }
        acc
    }
}

impl<T, A> DoubleEndedIterator for IterRaw<T, A>
where
    T: Soars,
    A: IterRawAdapter<T>,
{
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.len == 0 {
            None
        } else {
            self.len -= 1;
            // SAFETY: After adjusting len, len refers to the last item at the
            // previous size. Offsetting by len will point to at least one item.
            Some(unsafe { A::item_from_raw(self.slice.raw.offset(self.len)) })
        }
    }

    #[inline]
    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        if n >= self.len {
            self.len = 0;
            None
        } else {
            self.len -= n + 1;
            // SAFETY: After adjusting len, len refers to the last item at the
            // previous size. Offsetting by len will point to at least one item.
            Some(unsafe { A::item_from_raw(self.slice.raw.offset(self.len)) })
        }
    }
}

impl<T, A> FusedIterator for IterRaw<T, A>
where
    T: Soars,
    A: IterRawAdapter<T>,
{
}

impl<T, A> ExactSizeIterator for IterRaw<T, A>
where
    T: Soars,
    A: IterRawAdapter<T>,
{
}

macro_rules! iter_with_raw {
    ($t:ty $(,$lifetime:tt)?) => {
        impl<$($lifetime,)? T> Iterator for $t
        where
            T: $($lifetime +)? Soars,
        {
            type Item = <$t as IterRawAdapter<T>>::Item;

            #[inline]
            fn next(&mut self) -> Option<Self::Item> {
                self.iter_raw.next()
            }

            fn size_hint(&self) -> (usize, Option<usize>) {
                self.iter_raw.size_hint()
            }

            fn count(self) -> usize
            where
                Self: Sized,
            {
                self.iter_raw.count()
            }

            fn nth(&mut self, n: usize) -> Option<Self::Item> {
                self.iter_raw.nth(n)
            }

            fn last(self) -> Option<Self::Item>
            where
                Self: Sized,
            {
                self.iter_raw.last()
            }

            fn fold<B, F>(self, init: B, f: F) -> B
            where
                Self: Sized,
                F: FnMut(B, Self::Item) -> B,
            {
                self.iter_raw.fold(init, f)
            }
        }

        impl<$($lifetime,)? T> DoubleEndedIterator for $t
        where
            T: $($lifetime +)? Soars,
        {
            #[inline]
            fn next_back(&mut self) -> Option<Self::Item> {
                self.iter_raw.next_back()
            }
        }

        impl<$($lifetime,)? T> FusedIterator for $t where T: $($lifetime +)? Soars {}
        impl<$($lifetime,)? T> ExactSizeIterator for $t where T: $($lifetime +)? Soars {}

        impl<$($lifetime,)? T> AsRef<Slice<T>> for $t where T: $($lifetime +)? Soars {
            fn as_ref(&self) -> &Slice<T> {
                // SAFETY: The returned lifetime is bound to self
                unsafe { self.iter_raw.as_slice() }
            }
        }
    };
}

pub(crate) use iter_with_raw;

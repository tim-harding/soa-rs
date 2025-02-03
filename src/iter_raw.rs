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

    fn last(mut self) -> Option<Self::Item>
    where
        Self: Sized,
    {
        self.next_back()
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

            fn last(self) -> Option<Self::Item>
            where
                Self: Sized,
            {
                self.iter_raw.last()
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

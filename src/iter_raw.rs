use crate::{Slice, SoaRaw, Soars};
use std::{fmt::Debug, iter::FusedIterator, marker::PhantomData};

pub trait IterRawAdapter<T>
where
    T: Soars,
{
    type Item;
    fn item_from_raw(raw: T::Raw) -> Self::Item;
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
    pub(crate) unsafe fn as_slice<'a>(&self) -> &'a Slice<T> {
        unsafe { self.slice.as_unsized(self.len) }
    }

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
            self.len -= 1;
            let out = Some(A::item_from_raw(self.slice.raw()));
            self.slice.raw = unsafe { self.slice.raw().offset(1) };
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
            let out = A::item_from_raw(self.slice.raw());
            self.len -= n + 1;
            self.slice.raw = unsafe { self.slice.raw().offset(n + 1) };
            Some(out)
        }
    }

    fn last(self) -> Option<Self::Item>
    where
        Self: Sized,
    {
        if self.len == 0 {
            None
        } else {
            Some(A::item_from_raw(unsafe {
                self.slice.raw().offset(self.len - 1)
            }))
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
        if len == 0 {
            return init;
        }
        let mut acc = init;
        let mut i = 0;
        loop {
            acc = f(acc, A::item_from_raw(unsafe { slice.raw().offset(i) }));
            i += 1;
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
            Some(unsafe {
                A::item_from_raw(self.slice.as_unsized(self.len).raw().offset(self.len))
            })
        }
    }

    #[inline]
    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        if n >= self.len {
            self.len = 0;
            None
        } else {
            self.len -= n + 1;
            Some(A::item_from_raw(unsafe {
                self.slice.as_unsized(self.len).raw().offset(self.len)
            }))
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
                unsafe { self.iter_raw.as_slice() }
           }
        }
    };
}

pub(crate) use iter_with_raw;

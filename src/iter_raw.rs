use crate::{Slice, SoaRaw, Soapy};
use std::{
    iter::FusedIterator,
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

pub trait IterRawAdapter<T>
where
    T: Soapy,
{
    type Item;
    fn item_from_raw(raw: T::Raw) -> Self::Item;
}

#[derive(Debug)]
pub struct IterRaw<T, A>
where
    T: Soapy,
    A: IterRawAdapter<T>,
{
    pub(crate) slice: Slice<T>,
    pub(crate) adapter: PhantomData<A>,
}

impl<T, A> Clone for IterRaw<T, A>
where
    T: Soapy,
    A: IterRawAdapter<T>,
{
    fn clone(&self) -> Self {
        *self
    }
}

impl<T, A> Copy for IterRaw<T, A>
where
    T: Soapy,
    A: IterRawAdapter<T>,
{
}

impl<T, A> Iterator for IterRaw<T, A>
where
    T: Soapy,
    A: IterRawAdapter<T>,
{
    type Item = A::Item;

    fn next(&mut self) -> Option<Self::Item> {
        if self.len == 0 {
            None
        } else {
            self.len -= 1;
            let out = Some(A::item_from_raw(self.raw));
            self.raw = unsafe { self.raw.offset(1) };
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

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        if n >= self.len {
            self.len = 0;
            None
        } else {
            let out = A::item_from_raw(self.raw);
            self.len -= n + 1;
            self.raw = unsafe { self.raw.offset(n + 1) };
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
            Some(A::item_from_raw(unsafe { self.raw.offset(self.len - 1) }))
        }
    }

    fn fold<B, F>(self, init: B, mut f: F) -> B
    where
        Self: Sized,
        F: FnMut(B, Self::Item) -> B,
    {
        let Self {
            slice: Slice { raw, len },
            adapter: _,
        } = self;
        if len == 0 {
            return init;
        }
        let mut acc = init;
        let mut i = 0;
        loop {
            acc = f(acc, A::item_from_raw(unsafe { raw.offset(i) }));
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
    T: Soapy,
    A: IterRawAdapter<T>,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.len == 0 {
            None
        } else {
            self.len -= 1;
            Some(unsafe { A::item_from_raw(self.raw.offset(self.len)) })
        }
    }

    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        if n >= self.len {
            self.len = 0;
            None
        } else {
            self.len -= n + 1;
            Some(A::item_from_raw(unsafe { self.raw.offset(self.len) }))
        }
    }
}

impl<T, A> FusedIterator for IterRaw<T, A>
where
    T: Soapy,
    A: IterRawAdapter<T>,
{
}

impl<T, A> ExactSizeIterator for IterRaw<T, A>
where
    T: Soapy,
    A: IterRawAdapter<T>,
{
}

impl<T, A> Deref for IterRaw<T, A>
where
    T: Soapy,
    A: IterRawAdapter<T>,
{
    type Target = Slice<T>;

    fn deref(&self) -> &Self::Target {
        &self.slice
    }
}

impl<T, A> DerefMut for IterRaw<T, A>
where
    T: Soapy,
    A: IterRawAdapter<T>,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.slice
    }
}

macro_rules! iter_with_raw {
    ($t:ty $(,$lifetime:tt)?) => {
        impl<$($lifetime,)? T> Iterator for $t
        where
            T: $($lifetime +)? Soapy,
        {
            type Item = <$t as IterRawAdapter<T>>::Item;

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
            T: $($lifetime +)? Soapy,
        {
            fn next_back(&mut self) -> Option<Self::Item> {
                self.iter_raw.next_back()
            }
        }

        impl<$($lifetime,)? T> FusedIterator for $t where T: $($lifetime +)? Soapy {}
        impl<$($lifetime,)? T> ExactSizeIterator for $t where T: $($lifetime +)? Soapy {}

        impl<$($lifetime,)? T> AsRef<Slice<T>> for $t where T: $($lifetime +)? Soapy {
            fn as_ref(&self) -> &Slice<T> {
                &self.iter_raw.slice
            }
        }
    };
}

pub(crate) use iter_with_raw;

use crate::{Iter, Slice};
use soapy_shared::Soapy;
use std::{
    cmp::Ordering,
    fmt::{self, Debug, Formatter},
    hash::{Hash, Hasher},
    marker::PhantomData,
    ops::Deref,
};

pub struct SliceRef<'a, T>(pub(crate) Slice<T>, pub(crate) PhantomData<&'a T>)
where
    T: 'a + Soapy;

impl<'a, T> Clone for SliceRef<'a, T>
where
    T: 'a + Soapy,
{
    fn clone(&self) -> Self {
        Self(self.0, PhantomData)
    }
}

impl<'a, T> Copy for SliceRef<'a, T> where T: 'a + Soapy {}

impl<'a, T> AsRef<Slice<T>> for SliceRef<'a, T>
where
    T: 'a + Soapy,
{
    fn as_ref(&self) -> &Slice<T> {
        &self.0
    }
}

impl<'a, T> Deref for SliceRef<'a, T>
where
    T: 'a + Soapy,
{
    type Target = Slice<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a, T> IntoIterator for SliceRef<'a, T>
where
    T: 'a + Soapy,
{
    type Item = T::Ref<'a>;
    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        Iter {
            start: 0,
            end: self.len(),
            raw: self.0 .0.raw,
            _marker: PhantomData,
        }
    }
}

impl<'a, T, R> PartialEq<R> for SliceRef<'a, T>
where
    T: 'a + Soapy + PartialEq,
    R: AsRef<[T]>,
{
    fn eq(&self, other: &R) -> bool {
        self.0.eq(other)
    }
}

impl<'a, T> PartialEq<Slice<T>> for SliceRef<'a, T>
where
    T: 'a + Soapy + PartialEq,
{
    fn eq(&self, other: &Slice<T>) -> bool {
        self.0.eq(other)
    }
}

impl<'a, T> PartialEq for SliceRef<'a, T>
where
    T: 'a + Soapy + PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0)
    }
}

impl<'a, T> Eq for SliceRef<'a, T> where T: 'a + Soapy + Eq {}

impl<'a, T> Debug for SliceRef<'a, T>
where
    T: 'a + Soapy + Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        (&self.0).fmt(f)
    }
}

impl<'a, T> PartialOrd for SliceRef<'a, T>
where
    T: 'a + Soapy + PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        (&self.0).partial_cmp(&other.0)
    }
}

impl<'a, T> Ord for SliceRef<'a, T>
where
    T: 'a + Soapy + Ord,
{
    fn cmp(&self, other: &Self) -> Ordering {
        (&self.0).cmp(&other.0)
    }
}

impl<'a, T> Hash for SliceRef<'a, T>
where
    T: 'a + Soapy + Hash,
{
    fn hash<H: Hasher>(&self, state: &mut H) {
        (&self.0).hash(state)
    }
}

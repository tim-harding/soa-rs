use crate::{eq_impl, IterMut, Slice, Soapy};
use std::{
    cmp::Ordering,
    fmt::{self, Debug, Formatter},
    hash::{Hash, Hasher},
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

pub struct SliceMut<'a, T>(pub(crate) Slice<T>, pub(crate) PhantomData<&'a mut T>)
where
    T: 'a + Soapy;

impl<'a, T> AsRef<Slice<T>> for SliceMut<'a, T>
where
    T: 'a + Soapy,
{
    fn as_ref(&self) -> &Slice<T> {
        &self.0
    }
}

impl<'a, T> AsMut<Slice<T>> for SliceMut<'a, T>
where
    T: 'a + Soapy,
{
    fn as_mut(&mut self) -> &mut Slice<T> {
        &mut self.0
    }
}

impl<'a, T> Deref for SliceMut<'a, T>
where
    T: 'a + Soapy,
{
    type Target = Slice<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a, T> DerefMut for SliceMut<'a, T>
where
    T: 'a + Soapy,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<'a, T> IntoIterator for SliceMut<'a, T>
where
    T: 'a + Soapy,
{
    type Item = T::RefMut<'a>;
    type IntoIter = IterMut<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        IterMut {
            start: 0,
            end: self.len(),
            raw: self.0.raw,
            _marker: PhantomData,
        }
    }
}

eq_impl::impl_for!(SliceMut<'a, T>);

impl<'a, T> Debug for SliceMut<'a, T>
where
    T: 'a + Soapy + Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        (&self.0).fmt(f)
    }
}

impl<'a, T> PartialOrd for SliceMut<'a, T>
where
    T: 'a + Soapy + PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        (&self.0).partial_cmp(&other.0)
    }
}

impl<'a, T> Ord for SliceMut<'a, T>
where
    T: 'a + Soapy + Ord,
{
    fn cmp(&self, other: &Self) -> Ordering {
        (&self.0).cmp(&other.0)
    }
}

impl<'a, T> Hash for SliceMut<'a, T>
where
    T: 'a + Soapy + Hash,
{
    fn hash<H: Hasher>(&self, state: &mut H) {
        (&self.0).hash(state)
    }
}

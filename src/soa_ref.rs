use crate::{Soapy, WithRef};
use std::{
    fmt::Debug,
    hash::{Hash, Hasher},
    ops::{Deref, DerefMut},
};

/// An immutable reference to an element of a [`Soa`].
///
/// This is similar to `&T` for an element of a `&[T]`. However, since the
/// fields are stored separately for SoA, we need a different type that has
/// references to each of fields for the element. This is a convenience wrapper
/// over [`Soapy::Ref`], which is implemented for each type that derives
/// [`Soapy`]. It uses [`WithRef`] to provide implementations of common standard
/// library traits with less codegen and ceremony than doing the same in the
/// macro.
///
/// [`Soa`]: crate::Soa
#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct Ref<'a, T>(pub(crate) T::Ref<'a>)
where
    T: 'a + Soapy;

/// A mutable reference to an element of a [`Soa`].
///
/// This is similar to `&mut T` for an element of a `&mut [T]`. See [`Ref`] for
/// more details.
///
/// [`Soa`]: crate::Soa
#[repr(transparent)]
pub struct RefMut<'a, T>(pub(crate) T::RefMut<'a>)
where
    T: 'a + Soapy;

impl<'a, T> Deref for Ref<'a, T>
where
    T: Soapy,
{
    type Target = T::Ref<'a>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a, T> Deref for RefMut<'a, T>
where
    T: Soapy,
{
    type Target = T::RefMut<'a>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a, T> DerefMut for RefMut<'a, T>
where
    T: Soapy,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<'a, T> AsRef<T::Ref<'a>> for Ref<'a, T>
where
    T: Soapy,
{
    fn as_ref(&self) -> &T::Ref<'a> {
        &self.0
    }
}

impl<'a, T> AsRef<T::RefMut<'a>> for RefMut<'a, T>
where
    T: Soapy,
{
    fn as_ref(&self) -> &T::RefMut<'a> {
        &self.0
    }
}

impl<'a, T> AsMut<T::RefMut<'a>> for RefMut<'a, T>
where
    T: Soapy,
{
    fn as_mut(&mut self) -> &mut T::RefMut<'a> {
        &mut self.0
    }
}

macro_rules! ref_impls {
    ($t:ident) => {
        impl<'a, T> WithRef for $t<'a, T>
        where
            T: Soapy,
        {
            type Item = T;

            fn with_ref<F, U>(&self, f: F) -> U
            where
                F: FnOnce(&Self::Item) -> U,
            {
                self.0.with_ref(f)
            }
        }

        impl<'a, T> Debug for $t<'a, T>
        where
            T: Soapy + Debug,
        {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                self.with_ref(|me| me.fmt(f))
            }
        }

        impl<'a, T, U, R> PartialEq<R> for $t<'a, T>
        where
            T: Soapy + PartialEq<U>,
            R: WithRef<Item = U>,
        {
            fn eq(&self, other: &R) -> bool {
                self.with_ref(|me| other.with_ref(|them| me.eq(them)))
            }
        }

        impl<'a, T> Eq for $t<'a, T> where T: 'a + Soapy + Eq {}

        impl<'a, T, U, R> PartialOrd<R> for $t<'a, T>
        where
            T: Soapy + PartialOrd<U>,
            R: WithRef<Item = U>,
        {
            fn partial_cmp(&self, other: &R) -> Option<std::cmp::Ordering> {
                self.with_ref(|me| other.with_ref(|them| me.partial_cmp(them)))
            }
        }

        impl<'a, T> Ord for $t<'a, T>
        where
            T: Soapy + Ord,
        {
            fn cmp(&self, other: &Self) -> std::cmp::Ordering {
                self.with_ref(|me| other.with_ref(|them| me.cmp(them)))
            }
        }

        impl<'a, T> Hash for $t<'a, T>
        where
            T: Soapy + Hash,
        {
            fn hash<H: Hasher>(&self, state: &mut H) {
                self.with_ref(|me| me.hash(state))
            }
        }
    };
}

ref_impls!(Ref);
ref_impls!(RefMut);

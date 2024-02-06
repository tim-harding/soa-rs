use crate::{Soapy, WithRef};
use std::{
    fmt::Debug,
    hash::{Hash, Hasher},
    ops::{Deref, DerefMut},
};

#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct Ref<'a, T>(pub(crate) T::Ref<'a>)
where
    T: 'a + Soapy;

#[repr(transparent)]
pub struct RefMut<'a, T>(pub(crate) T::RefMut<'a>)
where
    T: 'a + Soapy;

impl<'a, T> Deref for Ref<'a, T>
where
    T: 'a + Soapy,
{
    type Target = T::Ref<'a>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a, T> Deref for RefMut<'a, T>
where
    T: 'a + Soapy,
{
    type Target = T::RefMut<'a>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a, T> DerefMut for RefMut<'a, T>
where
    T: 'a + Soapy,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

macro_rules! ref_impls {
    ($t:ident) => {
        impl<'a, T> WithRef for $t<'a, T>
        where
            T: 'a + Soapy,
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
            T: 'a + Soapy + Debug,
        {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                self.with_ref(|me| me.fmt(f))
            }
        }

        impl<'a, T, R> PartialEq<R> for $t<'a, T>
        where
            T: 'a + Soapy + PartialEq,
            R: WithRef<Item = T>,
        {
            fn eq(&self, other: &R) -> bool {
                self.with_ref(|me| other.with_ref(|them| me.eq(them)))
            }
        }

        impl<'a, T> Eq for $t<'a, T> where T: 'a + Soapy + Eq {}

        impl<'a, T, R> PartialOrd<R> for $t<'a, T>
        where
            T: 'a + Soapy + PartialOrd,
            R: WithRef<Item = T>,
        {
            fn partial_cmp(&self, other: &R) -> Option<std::cmp::Ordering> {
                self.with_ref(|me| other.with_ref(|them| me.partial_cmp(them)))
            }
        }

        impl<'a, T> Ord for $t<'a, T>
        where
            T: 'a + Soapy + Ord,
        {
            fn cmp(&self, other: &Self) -> std::cmp::Ordering {
                self.with_ref(|me| other.with_ref(|them| me.cmp(them)))
            }
        }

        impl<'a, T> Hash for $t<'a, T>
        where
            T: 'a + Soapy + Hash,
        {
            fn hash<H: Hasher>(&self, state: &mut H) {
                self.with_ref(|me| me.hash(state))
            }
        }
    };
}

ref_impls!(Ref);
ref_impls!(RefMut);

macro_rules! slice {
    ($u:ty, $t:ty) => {
        impl<'a, T> PartialEq<$t> for $u
        where
            T: 'a + Soapy + PartialEq,
        {
            fn eq(&self, other: &$t) -> bool {
                self.as_ref().eq(other)
            }
        }

        impl<'a, T> PartialEq<$u> for $t
        where
            T: 'a + Soapy + PartialEq,
        {
            fn eq(&self, other: &$u) -> bool {
                other.eq(self)
            }
        }
    };
}

pub(crate) use slice;

macro_rules! array {
    ($u:ty, $t:ty) => {
        impl<'a, T, const N: usize> PartialEq<$t> for $u
        where
            T: 'a + Soapy + PartialEq,
        {
            fn eq(&self, other: &$t) -> bool {
                self.as_ref().eq(other.as_slice())
            }
        }

        impl<'a, T, const N: usize> PartialEq<$u> for $t
        where
            T: 'a + Soapy + PartialEq,
        {
            fn eq(&self, other: &$u) -> bool {
                other.eq(self)
            }
        }
    };
}

pub(crate) use array;

macro_rules! impl_for {
    ($t:ty) => {
        $crate::eq_impl::slice!($t, Vec<T>);
        $crate::eq_impl::slice!($t, [T]);
        $crate::eq_impl::slice!($t, &[T]);
        $crate::eq_impl::slice!($t, &mut [T]);
        $crate::eq_impl::array!($t, [T; N]);
        $crate::eq_impl::array!($t, &[T; N]);
        $crate::eq_impl::array!($t, &mut [T; N]);

        impl<'a, T, R> PartialEq<R> for $t
        where
            T: 'a + Soapy + PartialEq,
            R: AsRef<Slice<T>>,
            Slice<T>: PartialEq,
        {
            fn eq(&self, other: &R) -> bool {
                self.as_ref().eq(other.as_ref())
            }
        }

        impl<'a, T> Eq for $t where T: 'a + Soapy + Eq {}
    };
}

pub(crate) use impl_for;

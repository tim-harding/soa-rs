macro_rules! array {
    ($u:ty, $t:ty $(,$N:tt)?) => {
        impl<'a, T $(,const $N: usize)?> PartialEq<$t> for $u
        where
            T: 'a + Soapy + PartialEq,
        {
            fn eq(&self, other: &$t) -> bool {
                self.as_ref().eq(other)
            }
        }

        impl<'a, T $(,const $N: usize)?> PartialEq<$u> for $t
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
        $crate::eq_impl::array!($t, Vec<T>);
        $crate::eq_impl::array!($t, [T]);
        $crate::eq_impl::array!($t, &[T]);
        $crate::eq_impl::array!($t, &mut [T]);
        $crate::eq_impl::array!($t, [T; N], N);
        $crate::eq_impl::array!($t, &[T; N], N);
        $crate::eq_impl::array!($t, &mut [T; N], N);

        impl<'a, T, R> PartialEq<R> for $t
        where
            T: 'a + Soapy + PartialEq,
            R: AsRef<Slice<T>>,
        {
            fn eq(&self, other: &R) -> bool {
                self.as_ref().eq(other.as_ref())
            }
        }

        impl<'a, T> Eq for $t where T: 'a + Soapy + Eq {}
    };
}

pub(crate) use impl_for;

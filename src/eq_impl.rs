macro_rules! uni {
    ($u:ty, $t:ty $(,$N:tt)?) => {
        impl<'a, T $(,const $N: usize)?> PartialEq<$t> for $u
        where
            T: Soapy + PartialEq,
        {
            fn eq(&self, other: &$t) -> bool {
                let me: &Slice<T> = self.as_ref();
                me.eq(other)
            }
        }

    };
}

macro_rules! bi {
    ($t:ty, $u:ty $(,$N:tt)?) => {
        $crate::eq_impl::uni!($t, $u $(,$N)?);

        impl<'a, T $(,const $N: usize)?> PartialEq<$t> for $u
        where
            T: Soapy + PartialEq,
        {
            fn eq(&self, other: &$t) -> bool {
                other.eq(self)
            }
        }
    };
}

macro_rules! impl_for {
    ($t:ty) => {
        $crate::eq_impl::bi!($t, Vec<T>);
        $crate::eq_impl::bi!($t, [T]);
        $crate::eq_impl::bi!($t, &[T]);
        $crate::eq_impl::bi!($t, &mut [T]);
        $crate::eq_impl::bi!($t, [T; N], N);
        $crate::eq_impl::bi!($t, &[T; N], N);
        $crate::eq_impl::bi!($t, &mut [T; N], N);
        $crate::eq_impl::bi!($t, Slice<T>);
        $crate::eq_impl::uni!($t, SliceRef<'_, T>);
        $crate::eq_impl::uni!($t, SliceMut<'_, T>);
        $crate::eq_impl::uni!($t, Soa<T>);
        impl<'a, T> Eq for $t where T: Soapy + Eq {}
    };
}

pub(crate) use bi;
pub(crate) use impl_for;
pub(crate) use uni;

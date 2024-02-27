macro_rules! uni {
    ($t:ty, $u:ty $(, $($b:tt)+)?) => {
        impl<T $(,$($b)+)?> PartialEq<$u> for $t
        where
            T: Soars ,
            for<'a> T::Ref<'a>: PartialEq,
        {
            fn eq(&self, other: &$u) -> bool {
                let me: &Slice<T> = self.as_ref();
                me.eq(other)
            }
        }
    };
}

macro_rules! bi {
    ($t:ty, $u:ty $(, $($b:tt)+)?) => {
        $crate::eq_impl::uni!($t, $u $(, $($b)+)?);

        impl<T $(,$($b)+)?> PartialEq<$t> for $u
        where
            T: Soars,
            for<'a> T::Ref<'a>: PartialEq,
        {
            fn eq(&self, other: &$t) -> bool {
                let other: &Slice<T> = other.as_ref();
                self.eq(other)
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
        $crate::eq_impl::bi!($t, [T; N], const N: usize);
        $crate::eq_impl::bi!($t, &[T; N], const N: usize);
        $crate::eq_impl::bi!($t, &mut [T; N], const N: usize);
        $crate::eq_impl::bi!($t, Slice<T>);
        $crate::eq_impl::uni!($t, SliceRef<'_, T>);
        $crate::eq_impl::uni!($t, SliceMut<'_, T>);
        $crate::eq_impl::uni!($t, Soa<T>);
        impl<'a, T> Eq for $t where T: Soars, for<'b> T::Ref<'b>: Eq {}
    };
}

pub(crate) use bi;
pub(crate) use impl_for;
pub(crate) use uni;

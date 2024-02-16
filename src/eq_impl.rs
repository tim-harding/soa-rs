macro_rules! uni {
    ($t:ty, $u:ty $(, $($b:tt)+)? $(; $soapy:ident)?) => {
        impl<T, U $(,$($b)+)?> PartialEq<$u> for $t
        where
            T: Soapy + PartialEq<U>,
            $(U: $soapy,)?
        {
            fn eq(&self, other: &$u) -> bool {
                let me: &Slice<T> = self.as_ref();
                me.eq(other)
            }
        }
    };
}

macro_rules! bi {
    ($t:ty, $u:ty $(, $($b:tt)+)? $(; $soapy:ident)?) => {
        $crate::eq_impl::uni!($t, $u $(, $($b)+)? $(; $soapy)?);

        impl<T, U $(,$($b)+)?> PartialEq<$t> for $u
        where
            T: Soapy,
            U: PartialEq<T> $(+ $soapy)?,
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
        $crate::eq_impl::bi!($t, Vec<U>);
        $crate::eq_impl::bi!($t, [U]);
        $crate::eq_impl::bi!($t, &[U]);
        $crate::eq_impl::bi!($t, &mut [U]);
        $crate::eq_impl::bi!($t, [U; N], const N: usize);
        $crate::eq_impl::bi!($t, &[U; N], const N: usize);
        $crate::eq_impl::bi!($t, &mut [U; N], const N: usize);
        $crate::eq_impl::bi!($t, Slice<U>; Soapy);
        $crate::eq_impl::uni!($t, SliceRef<'_, U>; Soapy);
        $crate::eq_impl::uni!($t, SliceMut<'_, U>; Soapy);
        $crate::eq_impl::uni!($t, Soa<U>; Soapy);
        impl<'a, T> Eq for $t where T: Soapy + Eq {}
    };
}

pub(crate) use bi;
pub(crate) use impl_for;
pub(crate) use uni;

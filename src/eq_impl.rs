macro_rules! uni {
    ($t:ty, $u:ty, $($b:tt)*) => {
        impl<$($b)*> PartialEq<$u> for $t {
            fn eq(&self, other: &$u) -> bool {
                let me: &Slice<T> = self.as_ref();
                me.eq(other)
            }
        }
    };
}

macro_rules! bi {
    ($t:ty, $u:ty, $($b:tt)*) => {
        $crate::eq_impl::uni!($t, $u, $($b)*);

        impl<$($b)*> PartialEq<$t> for $u {
            fn eq(&self, other: &$t) -> bool {
                other.eq(self)
            }
        }
    };
}

macro_rules! impl_for {
    ($t:ty) => {
        $crate::eq_impl::bi!($t, Vec<T>, T: Soapy + PartialEq);
        $crate::eq_impl::bi!($t, [T], T: Soapy + PartialEq);
        $crate::eq_impl::bi!($t, &[T], T: Soapy + PartialEq);
        $crate::eq_impl::bi!($t, &mut [T], T: Soapy + PartialEq);
        $crate::eq_impl::bi!($t, [T; N], T: Soapy + PartialEq, const N: usize);
        $crate::eq_impl::bi!($t, &[T; N], T: Soapy + PartialEq, const N: usize);
        $crate::eq_impl::bi!($t, &mut [T; N], T: Soapy + PartialEq, const N: usize);
        $crate::eq_impl::bi!($t, Slice<T>, T: Soapy + PartialEq);
        $crate::eq_impl::uni!($t, SliceRef<'_, T>, T: Soapy + PartialEq);
        $crate::eq_impl::uni!($t, SliceMut<'_, T>, T: Soapy + PartialEq);
        $crate::eq_impl::uni!($t, Soa<T>, T: Soapy + PartialEq);
        impl<'a, T> Eq for $t where T: Soapy + Eq {}
    };
}

pub(crate) use bi;
pub(crate) use impl_for;
pub(crate) use uni;

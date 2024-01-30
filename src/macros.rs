#[allow(unused)]
macro_rules! ref_derive_debug {
    ($t:ident) => {
        impl<'a> ::std::fmt::Debug for $t<'a> {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                self.with_ref(|me| me.fmt(f))
            }
        }
    };
}

#[allow(unused)]
macro_rules! ref_derive_partial_eq {
    ($t:ty, $r:ident) => {
        impl<'a> ::std::cmp::PartialEq<$t> for $r<'a> {
            fn eq(&self, other: &$t) -> bool {
                self.with_ref(|me| me == other)
            }
        }
    };
}

#[allow(unused)]
macro_rules! ref_derive_partial_ord {
    ($t:ty, $r:ident) => {
        impl<'a> ::std::cmp::PartialOrd<$t> for $r<'a> {
            fn partial_cmp(&self, other: &$t) -> ::std::option::Option<::std::cmp::Ordering> {
                self.with_ref(|me| other.partial_cmp(me))
            }
        }
    };
}

#[allow(unused)]
macro_rules! ref_derive_hash {
    ($r:ident) => {
        impl<'a> ::std::hash::Hash for $r<'a> {
            fn hash<H: ::std::hash::Hasher>(&self, state: &mut H) {
                self.with_ref(|me| me.hash(state))
            }
        }
    };
}

#[macro_export]
macro_rules! ref_derive {
    (; $t:ident, $r:ident, $m:ident) => {};

    (Debug; $t:ident, $r:ident, $m:ident) => {
        ref_derive!(Debug,;$t,$r,$m);
    };

    (Debug, $($traits:ident),*; $t:ident, $r:ident, $m:ident) => {
        ref_derive_debug!($r);
        ref_derive_debug!($m);
        ref_derive!($($traits),*; $t, $r, $m);
    };

    (PartialEq; $t:ident, $r:ident, $m:ident) => {
        ref_derive!(PartialEq,;$t,$r,$m);
    };

    (PartialEq, $($traits:ident),*; $t:ident, $r:ident, $m:ident) => {
        ref_derive_partial_eq!($t, $r);
        ref_derive_partial_eq!($t, $m);
        ref_derive!($($traits),*; $t, $r, $m);
    };

    (PartialOrd; $t:ident, $r:ident, $m:ident) => {
        ref_derive!(PartialOrd,;$t,$r,$m);
    };

    (PartialOrd, $($traits:ident),*; $t:ident, $r:ident, $m:ident) => {
        ref_derive_partial_ord!($t, $r);
        ref_derive_partial_ord!($t, $m);
        ref_derive!($($traits),*; $t, $r, $m);
    };

    (Hash; $t:ident, $r:ident, $m:ident) => {
        ref_derive!(Hash,;$t,$r,$m);
    };

    (Hash, $($traits:ident),*; $t:ident, $r:ident, $m:ident) => {
        ref_derive_hash!($r);
        ref_derive_hash!($m);
        ref_derive!($($traits),*; $t, $r, $m);
    };
}

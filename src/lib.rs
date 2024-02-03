#![doc = include_str!("../README.md")]

mod soa;
pub use soa::Soa;

mod index;
pub use index::SoaIndex;

mod into_iter;
pub use into_iter::IntoIter;

mod iter;
pub use iter::Iter;

mod iter_mut;
pub use iter_mut::IterMut;

mod slice;
pub use slice::Slice;

mod slice_mut;
pub use slice_mut::SliceMut;

mod slice_ref;
pub use slice_ref::SliceRef;

pub use soapy_shared::{Soapy, WithRef};

/// Derive macro for the [`Soapy`] trait.
///
/// [`Soapy`]: soapy_shared::Soapy
pub use soapy_derive::Soapy;

/// Creates a [`Soa`] containing the arguments.
///
/// `soa!` allows [`Soa`]s to be defined with the same syntax as array
/// expressions. There are two forms of this macro:
///
/// - Create a [`Soa`] containing a given list of elements:
/// ```
/// # use soapy::{Soapy, soa};
/// # #[derive(Soapy, Debug, PartialEq, Copy, Clone)]
/// # #[extra_impl(Debug)]
/// # struct Foo(u8, u16);
/// let soa = soa![Foo(1, 2), Foo(3, 4)];
/// assert_eq!(soa, [Foo(1, 2), Foo(3, 4)]);
/// ```
///
/// - Create a [`Soa`] from a given element and size:
///
/// ```
/// # use soapy::{Soapy, soa};
/// # #[derive(Soapy, Debug, PartialEq, Copy, Clone)]
/// # #[extra_impl(Debug, PartialEq)]
/// # struct Foo(u8, u16);
/// let soa = soa![Foo(1, 2); 2];
/// assert_eq!(soa, [Foo(1, 2); 2]);
/// ```
#[macro_export]
macro_rules! soa {
    () => {
        $crate::Soa::new()
    };

    ($x:expr $(,$xs:expr)*) => {
        {
            let mut out = $crate::Soa::with($x);
            $(
            out.push($xs);
            )*
            out
        }
    };

    ($elem:expr; 0) => {
        soa![]
    };

    ($elem:expr; 1) => {
        $crate::Soa::with($elem)
    };

    ($elem:expr; $n:expr) => {
        {
            let mut out = $crate::Soa::with($elem);
            for _ in 1..$n {
                let first = out.first();
                // SAFETY: We already added the first element in Soa::with
                let first = unsafe { first.unwrap_unchecked() };
                let clone = $crate::WithRef::with_ref(&first, |first| first.clone());
                out.push(clone);
            }
            out
        }
    };
}

#[cfg(test)]
mod tests {
    use crate::{Soa, Soapy};

    #[derive(Soapy, Debug, Clone, Copy, PartialEq, Eq, Hash)]
    struct El {
        foo: u64,
        bar: u8,
        baz: [u32; 2],
    }

    const A: El = El {
        foo: 0,
        bar: 1,
        baz: [2, 3],
    };

    const B: El = El {
        foo: 4,
        bar: 5,
        baz: [6, 7],
    };

    const C: El = El {
        foo: 8,
        bar: 9,
        baz: [10, 11],
    };

    const D: El = El {
        foo: 12,
        bar: 13,
        baz: [14, 15],
    };

    const E: El = El {
        foo: 16,
        bar: 17,
        baz: [18, 19],
    };

    const ABCDE: [El; 5] = [A, B, C, D, E];

    #[derive(Soapy, Debug, Clone, Copy, PartialEq, Eq, Default)]
    struct Unit;

    #[derive(Soapy, Debug, Clone, Copy, PartialEq, Eq, Default)]
    struct Empty {}

    #[derive(Soapy, Debug, Clone, Copy, PartialEq, Eq, Default)]
    struct EmptyTuple();

    #[derive(Soapy, Debug, Clone, Copy, PartialEq, Eq, Default)]
    struct ZstFields {
        a: Unit,
        b: (),
    }

    #[derive(Soapy, Debug, Clone, Copy, PartialEq, Eq, Default)]
    struct Tuple(u8, u16, u32);

    #[test]
    pub fn tuple() {
        let mut soa = Soa::<Tuple>::new();
        let elements = [Tuple(0, 1, 2), Tuple(3, 4, 5), Tuple(6, 7, 8)];
        for element in elements {
            soa.push(element);
        }
        assert!(elements.into_iter().eq(soa.into_iter()));
    }

    #[test]
    pub fn zst_fields() {
        let mut soa = Soa::<ZstFields>::new();
        for _ in 0..5 {
            soa.push(ZstFields::default());
        }
        for _ in 0..5 {
            assert_eq!(soa.pop(), Some(ZstFields::default()));
        }
        assert_eq!(soa.pop(), None);
    }

    #[test]
    pub fn empty_tuple() {
        let mut soa = Soa::<EmptyTuple>::new();
        for _ in 0..5 {
            soa.push(EmptyTuple());
        }
        for _ in 0..5 {
            assert_eq!(soa.pop(), Some(EmptyTuple()));
        }
        assert_eq!(soa.pop(), None);
    }

    #[test]
    pub fn empty_struct() {
        let mut soa = Soa::<Empty>::new();
        for _ in 0..5 {
            soa.push(Empty {});
        }
        for _ in 0..5 {
            assert_eq!(soa.pop(), Some(Empty {}));
        }
        assert_eq!(soa.pop(), None);
    }

    #[test]
    pub fn unit_struct() {
        let mut soa = Soa::<Unit>::new();
        for _ in 0..5 {
            soa.push(Unit);
        }
        for _ in 0..5 {
            assert_eq!(soa.pop(), Some(Unit));
        }
        assert_eq!(soa.pop(), None);
    }

    #[test]
    pub fn push_and_pop() {
        let mut soa = Soa::<El>::new();
        for element in ABCDE.into_iter() {
            soa.push(element);
        }
        for element in ABCDE.into_iter().rev() {
            assert_eq!(Some(element), soa.pop());
        }
    }

    #[test]
    pub fn insert() {
        test_insert(0, [B, A, A, A]);
        test_insert(1, [A, B, A, A]);
        test_insert(2, [A, A, B, A]);
        test_insert(3, [A, A, A, B]);
    }

    fn test_insert(index: usize, expected: [El; 4]) {
        let mut soa = Soa::<El>::new();
        for element in [A, A, A].into_iter() {
            soa.push(element);
        }
        soa.insert(index, B);
        assert!(soa.into_iter().eq(expected.into_iter()));
    }

    #[test]
    pub fn remove() {
        test_remove(0, A, [B, C, D, E]);
        test_remove(1, B, [A, C, D, E]);
        test_remove(2, C, [A, B, D, E]);
        test_remove(3, D, [A, B, C, E]);
        test_remove(4, E, [A, B, C, D]);
    }

    fn test_remove(index: usize, expected_return: El, expected_contents: [El; 4]) {
        let mut soa = Soa::<El>::new();
        for element in ABCDE.into_iter() {
            soa.push(element);
        }
        assert_eq!(expected_return, soa.remove(index));
        assert!(soa.into_iter().eq(expected_contents.into_iter()));
    }

    #[test]
    pub fn with_capacity() {
        let mut soa = Soa::<El>::with_capacity(5);
        assert_eq!(soa.capacity(), 5);
        assert_eq!(soa.len(), 0);
        for element in ABCDE.into_iter() {
            soa.push(element);
        }
        assert_eq!(soa.capacity(), 5);
        assert_eq!(soa.len(), 5);
    }

    #[test]
    pub fn from_iter() {
        let soa: Soa<_> = ABCDE.into_iter().collect();
        assert!(soa.into_iter().eq(ABCDE.into_iter()));
    }

    #[test]
    pub fn iter() {
        let soa: Soa<_> = ABCDE.into();
        for (borrowed, owned) in soa.iter().zip(ABCDE.into_iter()) {
            assert_eq!(borrowed.foo, &owned.foo);
            assert_eq!(borrowed.bar, &owned.bar);
            assert_eq!(borrowed.baz, &owned.baz);
        }
    }

    #[test]
    pub fn iter_mut() {
        let mut soa: Soa<_> = ABCDE.into();
        for el in soa.iter_mut() {
            *el.foo += 1;
            *el.bar += 2;
            let [a, b] = *el.baz;
            *el.baz = [a + 3, b + 4];
        }
        for (borrowed, owned) in soa.iter().zip(ABCDE.into_iter()) {
            assert_eq!(borrowed.foo, &(owned.foo + 1));
            assert_eq!(borrowed.bar, &(owned.bar + 2));
            assert_eq!(borrowed.baz, &[owned.baz[0] + 3, owned.baz[1] + 4]);
        }
    }

    #[test]
    pub fn from_impls() {
        let expected: Soa<_> = ABCDE.into_iter().collect();
        let array: [El; 5] = ABCDE;
        let array_ref: &[El; 5] = &ABCDE;
        let mut tmp = ABCDE;
        let array_ref_mut: &mut [El; 5] = &mut tmp;
        let slice: &[El] = ABCDE.as_slice();
        let mut tmp = ABCDE;
        let slice_mut: &mut [El] = tmp.as_mut_slice();
        assert_eq!(expected, Soa::from(array));
        assert_eq!(expected, Soa::from(array_ref));
        assert_eq!(expected, Soa::from(array_ref_mut));
        assert_eq!(expected, Soa::from(slice));
        assert_eq!(expected, Soa::from(slice_mut));
    }

    #[test]
    pub fn extend() {
        let mut soa: Soa<_> = [A, B].into();
        soa.extend([C, D]);
        assert!(soa.into_iter().eq([A, B, C, D].into_iter()));
    }

    #[test]
    pub fn clone() {
        let expected: Soa<_> = ABCDE.into();
        let actual = expected.clone();
        assert_eq!(expected, actual);
    }

    #[test]
    pub fn clone_from() {
        let mut dst: Soa<_> = ABCDE.into();
        let src: Soa<_> = [A, A, A].into();
        dst.clone_from(&src);
        assert_eq!(dst, src);
    }

    #[test]
    pub fn partial_ordering_and_equality() {
        #[derive(Soapy, Debug, PartialEq, PartialOrd, Clone, Copy)]
        struct A(f32);

        let cases = [
            (&[][..], &[][..]),
            (&[A(1.), A(2.), A(3.)][..], &[A(1.), A(2.), A(3.)][..]),
            (&[A(1.), A(2.), A(2.)][..], &[A(1.), A(2.), A(3.)][..]),
            (&[A(1.), A(2.), A(4.)][..], &[A(1.), A(2.), A(3.)][..]),
            (
                &[A(1.), A(2.), A(3.)][..],
                &[A(1.), A(2.), A(3.), A(0.)][..],
            ),
            (
                &[A(1.), A(2.), A(3.), A(0.)][..],
                &[A(1.), A(2.), A(3.)][..],
            ),
            (&[A(1.)][..], &[A(f32::NAN)][..]),
        ];

        for case in cases {
            let (l, r) = case;
            let expected_cmp = l.partial_cmp(r);
            let expected_eq = l == r;
            let l: Soa<_> = l.into();
            let r: Soa<_> = r.into();
            assert_eq!(l.partial_cmp(&r), expected_cmp);
            assert_eq!(l == r, expected_eq);
        }
    }

    #[test]
    pub fn ordering() {
        #[derive(Soapy, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
        struct A(u8);

        let cases = [
            (&[][..], &[][..]),
            (&[A(1), A(2), A(3)][..], &[A(1), A(2), A(3)][..]),
            (&[A(1), A(2), A(2)][..], &[A(1), A(2), A(3)][..]),
            (&[A(1), A(2), A(4)][..], &[A(1), A(2), A(3)][..]),
            (&[A(1), A(2), A(3)][..], &[A(1), A(2), A(3), A(0)][..]),
            (&[A(1), A(2), A(3), A(0)][..], &[A(1), A(2), A(3)][..]),
        ];

        for case in cases {
            let (l, r) = case;
            let expected = l.cmp(r);
            let l: Soa<_> = l.into();
            let r: Soa<_> = r.into();
            let actual = l.cmp(&r);
            assert_eq!(actual, expected);
        }
    }

    #[test]
    pub fn debug() {
        let slice = format!("{:?}", ABCDE);
        let soa = format!("{:?}", Soa::from(ABCDE));
        assert_eq!(slice, soa);
    }

    #[test]
    pub fn hashing() {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut expected = DefaultHasher::new();
        ABCDE.hash(&mut expected);

        let mut actual = DefaultHasher::new();
        let soa: Soa<_> = ABCDE.into();
        soa.hash(&mut actual);

        assert_eq!(actual.finish(), expected.finish());
    }

    #[test]
    pub fn get_index() {
        let soa: Soa<_> = ABCDE.into();
        let actual = soa.get(2).unwrap();
        let actual = El {
            foo: *actual.foo,
            bar: *actual.bar,
            baz: *actual.baz,
        };
        assert_eq!(actual, ABCDE[2]);
    }

    #[test]
    pub fn swap() {
        let mut soa: Soa<_> = [A, B, C].into();
        soa.swap(0, 2);
        assert!([C, B, A].into_iter().eq(soa.into_iter()));
    }

    #[test]
    pub fn macro_no_elements() {
        let a: Soa<El> = Soa::new();
        let b = soa![];
        assert_eq!(a, b);
    }

    #[test]
    pub fn field_getters() {
        let mut soa: Soa<_> = ABCDE.into();

        assert_eq!(soa.foo(), &[0, 4, 8, 12, 16]);
        assert_eq!(soa.bar(), &[1, 5, 9, 13, 17]);
        assert_eq!(soa.baz(), &[[2, 3], [6, 7], [10, 11], [14, 15], [18, 19]]);

        for el in soa.foo_mut() {
            *el += 1;
        }

        for el in soa.bar_mut() {
            *el += 1;
        }

        for el in soa.baz_mut() {
            el[0] += 1;
            el[1] += 1;
        }

        assert_eq!(soa.foo(), &[1, 5, 9, 13, 17]);
        assert_eq!(soa.bar(), &[2, 6, 10, 14, 18]);
        assert_eq!(soa.baz(), &[[3, 4], [7, 8], [11, 12], [15, 16], [19, 20]]);
    }
}

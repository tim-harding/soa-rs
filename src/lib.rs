//! Soapy makes it simple to work with structure-of-arrays memory layout. What `Vec<T>`
//! is to array-of-structures (AoS), `Soa<T>` is to structure-of-arrays (SoA).
//!
//! # Example
//!
//! ```
//! # use soapy::{Soa, Soapy};
//! #[derive(Soapy, Debug, Clone, Copy, PartialEq)]
//! struct Example {
//!     foo: u8,
//!     bar: u16,
//! }
//!
//! let elements = [Example { foo: 1, bar: 2 }, Example { foo: 3, bar: 4 }];
//! let mut soa: Soa<_> = elements.into_iter().collect();
//!
//! // The index operator is not possible, but we can use nth:
//! *soa.nth_mut(0).foo += 10;
//!
//! // We can get the fields as slices as well:
//! let slices = soa.slices();
//! assert_eq!(slices.foo, &[11, 3][..]);
//! assert_eq!(slices.bar, &[2, 4][..]);
//!
//! for (actual, expected) in soa.iter().zip(elements.iter()) {
//!     assert_eq!(&expected.bar, actual.bar);
//! }
//! ```
//!
//! # What is SoA?
//!
//! The following types illustrate the difference between AoS and Soa:
//! ```text
//! [(u8,   u64)] // AoS
//! ([u8], [u64]) // Soa
//! ```
//!
//! Whereas AoS stores all the fields of a type in each element of the array,
//! SoA splits each field into its own array. This has several benefits:
//!
//! - There is no padding required between instances of the same type. In the
//! above example, each AoS element requires 128 bits to satisfy memory
//! alignment requirements, whereas each SoA element only takes 72. This can
//! mean better cache locality and lower memory usage.
//! - SoA can be more amenable to vectorization. With SoA, multiple values can
//! be direcly loaded into SIMD registers in bulk, as opposed to shuffling
//! struct fields into and out of different SIMD registers.
//!
//! SoA is a popular technique in data-oriented design. Andrew Kelley gives a
//! wonderful [talk](https://vimeo.com/649009599) describing how SoA and other
//! data-oriented design patterns earned him a 39% reduction in wall clock time
//! in the Zig compiler.
//!
//! Note that SoA does not offer performance wins in all cases. SoA is most
//! appropriate when either
//! - Sequential access is the common access pattern
//! - You are frequently accessing or modifying only a subset of the fields
//!
//! As always, it is best to profile both for your use case.
//!
//! # Derive
//!
//! Soapy provides the [`Soapy`] derive macro to generate SoA compatibility for
//! structs automatically. When deriving Soapy, several new structs are
//! created. Because of the way SoA data is stored, iterators and getters often
//! yield these types instead of the original struct. If each field of some
//! struct `Example` has type `F`, our new structs have the same fields but
//! different types:
//!
//! | Struct             | Field type | Use                                   |
//! |--------------------|------------|---------------------------------------|
//! | `ExampleRawSoa`    | `*mut F`   | Low-level, unsafe interface for `Soa` |
//! | `ExampleRef`       | `&F`       | `.iter()`, `nth()`, `.get()`          |
//! | `ExampleRefMut`    | `&mut F`   | `.iter_mut()`, `nth_mut`, `get_mut()` |
//! | `ExampleSlices`    | `&[F]`     | `.slices()`, `.get()`                 |
//! | `ExampleSlicesMut` | `&mut [F]` | `.slices_mut()`, `.get_mut()`         |
//!
//! These types are included as associated types on the [`Soapy`] trait as well.
//! Generally, you won't need to think about these them as [`Soa`] picks them up
//! automatically. However, since they inherit the visibility of the derived
//! struct, you should consider whether to include them in the `pub` items of
//! your module.

mod index;
mod into_iter;
mod iter;
mod iter_mut;
mod slice;
mod soa;

pub use into_iter::IntoIter;
pub use iter::Iter;
pub use iter_mut::IterMut;
pub use soa::Soa;
pub use soapy_shared::{Soapy, WithRef};

/// Creates a [`Soa`] containing the arguments.
///
/// `soa!` allows [`Soa`]s to be defined with the same syntax as array
/// expressions. There are two forms of this macro:
///
/// - Create a [`Soa`] containing a given list of elements:
/// ```
/// # use soapy::{Soapy, soa};
/// # #[derive(Soapy)]
/// # struct Foo(u8, u16);
/// let soa = soa![Foo(1, 2), Foo(3, 4)];
/// assert_eq!(soa.slices().0, &[1, 3]);
/// assert_eq!(soa.slices().1, &[2, 4]);
/// ```
///
/// - Create a [`Soa`] from a given element and size:
///
/// ```
/// # use soapy::{Soapy, soa};
/// # #[derive(Soapy, Copy, Clone)]
/// # struct Foo(u8, u16);
/// let soa = soa![Foo(1, 2); 2];
/// assert_eq!(soa.slices().0, &[1, 1]);
/// assert_eq!(soa.slices().1, &[2, 2]);
/// ```
#[macro_export]
macro_rules! soa {
    () => {
        $crate::Soa::new()
    };

    ($($e:expr),*) => {
        // TODO: Downside, does this array have to be constructed on the stack?
        [$($e),*].into_iter().collect::<$crate::Soa<_>>()
    };

    ($elem:expr; $n:expr) => {
        // TODO: Support Clone types in addition to Copy
        [$elem; $n].into_iter().collect::<$crate::Soa<_>>()
    }
}

/// Derive macro for the [`Soapy`] trait.
///
/// [`Soapy`]: soapy_shared::Soapy
pub use soapy_derive::Soapy;

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
    pub fn get_range() {
        let soa: Soa<_> = ABCDE.into();
        let slices = soa.get(1..3).unwrap();
        for (&expected, (foo, (bar, baz))) in ABCDE[1..3].iter().zip(
            slices
                .foo
                .iter()
                .zip(slices.bar.iter().zip(slices.baz.iter())),
        ) {
            let actual = El {
                foo: *foo,
                bar: *bar,
                baz: *baz,
            };
            assert_eq!(actual, expected);
        }
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

#![cfg(test)]

use soapy::{soa, AsSoaRef, Soa, SoaArray, Soapy};
use std::fmt::Debug;

#[derive(Soapy, Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[soa_derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
struct ExtraImplTester {
    things: u8,
    stuff: u8,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct SingleDrop(u8);

impl SingleDrop {
    pub const DEFAULT: Self = Self(0);
}

impl Drop for SingleDrop {
    fn drop(&mut self) {
        assert_eq!(self.0, 0);
        self.0 += 1;
    }
}

#[derive(Soapy, Debug, Clone, PartialEq, Eq, Hash)]
#[soa_derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct El {
    foo: u64,
    bar: u8,
    baz: SingleDrop,
}

const A: El = El {
    foo: 0,
    bar: 1,
    baz: SingleDrop::DEFAULT,
};

const B: El = El {
    foo: 4,
    bar: 5,
    baz: SingleDrop::DEFAULT,
};

const C: El = El {
    foo: 8,
    bar: 9,
    baz: SingleDrop::DEFAULT,
};

const D: El = El {
    foo: 12,
    bar: 13,
    baz: SingleDrop::DEFAULT,
};

const E: El = El {
    foo: 16,
    bar: 17,
    baz: SingleDrop::DEFAULT,
};

const ABCDE: [El; 5] = [A, B, C, D, E];

#[derive(Soapy, Debug, Clone, Copy, PartialEq, Eq, Default, PartialOrd, Ord)]
#[soa_derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Unit;

#[derive(Soapy, Debug, Clone, Copy, PartialEq, Eq, Default)]
#[soa_derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Empty {}

#[derive(Soapy, Debug, Clone, Copy, PartialEq, Eq, Default)]
#[soa_derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
struct EmptyTuple();

#[derive(Soapy, Debug, Clone, Copy, PartialEq, Eq, Default)]
#[soa_derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
struct ZstFields {
    a: Unit,
    b: (),
}

#[derive(Soapy, Debug, Clone, Copy, PartialEq, Eq, Default)]
#[soa_derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
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
    }
    for (borrowed, owned) in soa.iter().zip(ABCDE.into_iter()) {
        assert_eq!(borrowed.foo, &(owned.foo + 1));
        assert_eq!(borrowed.bar, &(owned.bar + 2));
    }
}

#[test]
pub fn from_impls() {
    let expected: Soa<_> = ABCDE.into_iter().collect();
    let array: [El; 5] = ABCDE;
    let array_ref: &[El; 5] = &ABCDE;
    let mut tmp = ABCDE;
    let array_ref_mut: &mut [El; 5] = &mut tmp;
    assert_eq!(expected, Soa::from(array));
    assert_eq!(expected, Soa::from(array_ref));
    assert_eq!(expected, Soa::from(array_ref_mut));
}

#[test]
pub fn extend() {
    let mut soa: Soa<_> = [A, B].into();
    soa.extend([C, D]);
    assert!(soa.into_iter().eq([A, B, C, D].into_iter()));
}

#[test]
pub fn clone() {
    let expected: Soa<_> = [Tuple(1, 2, 3), Tuple(4, 5, 6), Tuple(7, 8, 9)].into();
    let actual = expected.clone();
    assert_eq!(expected, actual);
}

#[test]
pub fn clone_from() {
    let mut dst: Soa<_> = std::iter::repeat(Tuple(100, 100, 100)).take(7).collect();
    let src: Soa<_> = [Tuple(1, 2, 3), Tuple(4, 5, 6), Tuple(7, 8, 9)].into();
    dst.clone_from(&src);
    assert_eq!(dst, src);
}

#[test]
pub fn partial_ordering_and_equality() {
    #[derive(Soapy, Debug, PartialEq, PartialOrd, Clone, Copy)]
    #[soa_derive(Debug, PartialEq, PartialOrd)]
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
    #[soa_derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
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
        baz: Default::default(),
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

    for el in soa.foo_mut() {
        *el += 1;
    }

    for el in soa.bar_mut() {
        *el += 1;
    }

    assert_eq!(soa.foo(), &[1, 5, 9, 13, 17]);
    assert_eq!(soa.bar(), &[2, 6, 10, 14, 18]);
}

#[derive(Debug, Clone, Copy, PartialEq, Soapy)]
#[soa_derive(Debug, PartialEq, PartialOrd)]
struct Alignment {
    #[align(64)]
    a: f32,
    #[align(64)]
    b: f32,
    #[align(64)]
    c: f32,
    #[align(64)]
    d: f32,
}

#[test]
pub fn align_attribute() {
    let aligns = [
        Alignment {
            a: 0.0,
            b: 1.0,
            c: 2.0,
            d: 3.0,
        },
        Alignment {
            a: 4.0,
            b: 5.0,
            c: 6.0,
            d: 7.0,
        },
        Alignment {
            a: 8.0,
            b: 9.0,
            c: 10.0,
            d: 11.0,
        },
    ];

    let soa: Soa<_> = aligns.into_iter().collect();
    assert_eq!(soa, aligns);
}

#[test]
pub fn iterator_slice_methods() {
    let mut soa = Soa::from(ABCDE);
    let expected = &ABCDE[1..];

    {
        let mut iter = soa.iter();
        iter.next();
        assert_eq!(iter.as_slice(), expected);
    }

    {
        let mut iter = soa.iter_mut();
        iter.next();
        assert_eq!(iter.as_slice(), expected);
        assert_eq!(iter.as_mut_slice(), expected);
        assert_eq!(iter.into_slice(), expected);
    }

    {
        let mut iter = soa.into_iter();
        iter.next();
        assert_eq!(iter.as_slice(), expected);
        assert_eq!(iter.as_mut_slice(), expected);
    }
}

#[test]
fn iterator_size_hint() {
    let soa = Soa::from(ABCDE);
    assert_eq!(soa.iter().size_hint(), (5, Some(5)));
}

#[test]
fn iterator_count() {
    let soa = Soa::from(ABCDE);
    assert_eq!(soa.iter().count(), 5);
}

macro_rules! assert_option_eq {
    ($u:expr, $v:expr) => {
        #[allow(clippy::iter_nth_zero)]
        match ($u, $v) {
            (Some(u), Some(v)) => assert_eq!(u.as_soa_ref(), v.as_soa_ref()),
            (None, None) => {}
            (u, v) => panic!("not equal: {u:?}, {v:?}"),
        }
    };
}

#[test]
fn iterator_last() {
    let soa: Soa<_> = ABCDE.into();
    assert_eq!(soa.into_iter().last(), Some(E));
}

#[test]
fn iterator_nth() {
    let soa: Soa<_> = ABCDE.into_iter().cycle().take(20).collect();
    let vec: Vec<_> = ABCDE.into_iter().cycle().take(20).collect();
    let mut iter_soa = soa.iter();
    let mut iter_vec = vec.iter();
    for _ in 0..22 {
        assert_option_eq!(iter_vec.nth(0), iter_soa.nth(0));
    }
    for i in 0..10 {
        assert_option_eq!(iter_vec.nth(i), iter_soa.nth(i));
    }
}

#[test]
fn iterator_nth_back() {
    let soa: Soa<_> = ABCDE.into_iter().cycle().take(20).collect();
    let vec: Vec<_> = ABCDE.into_iter().cycle().take(20).collect();
    let mut iter_soa = soa.iter();
    let mut iter_vec = vec.iter();
    for _ in 0..22 {
        assert_option_eq!(iter_vec.nth_back(0), iter_soa.nth_back(0));
    }
    for i in 0..10 {
        assert_option_eq!(iter_vec.nth_back(i), iter_soa.nth_back(i));
    }
}

#[test]
fn iterator_next_back() {
    let soa: Soa<_> = ABCDE.into();
    let vec: Vec<_> = ABCDE.into();
    let mut soa_iter = soa.iter();
    let mut vec_iter = vec.iter();
    for _ in 0..6 {
        assert_option_eq!(vec_iter.next_back(), soa_iter.next_back());
    }
}

#[test]
fn iterator_fold() {
    fn fold(acc: u64, el: El) -> u64 {
        acc + el.foo + el.bar as u64
    }

    let soa: Soa<_> = ABCDE.into();
    let actual = soa.into_iter().fold(0, fold);
    let expected = ABCDE.into_iter().fold(0, fold);
    assert_eq!(actual, expected);
}

#[test]
fn chunks_exact() {
    let soa: Soa<_> = ABCDE.into_iter().cycle().take(19).collect();
    let vec: Vec<_> = ABCDE.into_iter().cycle().take(19).collect();
    let mut soa_iter = soa.chunks_exact(4);
    let mut vec_iter = vec.chunks_exact(4);
    for _ in 0..(19 / 4) {
        match (soa_iter.next(), vec_iter.next()) {
            (Some(u), Some(v)) => assert_eq!(u, v),
            (None, None) => {}
            (u, v) => panic!("not equal: {u:?}, {v:?}"),
        }
    }
    assert_eq!(soa_iter.remainder(), vec_iter.remainder());
}

const ARRAY: ElArray<5> = ElArray::from_array(ABCDE);

#[test]
fn array_slice_eq() {
    let array = ARRAY;
    let slice = array.as_slice();
    assert_eq!(slice, ABCDE);
}

#[test]
fn array_slice_mut() {
    let mut array = ARRAY;
    let mut slice = array.as_mut_slice();
    for item in slice.iter_mut() {
        *item.foo += 10;
        *item.bar += 10;
    }
    let expected = ABCDE.map(|el| El {
        foo: el.foo + 10,
        bar: el.bar + 10,
        baz: el.baz,
    });
    assert_eq!(slice, expected);
}

#[test]
fn slices() {
    let soa = Soa::from(ABCDE);
    let slices = soa.slices();
    assert_eq!(slices.foo, soa.foo());
    assert_eq!(slices.bar, soa.bar());
    assert_eq!(slices.baz, soa.baz());
}

#[test]
fn slices_mut() {
    let mut soa = Soa::from(ABCDE);
    let slices = soa.slices_mut();
    for foo in slices.foo {
        *foo += 10;
    }
    for bar in slices.bar {
        *bar += 10;
    }

    let expected = ABCDE.map(|el| el.foo + 10);
    assert_eq!(soa.foo(), expected);

    let expected = ABCDE.map(|el| el.bar + 10);
    assert_eq!(soa.bar(), expected);
}

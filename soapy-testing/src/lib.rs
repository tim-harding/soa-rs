#![cfg(test)]

use soapy::{soa, Soa, Soapy};

#[derive(Soapy, Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[extra_impl(Debug, PartialEq, Eq, PartialOrd, Ord)]
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
    for mut el in soa.iter_mut() {
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

#[test]
pub fn arrays() {
    const A: ElSoaArray<3> = ElSoaArray {
        foo: [1, 2, 3],
        bar: [4, 5, 6],
        baz: [SingleDrop::DEFAULT; 3],
    };
    const B: ElSoaArray<3> = ElSoaArray::from_array([
        El {
            foo: 1,
            bar: 4,
            baz: SingleDrop::DEFAULT,
        },
        El {
            foo: 2,
            bar: 5,
            baz: SingleDrop::DEFAULT,
        },
        El {
            foo: 3,
            bar: 6,
            baz: SingleDrop::DEFAULT,
        },
    ]);
    assert_eq!(A.as_slice(), B.as_slice());

    let mut a = A;
    for foo in a.as_mut_slice().foo_mut() {
        *foo += 10;
    }
    assert_eq!(
        a.as_slice(),
        ElSoaArray {
            foo: [11, 12, 13],
            bar: [4, 5, 6],
            baz: [SingleDrop::DEFAULT; 3],
        }
        .as_slice()
    );
}

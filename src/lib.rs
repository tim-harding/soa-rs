mod drain;
mod into_iter;
mod iter;
mod iter_mut;
mod raw_value_iter;
mod soa;

pub use into_iter::IntoIter;
pub use iter::Iter;
pub use iter_mut::IterMut;
pub use soa::Soa;
pub use soapy_derive::Soapy;

#[cfg(test)]
mod tests {
    use crate::{Soa, Soapy};

    #[derive(Soapy, Debug, Clone, Copy, PartialEq, Eq)]
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
        let mut soa = Soa::new();
        let elements = [Tuple(0, 1, 2), Tuple(3, 4, 5), Tuple(6, 7, 8)];
        for element in elements {
            soa.push(element);
        }
        assert!(elements.into_iter().eq(soa.into_iter()));
    }

    #[test]
    pub fn zst_fields() {
        let mut soa = Soa::new();
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
        let mut soa = Soa::new();
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
        let mut soa = Soa::new();
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
        let mut soa = Soa::new();
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
        let mut soa = Soa::new();
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
        let mut soa = Soa::new();
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
        let mut soa = Soa::new();
        for element in ABCDE.into_iter() {
            soa.push(element);
        }
        assert_eq!(expected_return, soa.remove(index));
        assert!(soa.into_iter().eq(expected_contents.into_iter()));
    }

    #[test]
    pub fn with_capacity() {
        let mut soa = Soa::with_capacity(5);
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
            assert_eq!(borrowed.baz, &owned.baz);
        }
    }

    #[test]
    pub fn from_impls() {
        let expected: Soa<_> = ABCDE.into_iter().collect();
        let vec: Vec<_> = ABCDE.into();
        let array: [El; 5] = ABCDE;
        let array_ref: &[El; 5] = &ABCDE;
        let mut tmp = ABCDE;
        let array_ref_mut: &mut [El; 5] = &mut tmp;
        let slice: &[El] = ABCDE.as_slice();
        let mut tmp = ABCDE;
        let slice_mut: &mut [El] = tmp.as_mut_slice();
        assert_eq!(expected, vec.into());
        assert_eq!(expected, array.into());
        assert_eq!(expected, array_ref.into());
        assert_eq!(expected, array_ref_mut.into());
        assert_eq!(expected, slice.into());
        assert_eq!(expected, slice_mut.into());
    }
}

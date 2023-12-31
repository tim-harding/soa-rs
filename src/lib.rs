mod soa;

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

    const ELEMENTS: [El; 5] = [A, B, C, D, E];
    const ALL_A: [El; 3] = [A, A, A];

    #[test]
    pub fn push_and_pop() {
        let mut soa = Soa::new();
        for element in ELEMENTS.into_iter() {
            soa.push(element);
        }
        for element in ELEMENTS.into_iter().rev() {
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
        for element in ALL_A.into_iter() {
            soa.push(element);
        }
        soa.insert(index, B);
        for element in expected.into_iter().rev() {
            assert_eq!(Some(element), soa.pop());
        }
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
        for element in ELEMENTS.into_iter() {
            soa.push(element);
        }
        assert_eq!(expected_return, soa.remove(index));
        for element in expected_contents.into_iter().rev() {
            assert_eq!(Some(element), soa.pop());
        }
    }
}

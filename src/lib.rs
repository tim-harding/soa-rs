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
}

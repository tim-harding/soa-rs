use derive_macro::Soa;
use std::{marker::PhantomData, ptr::NonNull};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct Unique<T> {
    ptr: NonNull<T>,
    _owns_t: PhantomData<T>,
}

unsafe impl<T: Send> Send for Unique<T> {}
unsafe impl<T: Sync> Sync for Unique<T> {}

impl<T> Unique<T> {
    pub const fn dangling() -> Self {
        Self {
            ptr: NonNull::dangling(),
            _owns_t: PhantomData,
        }
    }

    /// SAFETY: Ensure that T is non-null
    pub const unsafe fn new(ptr: *mut u8) -> Self {
        Self {
            ptr: unsafe { NonNull::new_unchecked(ptr as *mut T) },
            _owns_t: PhantomData,
        }
    }
}

#[derive(Soa, Debug, Clone, Copy, PartialEq, Eq)]
pub struct El {
    foo: u64,
    bar: u8,
    baz: [u32; 2],
}

#[cfg(test)]
mod tests {
    use super::*;

    const A: El = El {
        foo: 20,
        bar: 10,
        baz: [6, 4],
    };

    const B: El = El {
        foo: 10,
        bar: 5,
        baz: [3, 2],
    };

    const ZERO: El = El {
        foo: 0,
        bar: 0,
        baz: [0, 0],
    };

    fn soa() -> Soa {
        let mut soa = Soa::new();
        soa.push(A);
        soa.push(B);
        soa
    }

    #[test]
    fn push_and_pop() {
        let mut soa = soa();
        assert_eq!(soa.pop(), Some(B));
        assert_eq!(soa.pop(), Some(A));
        assert_eq!(soa.pop(), None);
    }

    #[test]
    fn iterators() {
        let soa = soa();
        assert_eq!(soa.foo(), &[20, 10]);
        assert_eq!(soa.bar(), &[10, 5]);
        assert_eq!(soa.baz(), &[[6, 4], [3, 2]]);
    }

    #[test]
    fn insert_and_remove() {
        let mut soa = soa();
        soa.insert(1, ZERO);
        assert_eq!(soa.foo(), &[20, 0, 10]);
        assert_eq!(soa.bar(), &[10, 0, 5]);
        assert_eq!(soa.baz(), &[[6, 4], [0, 0], [3, 2]]);
        assert_eq!(soa.remove(1), ZERO);
        assert_eq!(soa.foo(), &[20, 10]);
        assert_eq!(soa.bar(), &[10, 5]);
        assert_eq!(soa.baz(), &[[6, 4], [3, 2]]);
    }

    #[test]
    fn into_iter() {
        {
            let mut soa = soa().into_iter();
            assert_eq!(soa.next(), Some(A));
            assert_eq!(soa.next(), Some(B));
            assert_eq!(soa.next(), None);
        }
    }
}

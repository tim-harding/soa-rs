use std::{marker::PhantomData, ptr::NonNull};

/// A wrapper around a raw pointer with several properties:
/// - [Covariant](https://doc.rust-lang.org/nomicon/subtyping.html#variance) over T
/// - Owns a T
/// - [`Send`]/[`Sync`] if T is [`Send`]/[`Sync`]
/// - Never null
/// This can be replaced by [`core::ptr::Unique`] when it is stabilized.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Unique<T> {
    ptr: NonNull<T>,
    _owns_t: PhantomData<T>,
}

unsafe impl<T: Send> Send for Unique<T> {}
unsafe impl<T: Sync> Sync for Unique<T> {}

impl<T> Unique<T> {
    /// Creates a new [`Unique`] that is invalid but well-aligned. Intended for
    /// uninitialized memory.
    pub const fn dangling() -> Self {
        Self {
            ptr: NonNull::dangling(),
            _owns_t: PhantomData,
        }
    }

    /// Aquires the underlying pointer
    pub fn as_ptr(self) -> *mut T {
        self.ptr.as_ptr()
    }

    /// Creates a new [`Unique`]
    /// SAFETY: Ensure that T is non-null
    pub const unsafe fn new(ptr: *mut u8) -> Self {
        Self {
            ptr: unsafe { NonNull::new_unchecked(ptr as *mut T) },
            _owns_t: PhantomData,
        }
    }
}

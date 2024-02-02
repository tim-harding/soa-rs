use crate::{RawSoa, Slice, WithRef};

/// Provides SOA data structure compatibility.
///
/// This trait should be derived using the `soapy-derive` crate.
pub trait Soapy: Sized {
    /// Implements internal, unsafe, low-level routines used by `Soa`
    type RawSoa: RawSoa<Item = Self>;

    type Slice: Slice;

    /// For each field with type `F` in `T`, `Slices` has a field with type
    /// `&[F]`
    type Slices<'a>
    where
        Self: 'a;

    /// For each field with type `F` in `T`, `SlicesMut` has a field with type
    /// `&mut [F]`
    type SlicesMut<'a>
    where
        Self: 'a;

    /// For each field with type `F` in `T`, `Ref` has a field with type
    /// `&F`
    type Ref<'a>: WithRef<Item = Self>
    where
        Self: 'a;

    /// For each field with type `F` in `T`, `RefMut` has a field with type
    /// `&mut F`
    type RefMut<'a>: WithRef<Item = Self>
    where
        Self: 'a;
}

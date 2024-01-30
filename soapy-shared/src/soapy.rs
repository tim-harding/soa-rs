use crate::RawSoa;

/// Provides SOA data structure compatibility.
///
/// This trait should be derived using the `soapy-derive` crate.
pub trait Soapy: Sized {
    type RawSoa: RawSoa<Self>;
}

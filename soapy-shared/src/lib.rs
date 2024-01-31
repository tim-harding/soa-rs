//! This crate provides traits that are implemented by the `soapy-derive` crate.
//!
//! On its own, this crate is not useful. Soapy is organized this way, rather
//! than by putting these traits in the `soapy` crate directly, because the
//! derive macro needs to specify absolute paths to the traits it implements.
//! Unfortunately, if a trait is part of the current crate, the path has to be
//! prefixed with `crate::` rather than `::my_crate_name::`, so it would not be
//! possible to implement these traits for any types in `soapy` for testing.
//! That is why these are split out into a subcrate. See the [tracking
//! issue](https://github.com/rust-lang/rust/issues/54363) for more details.

mod raw_soa;
pub use raw_soa::RawSoa;

mod with_ref;
pub use with_ref::WithRef;

mod soapy;
pub use soapy::Soapy;

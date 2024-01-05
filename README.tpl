![docs.rs](https://img.shields.io/docsrs/soapy?link=https%3A%2F%2Fdocs.rs%2Fsoapy%2Flatest%2Fsoapy%2F)
![Crates.io Version](https://img.shields.io/crates/v/soapy?link=https%3A%2F%2Fcrates.io%2Fcrates%2Fsoapy)
![GitHub License](https://img.shields.io/github/license/tim-harding/soapy?link=https%3A%2F%2Fgithub.com%2Ftim-harding%2Fsoapy%2Fblob%2Fmain%2FLICENSE)

# Soapy

{{readme}}

## Comparison

### [`soa_derive`](https://docs.rs/soa_derive/latest/soa_derive/)

`soa_derive` makes each field its own `Vec`. Because of this, each field's
length, capacity, and allocation are managed separately. In contrast, Soapy
manages a single allocation for each `Soa`. This uses less space and allows
the collection to grow and shrink more efficiently. `soa_derive` also
generates a new collection type for every struct, whereas Soapy generates a
minimal, low-level interface and uses the generic `Soa` type for the
majority of the implementation. This provides more type system flexibility,
less code generation, and more accessible documentation.

### [`soa-vec`](https://docs.rs/soa-vec/latest/soa_vec/)

Whereas `soa-vec` only compiles on nightly, Soapy also compiles on stable.
Rather than using derive macros, `soa-vec` instead uses macros to generate
eight static copies of their SoA type with fixed tuple sizes.

## Progress

### Soa

- [ ] `depup` / `dedup_by` / `dedup_by_key`
- [ ] `drain`
- [ ] `extend_from_slice` / `extend_from_within`
- [ ] `extract_if`
- [ ] `leak`
- [ ] `retain`
- [ ] `try_reserve` / `try_reserve_exact`
- [ ] `dedup_by` / `dedup_by_key`
- [ ] `resize` / `resize_with`
- [ ] `splice`
- [ ] `split_off`

### SoaSlice
- [ ] `select_nth_unstable` / `select_nth_unstable_by` / `select_nth_unstable_by_key`
- [ ] `sort` / `sort_by` / `sort_by_key` / `sort_by_cached_key`
- [ ] `sort_unstable` / `sort_unstable_by` / `sort_unstable_by_key` / `sort_unstable_by_cached_key`
- [ ] `binary_search` / `binary_search_by` / `binary_search_by_key`
- [ ] `is_sorted` / `is_sorted_by` / `is_sorted_by_key`
- [ ] `chunks` / `rchunks`
- [ ] `chunks_exact` / `rchunks_exact`
- [ ] `first` / `last`
- [ ] `rotate_left` / `rotate_right`
- [ ] `split` / `rsplit` / `splitn`
- [ ] `split_at` / `split_first` / `split_last`
- [ ] `swap`
- [ ] `swap_with_slice`
- [ ] `group_by`
- [ ] `contains`
- [ ] `copy_within`
- [ ] `fill` / `fill_with`
- [ ] `repeat`
- [ ] `reverse`

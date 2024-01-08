![docs.rs](https://img.shields.io/docsrs/soapy?link=https%3A%2F%2Fdocs.rs%2Fsoapy%2Flatest%2Fsoapy%2F)
![Crates.io Version](https://img.shields.io/crates/v/soapy?link=https%3A%2F%2Fcrates.io%2Fcrates%2Fsoapy)
![GitHub License](https://img.shields.io/github/license/tim-harding/soapy?link=https%3A%2F%2Fgithub.com%2Ftim-harding%2Fsoapy%2Fblob%2Fmain%2FLICENSE)

# Soapy

Soapy makes it simple to work with structure-of-arrays memory layout. What `Vec<T>`
is to array-of-structures (AoS), `Soa<T>` is to structure-of-arrays (SoA).

## Example

```rust
#[derive(Soapy, Debug, Clone, Copy, PartialEq)]
struct Example {
    foo: u8,
    bar: u16,
}

let elements = [Example { foo: 1, bar: 2 }, Example { foo: 3, bar: 4 }];
let mut soa: Soa<_> = elements.into_iter().collect();

// The index operator is not possible, but we can use nth:
*soa.nth_mut(0).foo += 10;

// We can get the fields as slices as well:
let slices = soa.slices();
assert_eq!(slices.foo, &[11, 3][..]);
assert_eq!(slices.bar, &[2, 4][..]);

for (actual, expected) in soa.iter().zip(elements.iter()) {
    assert_eq!(&expected.bar, actual.bar);
}
```

## What is SoA?

The following types illustrate the difference between AoS and Soa:
```
[(u8,   u64)] // AoS
([u8], [u64]) // Soa
```

Whereas AoS stores all the fields of a type in each element of the array,
SoA splits each field into its own array. This has several benefits:

- There is no padding required between instances of the same type. In the
above example, each AoS element requires 128 bits to satisfy memory
alignment requirements, whereas each SoA element only takes 72. This can
mean better cache locality and lower memory usage.
- SoA can be more amenable to vectorization. With SoA, multiple values can
be direcly loaded into SIMD registers in bulk, as opposed to shuffling
struct fields into and out of different SIMD registers.

SoA is a popular technique in data-oriented design. Andrew Kelley gives a
wonderful [talk](https://vimeo.com/649009599) describing how SoA and other
data-oriented design patterns earned him a 39% reduction in wall clock time
in the Zig compiler.

Note that SoA does not offer performance wins in all cases. SoA is most
appropriate when either
- Sequential access is the common access pattern
- You are frequently accessing or modifying only a subset of the fields

As always, it is best to profile both for your use case.

## Derive

Soapy provides the [`Soapy`] derive macro to generate SoA compatibility for
structs automatically. When deriving Soapy, several new structs are
created. Because of the way SoA data is stored, iterators and getters often
yield these types instead of the original struct. If each field of some
struct `Example` has type `F`, our new structs have the same fields but
different types:

| Struct             | Field type | Use                                   |
|--------------------|------------|---------------------------------------|
| `ExampleRawSoa`    | `*mut F`   | Low-level, unsafe interface for `Soa` |
| `ExampleRef`       | `&F`       | `.iter()`, `nth()`, `.get()`          |
| `ExampleRefMut`    | `&mut F`   | `.iter_mut()`, `nth_mut`, `get_mut()` |
| `ExampleSlices`    | `&[F]`     | `.slices()`, `.get()`                 |
| `ExampleSlicesMut` | `&mut [F]` | `.slices_mut()`, `.get_mut()`         |

These types are included as associated types on the [`Soapy`] trait as well.
Generally, you won't need to think about these them as [`Soa`] picks them up
automatically. However, since they inherit the visibility of the derived
struct, you should consider whether to include them in the `pub` items of
your module.

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

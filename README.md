[![docs.rs](https://img.shields.io/docsrs/soapy?link=https%3A%2F%2Fdocs.rs%2Fsoapy%2Flatest%2Fsoapy%2F)](https://docs.rs/soapy/latest/soapy/)
[![Crates.io Version](https://img.shields.io/crates/v/soapy?link=https%3A%2F%2Fcrates.io%2Fcrates%2Fsoapy)](https://crates.io/crates/soapy)
[![GitHub License](https://img.shields.io/github/license/tim-harding/soapy?link=https%3A%2F%2Fgithub.com%2Ftim-harding%2Fsoapy%2Fblob%2Fmain%2FLICENSE)](https://choosealicense.com/licenses/mit/)

# Soapy

Soapy makes it simple to work with the structure-of-arrays memory layout. What
`Vec<T>` is to array-of-structures (AoS), `Soa<T>` is to structure-of-arrays
(SoA).

## Example

```rust
use soapy::{Soapy, soa};

// Derive Soapy for your type
#[derive(Soapy, PartialEq, Debug)]
struct Baz {
    foo: u16,
    bar: u8,
}

// Create the SoA
let mut soa = soa![
    Baz { foo: 1, bar: 2 }, 
    Baz { foo: 3, bar: 4 },
];

// Each field has a slice
assert_eq!(soa.foo(), [1, 3]);
assert_eq!(soa.bar(), [2, 4]);

// Normal Vec stuff works
soa.insert(0, Baz { foo: 5, bar: 6 });
assert_eq!(soa.pop(), Some(Baz { foo: 3, bar: 4 }));
assert_eq!(soa.foo(), [5, 1]);
for mut el in &mut soa {
    *el.foo += 10;
}
assert_eq!(soa.foo(), [15, 11]);

// Tuple structs are okay too
#[derive(Soapy, PartialEq, Debug)]
struct Tuple(u16, u8);
let tuple = soa![Tuple(1, 2), Tuple(3, 4), Tuple(5, 6), Tuple(7, 8)];
assert_eq!(tuple.f0(), [1, 3, 5, 7]);

// SoA can be sliced and indexed like other slices
assert_eq!(tuple.idx(1..3), [Tuple(3, 4), Tuple(5, 6)]);
assert_eq!(tuple.idx(3), Tuple(7, 8));
```

## What is SoA?

Whereas AoS stores all the fields of a type in each element of the array,
SoA splits each field into its own array. For example, consider 

```rust
struct Example {
    foo: u8,
    bar: u64,
}
```

In order to have proper memory alignment, this struct will have the following
layout. In this extreme example, almost half of the memory is wasted to padding.

```text
╭───┬───────────────────────────┬───────────────────────────────╮
│foo│         padding           │              bar              │
╰───┴───────────────────────────┴───────────────────────────────╯
```

By using SoA, the fields will be stored separately, removing the need for
padding:

```text
╭───┬───┬───┬───┬───┬───┬───┬───┬───┬───┬───┬───┬───┬───┬───┬───┬┄
│foo│foo│foo│foo│foo│foo│foo│foo│foo│foo│foo│foo│foo│foo│foo│foo│
╰───┴───┴───┴───┴───┴───┴───┴───┴───┴───┴───┴───┴───┴───┴───┴───┴┄
╭───────────────────────────────┬───────────────────────────────┬┄
│             bar               │              bar              │
╰───────────────────────────────┴───────────────────────────────┴┄
```

## Performance

In addition to lowering memory usage, there are several reasons why SoA can
offer better performance:

- By removing padding, each cacheline is typically more information-dense.
- When accessing only a subset of the available fields, only data for those
fields will be fetched. 

SoA does not offer performance wins in all cases. In particular, operations such
as `push` and `pop` are usually slower than for `Vec` since the memory for each
field is far apart. SoA is most appropriate when either

- Sequential access is the common access pattern
- You are frequently accessing or modifying only a subset of the fields

### SIMD vectorization

SoA makes getting data into and out of SIMD registers trivial. Since values are
stored sequentially, loading data is as simple as reading a range of memory into
the register. This bulk data transfer is very amenable to auto-vectorization. In
contrast, AoS stores fields at disjoint locations in memory. Therefore,
individual fields must be individually copied to different positions within the
registers and, later, shuffled back out in the same way. This can prevent the
compiler from applying vectorization. For this reason, SoA is much more likely
to benefit from SIMD optimizations. 

### Examples

#### Zig

SoA is a popular technique in data-oriented design. Andrew Kelley gives a
wonderful [talk](https://vimeo.com/649009599) describing how SoA and other
data-oriented design patterns earned him a 39% reduction in wall clock time
in the Zig compiler.

#### Benchmark

`soapy-testing` contains a
[benchmark](https://github.com/tim-harding/soapy/blob/92c12415d1fb8b9f2a015b35ff02a23b0e3aaa96/soapy-testing/benches/benchmark.rs#L82-L88)
comparison that sums the dot products of 2¹⁶ 4D vectors. The `Vec` version runs
in 132µs and the `Soa` version runs in 22µs, a 6x improvement. 

## Comparison

### [`soa_derive`](https://docs.rs/soa_derive/latest/soa_derive/)

`soa_derive` makes each field its own `Vec`. Because of this, each field's
length, capacity, and allocation are managed separately. In contrast, Soapy
manages a single allocation for each `Soa`. `soa_derive` also generates a new
collection type for every struct, whereas Soapy generates a minimal, low-level
interface that the generic `Soa` type uses for its implementation. This provides
more type system flexibility, less code generation, and better documentation.

### [`soa-vec`](https://docs.rs/soa-vec/latest/soa_vec/)

Whereas `soa-vec` only compiles on nightly, Soapy also compiles on stable.
Rather than using derive macros, `soa-vec` instead uses macros to generate
eight static copies of their SoA type with fixed tuple sizes.

![docs.rs](https://img.shields.io/docsrs/soapy?link=https%3A%2F%2Fdocs.rs%2Fsoapy%2Flatest%2Fsoapy%2F)
![Crates.io Version](https://img.shields.io/crates/v/soapy?link=https%3A%2F%2Fcrates.io%2Fcrates%2Fsoapy)
![GitHub License](https://img.shields.io/github/license/tim-harding/soapy?link=https%3A%2F%2Fgithub.com%2Ftim-harding%2Fsoapy%2Fblob%2Fmain%2FLICENSE)

# Soapy

Soapy makes it simple to work with the structure-of-arrays memory layout. What
`Vec<T>` is to array-of-structures (AoS), `Soa<T>` is to structure-of-arrays
(SoA).

## What is SoA?

The following types illustrate the difference between AoS and SoA:
```text
[(u8,   u64)] // AoS
([u8], [u64]) // SoA
```

Whereas AoS stores all the fields of a type in each element of the array,
SoA splits each field into its own array. This has several benefits:

- There is no padding required between instances of the same type. In the
above example, each AoS element requires 128 bits to satisfy memory
alignment requirements, whereas each SoA element only takes 72. This can
mean better cache locality and lower memory usage.
- SoA can be more amenable to vectorization. With SoA, multiple values can
be directly loaded into SIMD registers in bulk, as opposed to shuffling
struct fields into and out of different SIMD registers.

SoA is a popular technique in data-oriented design. Andrew Kelley gives a
wonderful [talk](https://vimeo.com/649009599) describing how SoA and other
data-oriented design patterns earned him a 39% reduction in wall clock time
in the Zig compiler.

Note that SoA does not offer performance wins in all cases. SoA is most
appropriate when either
- Sequential access is the common access pattern
- You are frequently accessing or modifying only a subset of the fields

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
    Baz { foo: 3, bar: 4 }
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

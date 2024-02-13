![docs.rs](https://img.shields.io/docsrs/soapy?link=https%3A%2F%2Fdocs.rs%2Fsoapy%2Flatest%2Fsoapy%2F)
![Crates.io Version](https://img.shields.io/crates/v/soapy?link=https%3A%2F%2Fcrates.io%2Fcrates%2Fsoapy)
![GitHub License](https://img.shields.io/github/license/tim-harding/soapy?link=https%3A%2F%2Fgithub.com%2Ftim-harding%2Fsoapy%2Fblob%2Fmain%2FLICENSE)

# Soapy

Soapy makes it simple to work with the structure-of-arrays memory layout. What `Vec<T>`
is to array-of-structures (AoS), `Soa<T>` is to structure-of-arrays (SoA).

## Examples

First, derive [`Soapy`] for your type:
```rust
# use soapy::{soa, Soapy};
#[derive(Soapy, Debug, Clone, Copy, PartialEq)]
struct Example {
    foo: u8,
    bar: u16,
}
```

You can create a [`Soa`] explicitly:
```rust
# use soapy::{soa, Soapy, Soa};
# #[derive(Soapy, Debug, Clone, Copy, PartialEq)]
# struct Example {
#     foo: u8,
#     bar: u16,
# }
let mut soa: Soa<Example> = Soa::new();
soa.push(Example { foo: 1, bar: 2 });
```

...or by using the [`soa!`] macro. 
```rust
# use soapy::{soa, Soapy};
# #[derive(Soapy, Debug, Clone, Copy, PartialEq)]
# struct Example {
#     foo: u8,
#     bar: u16,
# }
let mut soa = soa![
    Example { foo: 1, bar: 2 }, 
    Example { foo: 3, bar: 4 },
    Example { foo: 5, bar: 6 },
    Example { foo: 7, bar: 8 }
];
```

An SoA can be sliced just like a `&[T]`. Use `idx` in lieu of the index operator.
```rust
# use soapy::{soa, Soapy};
# #[derive(Soapy, Debug, Clone, Copy, PartialEq)]
# struct Example {
#     foo: u8,
#     bar: u16,
# }
# let mut soa = soa![
#     Example { foo: 1, bar: 2 }, 
#     Example { foo: 3, bar: 4 },
#     Example { foo: 5, bar: 6 },
#     Example { foo: 7, bar: 8 }
# ];
assert_eq!(soa.idx(1..3), [Example { foo: 3, bar: 4 }, Example { foo: 5, bar: 6 }]);
```

You can access the fields as slices. Add `_mut` for mutable slices. 
```rust
# use soapy::{soa, Soapy};
# #[derive(Soapy, Debug, Clone, Copy, PartialEq)]
# struct Example {
#     foo: u8,
#     bar: u16,
# }
# let mut soa = soa![
#     Example { foo: 1, bar: 2 }, 
#     Example { foo: 3, bar: 4 },
#     Example { foo: 5, bar: 6 },
#     Example { foo: 7, bar: 8 }
# ];
assert_eq!(soa.foo(), &[1, 3, 5, 7][..]);
soa.foo_mut().iter_mut().for_each(|foo| *foo += 10);
assert_eq!(soa.foo(), &[11, 13, 15, 17][..]);
```

The usual collection APIs and iterators work normally.
```rust
# use soapy::{soa, Soapy};
# #[derive(Soapy, Debug, Clone, Copy, PartialEq)]
# struct Example {
#     foo: u8,
#     bar: u16,
# }
# let mut soa = soa![
#     Example { foo: 1, bar: 2 }, 
#     Example { foo: 3, bar: 4 },
#     Example { foo: 5, bar: 6 },
#     Example { foo: 7, bar: 8 }
# ];
assert_eq!(soa.pop(), Some(Example { foo: 7, bar: 8 }));
for mut el in &mut soa {
    *el.bar += 10;
}
assert_eq!(soa.bar(), &[12, 14, 16][..]);
```

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

| Struct      | Field type | Use                                   |
|-------------|------------|---------------------------------------|
| `FooSoaRaw` | `*mut F`   | Low-level, unsafe interface for `Soa` |
| `FooRef`    | `&F`       | `.iter()`, `nth()`, `.get()`          |
| `FooRefMut` | `&mut F`   | `.iter_mut()`, `nth_mut`, `get_mut()` |

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

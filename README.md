# Soapy

Soapy is a structure-of-arrays derive macro. It creates an `Soa` type that
behaves like a `Vec` except that the elements of each field are stored in
separate arrays. This has several advantages: 

- No need for alignment-related padding, resulting in a more compact memory
layout
- Improved locality when iterating over a subset of the struct's fields
- Plays nicely with auto-vectorization as elements can be trivially
loaded into SIMD vectors without swizzling

Unlike other SOA crates, Soapy builds on stable and uses a single allocation 
to store the arrays of each field, rather than using separate `Vec`s for each 
field. 

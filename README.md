# relaxed-ref

This crate provides `RelaxedRef<'a, T>` and `RelaxedRefMut<'a, T>`,
relaxed versions of `&'a T` and `&'a mut T`, respectively, that are well-formed
without requiring `T: 'a`.
Instead, this bound is statically verified at the use site.
This is useful in GAT-based type families and other contexts where imposing
`T: 'a` on the surrounding interface is not always possible or desirable.

## Motivation

The reference types `&'a T` and `&'a mut T` require `T: 'a` to hold.
In some contexts, it is not possible to express this condition due to interface
constraints.
This commonly occurs with type families based on generic associated types
(GATs):

```rust
trait ValueFamily {
    type Of<T>;
}

struct RefFamily<'a>(PhantomData<&'a ()>);
impl<'a> ValueFamily for RefFamily<'a> {
    // error[E0309]: the parameter type `T` may not live long enough
    type Of<T> = &'a T;
}
```

In cases where modifying `ValueFamily` is impossible or undesirable,
we can replace `type Of<T> = &'a T` with `type Of<T> = RelaxedRef<'a, T>`,
since the latter is well-formed regardless of whether `T: 'a` holds:

```rust
impl<'a> ValueFamily for RefFamily<'a> {
    type Of<T> = RelaxedRef<'a, T>; // OK
}
```

## License

Licensed under either of

- Apache License, Version 2.0
  ([LICENSE-APACHE](LICENSE-APACHE) or <https://www.apache.org/licenses/LICENSE-2.0>)
- MIT license
  ([LICENSE-MIT](LICENSE-MIT) or <https://opensource.org/licenses/MIT>)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.

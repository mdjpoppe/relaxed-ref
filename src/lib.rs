//! This crate provides [`RelaxedRef<'a, T>`] and [`RelaxedRefMut<'a, T>`],
//! relaxed versions of `&'a T` and `&'a mut T`, respectively, that are
//! well-formed without requiring `T: 'a`.
//! Instead, this bound is statically verified at the use site.
//! This is useful in GAT-based type families and other contexts where imposing
//! `T: 'a` on the surrounding interface is not always possible or desirable.
//!
//! # Motivation
//!
//! The reference types `&'a T` and `&'a mut T` require the
//! [implied `T: 'a` bound][implied-bounds] to hold in order to be well-formed.
//! This is for a good reason, since it should not be possible for a reference
//! to outlive its referent.
//! In some contexts, it is not possible to impose `T: 'a` due to interface
//! constraints.
//! This commonly occurs with type families based on generic associated types
//! (GATs):
//!
//! ```compile_fail,E0309
//! # use std::marker::PhantomData;
//! #
//! trait ValueFamily {
//!     type Of<T>;
//! }
//!
//! struct RefFamily<'a>(PhantomData<&'a ()>);
//! impl<'a> ValueFamily for RefFamily<'a> {
//!     // error[E0309]: the parameter type `T` may not live long enough
//!     type Of<T> = &'a T;
//! }
//! ```
//!
//! In many cases, a pragmatic solution is to replace `type Of<T>` in
//! `ValueFamily` with `type Of<T: 'static>`, or `type Of<'a, T: 'a>` if `T`
//! need not be `'static`.
//! However, this requires changing the definition of `ValueFamily`,
//! which is impossible if it is defined in another crate and possibly a
//! breaking change otherwise.
//! Moreover, `type Of<'a, T: 'a>` imposes `T: 'a` on every use of `Of`,
//! even for implementations that do not use `'a`.
//! In such cases, `type Of<T> = RelaxedRef<'a, T>` provides an alternative that
//! keeps the lifetime constraints local to the implementations that use them:
//!
//! ```
//! # use std::marker::PhantomData;
//! # use relaxed_ref::RelaxedRef;
//! # trait ValueFamily {
//! #     type Of<T>;
//! # }
//! #
//! # struct RefFamily<'a>(PhantomData<&'a ()>);
//! impl<'a> ValueFamily for RefFamily<'a> {
//!     type Of<T> = RelaxedRef<'a, T>; // OK
//! }
//! ```
//!
//! See `examples/value_family.rs` for a slightly more complete example.
//!
//! # Usage and Examples
//!
//! [`RelaxedRef<'a, T>`] and [`RelaxedRefMut<'a, T>`] behave similarly to their
//! `&'a T` and `&'a mut T` counterparts.
//! They implement `Deref`, enabling methods on `T` to be called directly.
//! To avoid name clashes with methods on `T`, the wrappers expose their own
//! functionality through associated functions.
//! For example, [`RelaxedRef::into_ref`] converts a `RelaxedRef<'a, T>` into
//! `&'a T` (if `T: 'a`).
//!
//! ```
//! # use relaxed_ref::RelaxedRef;
//! #
//! # fn f<'a>() {
//! #
//! let int_ref: &'a i32 = todo!();
//!
//! // construct a `RelaxedRef<'a, i32>`
//! let int_rref: RelaxedRef<'a, i32> = RelaxedRef::new(int_ref);
//! // or
//! let int_rref: RelaxedRef<'a, i32> = int_ref.into();
//!
//! // convert it back
//! let _: &'a i32 = RelaxedRef::into_ref(int_rref);
//! #
//! # }
//! ```
//!
//! `RelaxedRef<'a, T>` and `RelaxedRefMut<'a, T>` implement many common traits
//! also implemented by `&'a T` and `&'a mut T`, such as `Copy` and `Clone`
//! for `RelaxedRef<'a, T>`, `Deref` for both, and `DerefMut` for
//! `RelaxedRefMut<'a, T>`.
//! Traits like `Display` and `PartialEq` are implemented conditionally on `T`
//! implementing these.
//!
//! ```
//! # use relaxed_ref::RelaxedRef;
//! #
//! let i = 0;
//! let r = RelaxedRef::new(&i);
//!
//! // like a reference, `RelaxedRef<'a, T>` is `Copy`
//! let copies = (r, r, r);
//!
//! // prints, e.g., "r: 0, ptr: 0x7ffd2616e6c0"
//! println!("r: {r}, ptr: {ptr:p}", ptr = RelaxedRef::as_ptr(&r));
//!
//! let x = RelaxedRef::new(&1);
//! let y = RelaxedRef::new(&1);
//!
//! // `RelaxedRef<'a, T>` implements `PartialEq` if `T` does
//! assert_eq!(x, y);
//!
//! // we can dereference `RelaxedRef<'a, T>` using `*`
//! let sum = *x + *y;
//! assert_eq!(sum, 2);
//!
//! // we can also call methods of `T` directly
//! let s = RelaxedRef::new("test");
//! assert_eq!(s.len(), 4);
//! ```
//!
//! `RelaxedRefMut<'a, T>` is similar to `RelaxedRef<'a, T>`, but also allows
//! mutating the referent:
//!
//! ```
//! # use relaxed_ref::RelaxedRefMut;
//! #
//! let mut i = 0;
//! let mut r = RelaxedRefMut::new(&mut i);
//! *r = 1;
//! assert_eq!(i, 1);
//! ```
//!
//! # Soundness
//!
//! Even though [`RelaxedRef<'a, T>`] and [`RelaxedRefMut<'a, T>`] are
//! well-formed independently of whether `T: 'a`, they cannot be used to create
//! invalid references.
//! The documentation for each type explains why its implementation is sound.
//!
//! # Internals
//!
//! [`RelaxedRef<'a, T>`] and [`RelaxedRefMut<'a, T>`] internally store a
//! [`NonNull<T>`](core::ptr::NonNull) pointer.
//! (They cannot simply wrap a `&'a T` or `&'a mut T`, since that would impose
//! `T: 'a`, defeating the purpose of these types.)
//!
//! Both types are `#[repr(transparent)]` and are guaranteed to have the same
//! size and alignment as their corresponding reference types.
//! They also benefit from the null pointer optimization, so it is guaranteed
//! that
//! `size_of::<Option<RelaxedRef<'a, T>>>() == size_of::<RelaxedRef<'a, T>>()`
//! and
//! `size_of::<Option<RelaxedRefMut<'a, T>>>() == size_of::<RelaxedRefMut<'a, T>>()`
//! for all types `T` (and lifetimes `'a`).
//!
//! # MSRV
//!
//! The minimum supported Rust version (MSRV) is 1.56.0.
//! Note that GATs, the main use case of this crate, were stabilized in Rust
//! 1.65.0.
//! The crate itself does not use them, but some examples do.
//!
//! Some functions are only `const fn` on Rust 1.58.0+ or 1.83.0+.
//! This is documented per function.
//!
//! [implied-bounds]: https://doc.rust-lang.org/reference/trait-bounds.html#implied-bounds

#![no_std]
#![deny(unsafe_op_in_unsafe_fn)]
#![warn(missing_docs)]

mod relaxed_ref;
mod relaxed_ref_mut;

pub use relaxed_ref::RelaxedRef;
pub use relaxed_ref_mut::RelaxedRefMut;

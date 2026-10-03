use core::{
    borrow::Borrow,
    cmp::Ordering,
    fmt::{self, Debug, Display, Formatter, Pointer},
    hash::{Hash, Hasher},
    marker::PhantomData,
    ops::Deref,
    ptr::NonNull,
};

use crate::RelaxedRefMut;

/// Relaxed version of `&'a T` that is well-formed without requiring `T: 'a`.
///
/// # Usage and Examples
///
/// A `RelaxedRef<'a, T>` mostly behaves like a `&'a T`;
/// it dereferences to `T` and implements many of the traits that `&'a T` does.
///
/// ```
/// # use relaxed_ref::RelaxedRef;
/// #
/// let i = 0;
///
/// // convert `&i32` to `RelaxedRef<'_, i32>`
/// let i_rref = RelaxedRef::new(&i);
///
/// // access `i` through `RelaxedRef<'_, i32>`
/// let j = *i_rref + 1;
/// assert_eq!(j, 1);
///
/// // convert `RelaxedRef<'_, i32>` back into `&i32`
/// let i_ref: &i32 = RelaxedRef::into_ref(i_rref);
/// ```
///
/// `RelaxedRef<'a, T>` can be used in the following GAT-based type family,
/// whereas `&'a T` cannot:
///
/// ```
/// # use std::marker::PhantomData;
/// # use relaxed_ref::RelaxedRef;
/// #
/// trait ValueFamily {
///     type Of<T>;
/// }
///
/// struct RefFamily<'a>(PhantomData<&'a ()>);
/// impl<'a> ValueFamily for RefFamily<'a> {
///     type Of<T> = RelaxedRef<'a, T>;
/// }
/// ```
///
/// See the [crate-level documentation](crate#usage-and-examples) for more usage
/// details and examples.
///
/// # Comparison with `&'a T`
///
/// Like a `&'a T` reference, `RelaxedRef<'a, T>` is:
///
/// - covariant in `'a`, and
/// - covariant in `T`.
///
/// Unlike references, however, it is possible to have a `RelaxedRef<'a, T>`
/// where `T` does not outlive `'a`.
/// This does not introduce unsoundness, since converting `RelaxedRef<'a, T>` to
/// `&'a T` requires that `T` outlives `'a`.
/// Consider the following example, where `'short` and `'long` are lifetimes
/// such that `'long: 'short` (but not necessarily `'short: 'long`), and `T` is
/// a type such that `T: 'long`:
///
/// ```
/// # use relaxed_ref::RelaxedRef;
/// #
/// # fn f<'short, 'long: 'short, T: 'long>() {
/// #
/// // Coercing `&'long &'long T` into `&'long &'short T` is invalid:
/// let r_ll: &'long &'long T = todo!();
/// // let _: &'long &'short T = r_ll; // error: lifetime may not live long enough
///
/// // Coercing `RelaxedRef<'long, &'long T>` into `RelaxedRef<'long, &'short T>`
/// // is perfectly fine ...
/// let rr_ll: RelaxedRef<'long, &'long T> = RelaxedRef::new(r_ll);
/// let rr_ls: RelaxedRef<'long, &'short T> = rr_ll;
///
/// // ... but `rr_ls` cannot be converted into a reference:
/// // let _: &'long &'short T = RelaxedRef::into_ref(rr_ls); // error: lifetime may not live long enough
///
/// // `RelaxedRef<'long, &'short T>` can be coerced into `RelaxedRef<'short, &'short T>`,
/// // which *can* be converted into a reference
/// let rr_ss: RelaxedRef<'short, &'short T> = rr_ls;
/// let r_ss: &'short &'short T = RelaxedRef::into_ref(rr_ss);
/// #
/// # }
/// ```
///
/// # Soundness
///
/// `RelaxedRef<'a, T>` maintains the following invariant:
///  >  For every lifetime `'r` and type `R`, if
///  >
///  >  - `'a: 'r`,
///  >  - `T <: R`, and
///  >  - `R: 'r`,
///  >
///  >  then `RelaxedRef<'a, T>` represents a valid `&'r R` reference.
///  >  (Here, `<:` denotes the subtype relation.)
///  >
///  >  As a consequence, a `RelaxedRef<'a, T>` represents a valid `&'a T` reference
///  >  if `T: 'a`.
///
/// (At first glance, it seems that the invariant above can be simplified to
/// "`RelaxedRef<'a, T>` represents a valid `&'a T` reference if `T: 'a`".
/// While being certainly true, this statement is too weak to serve as an
/// invariant, since it is not preserved by covariance.)
///
/// <details>
/// <summary>Proof</summary>
///
/// **Constructor.**
/// The constructor of `RelaxedRef<'a, T>` receives a `&'a T` reference.
/// If `'r` and `R` are such that `'a: 'r`, `T <: R`, and `R: 'r`, then
/// `&'a T <: &'r R`, so `RelaxedRef<'a, T>` represents a valid `&'r R`
/// reference.
///
/// **Covariance.**
/// The invariant is preserved by covariance of `RelaxedRef<'a, T>` in `'a` and
/// `T`:
/// Suppose the invariant holds for `RelaxedRef<'a, T>`, and consider
/// `RelaxedRef<'b, U>`, where `'b` is a lifetime such that `'a: 'b`, and `U` is
/// a type such that `T <: U`.
/// If `'r` and `R` are such that `'b: 'r`, `U <: R`, and `R: 'r`, then
/// `'a: 'r` and `T <: R` by transitivity of `:` and `<:`.
/// Since `RelaxedRef<'b, U>` wraps the same referent as `RelaxedRef<'a, T>`,
/// it represents a valid `&'r R` reference.
///
/// **Copying and Cloning.**
/// These operations trivially preserve the invariant, as shared references are
/// `Copy`.
///
/// </details>
#[repr(transparent)]
pub struct RelaxedRef<'a, T: ?Sized> {
    /// Internal pointer. Also makes `RelaxedRef<'a, T>` covariant in `T`.
    ptr: NonNull<T>,

    /// Uses `'a` and makes `RelaxedRef<'a, T>` covariant in `'a`.
    _lifetime: PhantomData<&'a ()>,
}

impl<'a, T: ?Sized> RelaxedRef<'a, T> {
    /// Constructs a `RelaxedRef<'a, T>` from a `&'a T` reference.
    #[must_use]
    pub const fn new(reference: &'a T) -> Self {
        // SAFETY: References are never null.
        let ptr = unsafe { NonNull::new_unchecked(reference as *const T as *mut T) };
        Self {
            ptr,
            _lifetime: PhantomData,
        }
    }

    /// Constructs a `RelaxedRef<'a, T>` directly from a `NonNull<T>`.
    ///
    /// # Safety
    /// The caller must ensure that the invariant described in the
    /// [soundness section](#soundness) is upheld.
    #[must_use]
    pub const unsafe fn from_non_null(ptr: NonNull<T>) -> Self {
        Self {
            ptr,
            _lifetime: PhantomData,
        }
    }

    /// Returns a `&'s T` reference which is valid for the duration of the
    /// `this` borrow.
    ///
    /// This is `const fn` on Rust 1.58.0+.
    #[allow(clippy::needless_lifetimes)]
    #[must_use]
    #[rustversion::attr(since(1.58.0), const)]
    pub fn get<'s>(this: &'s Self) -> &'s T {
        RelaxedRef::<'s, T>::into_ref(*this)
    }

    /// Returns the underlying const pointer.
    #[must_use]
    pub const fn as_ptr(this: &Self) -> *const T {
        this.ptr.as_ptr()
    }

    /// Returns the underlying `NonNull<T>`.
    #[must_use]
    pub const fn as_non_null(this: &Self) -> NonNull<T> {
        this.ptr
    }

    /// Converts this `RelaxedRef<'a, T>` to a `&'a T` reference,
    /// given that `T: 'a`.
    ///
    /// Note that there is no explicit `T: 'a` bound, since this is already
    /// implied by the return type `&'a T`.
    ///
    /// This is `const fn` on Rust 1.58.0+.
    #[allow(clippy::needless_pass_by_value)]
    #[must_use]
    #[rustversion::attr(since(1.58.0), const)]
    pub fn into_ref(this: Self) -> &'a T {
        // the `as *const T` cast allows this function to be `const fn` on older
        // Rust versions
        // SAFETY: Follows from the invariant in the soundness section with
        // `'r = 'a` and `R = T`.
        unsafe { &*(this.ptr.as_ptr() as *const T) }
    }
}

impl<T: ?Sized> Copy for RelaxedRef<'_, T> {}

impl<T: ?Sized> Clone for RelaxedRef<'_, T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: ?Sized> Deref for RelaxedRef<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        Self::get(self)
    }
}

impl<T: ?Sized> Borrow<T> for RelaxedRef<'_, T> {
    fn borrow(&self) -> &T {
        self
    }
}

impl<T: ?Sized> Pointer for RelaxedRef<'_, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Pointer::fmt(&self.ptr, f)
    }
}

impl<T: ?Sized + Display> Display for RelaxedRef<'_, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Display::fmt(&**self, f)
    }
}

impl<T: ?Sized + Debug> Debug for RelaxedRef<'_, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Debug::fmt(&**self, f)
    }
}

impl<T: ?Sized + PartialOrd<U>, U: ?Sized> PartialOrd<RelaxedRef<'_, U>> for RelaxedRef<'_, T> {
    fn partial_cmp(&self, other: &RelaxedRef<'_, U>) -> Option<Ordering> {
        PartialOrd::partial_cmp(&**self, &**other)
    }

    fn lt(&self, other: &RelaxedRef<'_, U>) -> bool {
        PartialOrd::lt(&**self, &**other)
    }

    fn le(&self, other: &RelaxedRef<'_, U>) -> bool {
        PartialOrd::le(&**self, &**other)
    }

    fn gt(&self, other: &RelaxedRef<'_, U>) -> bool {
        PartialOrd::gt(&**self, &**other)
    }

    fn ge(&self, other: &RelaxedRef<'_, U>) -> bool {
        PartialOrd::ge(&**self, &**other)
    }
}

impl<T: ?Sized + Ord> Ord for RelaxedRef<'_, T> {
    fn cmp(&self, other: &Self) -> Ordering {
        Ord::cmp(&**self, &**other)
    }
}

impl<T: ?Sized + PartialEq<U>, U: ?Sized> PartialEq<RelaxedRef<'_, U>> for RelaxedRef<'_, T> {
    fn eq(&self, other: &RelaxedRef<'_, U>) -> bool {
        PartialEq::eq(&**self, &**other)
    }

    #[allow(clippy::partialeq_ne_impl)]
    fn ne(&self, other: &RelaxedRef<'_, U>) -> bool {
        PartialEq::ne(&**self, &**other)
    }
}

impl<T: ?Sized + Eq> Eq for RelaxedRef<'_, T> {}

impl<T: ?Sized + AsRef<U>, U: ?Sized> AsRef<U> for RelaxedRef<'_, T> {
    fn as_ref(&self) -> &U {
        (**self).as_ref()
    }
}

impl<T: ?Sized + Hash> Hash for RelaxedRef<'_, T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        Hash::hash(&**self, state);
    }
}

// SAFETY: `&T` is `Sync` whenever `T` is `Sync`.
unsafe impl<T: ?Sized + Sync> Sync for RelaxedRef<'_, T> {}

// SAFETY: `&T` is `Send` whenever `T` is `Sync`.
unsafe impl<T: ?Sized + Sync> Send for RelaxedRef<'_, T> {}

impl<'a, T: ?Sized> From<&'a T> for RelaxedRef<'a, T> {
    fn from(reference: &'a T) -> Self {
        RelaxedRef::new(reference)
    }
}

impl<'a, T: ?Sized> From<RelaxedRefMut<'a, T>> for RelaxedRef<'a, T> {
    fn from(reference: RelaxedRefMut<'a, T>) -> Self {
        RelaxedRefMut::into_relaxed_ref(reference)
    }
}

#[allow(clippy::items_after_statements)]
#[allow(clippy::std_instead_of_alloc)]
#[allow(clippy::std_instead_of_core)]
#[allow(clippy::used_underscore_items)]
#[cfg(test)]
mod tests {
    extern crate std;
    use std::{
        cell::Cell,
        panic::{RefUnwindSafe, UnwindSafe},
        rc::Rc,
    };

    use assert_impl_trait::assert_impl;
    use static_assertions::assert_not_impl_any;

    use super::*;

    #[allow(clippy::similar_names)]
    #[test]
    fn round_trip() {
        let s = "test";
        let s_rref = RelaxedRef::new(s);
        let s_ref = RelaxedRef::into_ref(s_rref);
        assert_eq!(s_ref, s);
    }

    #[test]
    fn dereference_str() {
        let s_rref = RelaxedRef::new("test");
        assert_eq!(s_rref.len(), 4);
    }

    #[test]
    fn dereference_i32() {
        let i_rref = RelaxedRef::new(&123);
        assert_eq!(*i_rref, 123);
    }

    #[test]
    fn dereference_slice() {
        let slice_rref = RelaxedRef::new(&[1, 2, 3][..]);
        assert_eq!(*slice_rref, [1, 2, 3]);
    }

    fn _lifetime_covariance<'short, 'mid: 'short, 'long: 'mid, T>(
        r: RelaxedRef<'mid, &'long T>,
    ) -> RelaxedRef<'short, &'long T> {
        r
    }

    fn _type_covariance<'short, 'mid: 'short, 'long: 'mid, T>(
        r: RelaxedRef<'short, &'long T>,
    ) -> RelaxedRef<'short, &'mid T> {
        r
    }

    assert_impl!(
        for<'a, T: ?Sized> {
            RelaxedRef<'a, T>: Copy + Clone
        }
    );

    assert_impl!(
        for<'a, T: ?Sized + RefUnwindSafe> {
            RelaxedRef<'a, T>: UnwindSafe
        }
    );

    assert_not_impl_any!(RelaxedRef<'static, Rc<i32>>: Send, Sync);

    assert_impl!(
        for<'a, T: ?Sized + Sync> {
            RelaxedRef<'a, T>: Send
        }
    );

    assert_impl!(
        for<'a, T: ?Sized + Sync> {
            RelaxedRef<'a, T>: Sync
        }
    );

    assert_not_impl_any!(RelaxedRef<'static, Cell<i32>>: Send, Sync);

    const _SAME_SIZE_AND_ALIGNMENT_AS_REF: () = {
        assert!(size_of::<RelaxedRef<'static, u8>>() == size_of::<&u8>());
        assert!(align_of::<RelaxedRef<'static, u8>>() == align_of::<&u8>());
        assert!(size_of::<RelaxedRef<'static, str>>() == size_of::<&str>());
        assert!(align_of::<RelaxedRef<'static, str>>() == align_of::<&str>());
        assert!(size_of::<Option<RelaxedRef<'static, u8>>>() == size_of::<&u8>());
        assert!(align_of::<Option<RelaxedRef<'static, u8>>>() == align_of::<&u8>());
    };

    const _NULL_POINTER_OPTIMIZATION: () = {
        assert!(
            size_of::<Option<RelaxedRef<'static, u8>>>() == size_of::<RelaxedRef<'static, u8>>()
        );
        assert!(
            size_of::<Option<RelaxedRef<'static, str>>>() == size_of::<RelaxedRef<'static, str>>()
        );
    };
}

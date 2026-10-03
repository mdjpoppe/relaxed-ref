use core::{
    borrow::{Borrow, BorrowMut},
    cmp::Ordering,
    fmt::{self, Debug, Display, Formatter, Pointer},
    hash::{Hash, Hasher},
    marker::PhantomData,
    ops::{Deref, DerefMut},
    ptr::NonNull,
};

use crate::RelaxedRef;

/// Relaxed version of `&'a mut T` that is well-formed without requiring
/// `T: 'a`.
///
/// # Usage and Examples
///
/// A `RelaxedRefMut<'a, T>` mostly behaves like a `&'a mut T`;
/// it dereferences to `T` and implements many of the traits that `&'a mut T`
/// does.
///
/// ```
/// # use relaxed_ref::RelaxedRefMut;
/// #
/// let mut i = 0;
///
/// // convert `&mut i32` to `RelaxedRefMut<'_, i32>`
/// let mut i_rref = RelaxedRefMut::new(&mut i);
///
/// // access `i` through `RelaxedRefMut<'_, i32>`
/// *i_rref += 1;
/// assert_eq!(*i_rref, 1);
///
/// // convert `RelaxedRefMut<'_, i32>` back into `&mut i32`
/// let i_ref: &mut i32 = RelaxedRefMut::into_mut(i_rref);
///
/// // verify that we really did modify `i`
/// assert_eq!(i, 1);
/// ```
///
/// `RelaxedRefMut<'a, T>` can be used in the following GAT-based type family,
/// whereas `&'a mut T` cannot:
///
/// ```
/// # use std::marker::PhantomData;
/// # use relaxed_ref::RelaxedRefMut;
/// #
/// trait ValueFamily {
///     type Of<T>;
/// }
///
/// struct RefMutFamily<'a>(PhantomData<&'a ()>);
/// impl<'a> ValueFamily for RefMutFamily<'a> {
///     type Of<T> = RelaxedRefMut<'a, T>;
/// }
/// ```
///
/// See the [crate-level documentation](crate#usage-and-examples) for more usage
/// details and examples.
///
/// # Comparison with `&'a mut T`
///
/// Like a `&'a mut T` reference, `RelaxedRefMut<'a, T>` is:
///
/// - covariant in `'a`, and
/// - invariant in `T`.
///
/// Consequently, only the lifetime parameter of `RelaxedRefMut<'a, T>` can be
/// shortened, and `RelaxedRefMut<'a, T>` always represents a valid `&'a mut T`
/// reference.
///
/// In the following example, `'short` and `'long` are lifetimes such that
/// `'long: 'short` (but not necessarily `'short: 'long`), and `T` is a type
/// such that `T: 'long`:
///
/// ```
/// # use relaxed_ref::RelaxedRefMut;
/// #
/// # fn f<'short, 'long: 'short, T: 'long>() {
/// #
/// let r_ll: &'long mut &'long T = todo!();
/// let rr_ll: RelaxedRefMut<'long, &'long T> = RelaxedRefMut::new(r_ll);
/// let rr_sl: RelaxedRefMut<'short, &'long T> = rr_ll;
/// // let rr_ss: RelaxedRefMut<'short, &'short T> = rr_sl; // error: lifetime may not live long enough
/// let r_sl: &'short mut &'long T = RelaxedRefMut::into_mut(rr_sl);
/// #
/// # }
/// ```
///
/// # Soundness
///
/// Soundness of `RelaxedRefMut<'a, T>` is simpler than that of
/// [`RelaxedRef<'a, T>`], since `RelaxedRefMut<'a, T>` is covariant only in
/// `'a`.
/// A `RelaxedRefMut<'a, T>` therefore always represents a valid `&'a mut T`
/// reference:
/// this is certainly true for the constructor, and if `'b` is a lifetime such
/// that `'a: 'b`, then `RelaxedRefMut<'b, T>` indeed represents a valid
/// `&'b mut T` reference.
/// Moreover, `RelaxedRefMut<'a, T>` does not implement `Copy` or `Clone`,
/// preserving uniqueness of the reference.
#[repr(transparent)]
pub struct RelaxedRefMut<'a, T: ?Sized> {
    /// Internal pointer.
    ptr: NonNull<T>,

    /// Uses `'a` and makes `RelaxedRefMut<'a, T>` covariant in `'a`.
    /// Additionally makes this type `!UnwindSafe`.
    _lifetime: PhantomData<&'a mut ()>,

    /// Makes `RelaxedRefMut<'a, T>` invariant in `T`.
    _invariance: PhantomData<*mut T>,
}

impl<'a, T: ?Sized> RelaxedRefMut<'a, T> {
    /// Constructs a `RelaxedRefMut<'a, T>` from a `&'a mut T` reference.
    ///
    /// This is `const fn` on Rust 1.83.0+.
    #[must_use]
    #[rustversion::attr(since(1.83.0), const)]
    pub fn new(reference: &'a mut T) -> Self {
        // SAFETY: References are never null.
        let ptr = unsafe { NonNull::new_unchecked(reference as *mut T) };
        Self {
            ptr,
            _lifetime: PhantomData,
            _invariance: PhantomData,
        }
    }

    /// Constructs a `RelaxedRefMut<'a, T>` directly from a `NonNull<T>`.
    ///
    /// This is `const fn` on Rust 1.83.0+.
    ///
    /// # Safety
    /// The caller must ensure that `ptr` represents a valid `&'a mut T`
    /// reference.
    #[must_use]
    #[rustversion::attr(since(1.83.0), const)]
    pub unsafe fn from_non_null(ptr: NonNull<T>) -> Self {
        Self {
            ptr,
            _lifetime: PhantomData,
            _invariance: PhantomData,
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
        // the `as *const T` cast allows this function to be `const fn` on older
        // Rust versions
        // SAFETY: `this` represents a valid `&'a mut T` reference, which may be
        // reborrowed as a `&'s T` reference.
        unsafe { &*(this.ptr.as_ptr() as *const T) }
    }

    /// Returns a `&'s mut T` reference which is valid for the duration of the
    /// `this` borrow.
    ///
    /// This is `const fn` on Rust 1.83.0+.
    #[allow(clippy::needless_lifetimes)]
    #[must_use]
    #[rustversion::attr(since(1.83.0), const)]
    pub fn get_mut<'s>(this: &'s mut Self) -> &'s mut T {
        // SAFETY: `this` represents a valid `&'a mut T` reference, which may be
        // reborrowed as a `&'s mut T` reference.
        unsafe { &mut *this.ptr.as_ptr() }
    }

    /// Returns the underlying const pointer.
    #[must_use]
    pub const fn as_ptr(this: &Self) -> *const T {
        this.ptr.as_ptr()
    }

    /// Returns the underlying mutable pointer.
    ///
    /// This is `const fn` on Rust 1.83.0+.
    #[must_use]
    #[rustversion::attr(since(1.83.0), const)]
    pub fn as_mut_ptr(this: &mut Self) -> *mut T {
        this.ptr.as_ptr()
    }

    /// Returns the underlying `NonNull<T>`.
    ///
    /// This is `const fn` on Rust 1.83.0+.
    #[must_use]
    #[rustversion::attr(since(1.83.0), const)]
    pub fn as_non_null(this: &mut Self) -> NonNull<T> {
        this.ptr
    }

    /// Returns a `RelaxedRefMut<'s, T>` which is valid for the duration of the
    /// `this` borrow.
    ///
    /// This is `const fn` on Rust 1.83.0+.
    #[allow(clippy::elidable_lifetime_names)]
    #[must_use]
    #[rustversion::attr(since(1.83.0), const)]
    pub fn reborrow<'s>(this: &'s mut Self) -> RelaxedRefMut<'s, T> {
        RelaxedRefMut {
            ptr: this.ptr,
            _lifetime: PhantomData,
            _invariance: PhantomData,
        }
    }

    /// Converts this `RelaxedRefMut<'a, T>` into a `&'a T` reference.
    ///
    /// This is `const fn` on Rust 1.58.0+.
    #[allow(clippy::needless_pass_by_value)]
    #[must_use]
    #[rustversion::attr(since(1.58.0), const)]
    pub fn into_ref(this: Self) -> &'a T {
        // the `as *const T` cast allows this function to be `const fn` on older
        // Rust versions
        // SAFETY: Follows from the invariant in the soundness section.
        unsafe { &*(this.ptr.as_ptr() as *const T) }
    }

    /// Converts this `RelaxedRefMut<'a, T>` into a `&'a mut T` reference.
    ///
    /// This is `const fn` on Rust 1.83.0+.
    #[allow(clippy::needless_pass_by_value)]
    #[must_use]
    #[rustversion::attr(since(1.83.0), const)]
    pub fn into_mut(this: Self) -> &'a mut T {
        // SAFETY: Follows from the invariant in the soundness section.
        unsafe { &mut *this.ptr.as_ptr() }
    }

    /// Returns a `RelaxedRef<'s, T>` which is valid for the duration of the
    /// `this` borrow.
    ///
    /// This is `const fn` on Rust 1.58.0+.
    #[allow(clippy::elidable_lifetime_names)]
    #[must_use]
    #[rustversion::attr(since(1.58.0), const)]
    pub fn as_relaxed_ref<'s>(this: &'s Self) -> RelaxedRef<'s, T> {
        RelaxedRef::new(Self::get(this))
    }

    /// Converts this `RelaxedRefMut<'a, T>` into a `RelaxedRef<'a, T>`.
    /// This is analogous to converting a `&'a mut T` reference into a `&'a T`
    /// reference.
    #[allow(clippy::needless_pass_by_value)]
    #[must_use]
    pub const fn into_relaxed_ref(this: Self) -> RelaxedRef<'a, T> {
        // SAFETY: `this` represents a valid `&'a mut T` reference.
        unsafe { RelaxedRef::from_non_null(this.ptr) }
    }
}

impl<T: ?Sized> Deref for RelaxedRefMut<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        Self::get(self)
    }
}

impl<T: ?Sized> DerefMut for RelaxedRefMut<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        Self::get_mut(self)
    }
}

impl<T: ?Sized> Borrow<T> for RelaxedRefMut<'_, T> {
    fn borrow(&self) -> &T {
        self
    }
}

impl<T: ?Sized> BorrowMut<T> for RelaxedRefMut<'_, T> {
    fn borrow_mut(&mut self) -> &mut T {
        self
    }
}

impl<T: ?Sized> Pointer for RelaxedRefMut<'_, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Pointer::fmt(&self.ptr, f)
    }
}

impl<T: ?Sized + Display> Display for RelaxedRefMut<'_, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Display::fmt(&**self, f)
    }
}

impl<T: ?Sized + Debug> Debug for RelaxedRefMut<'_, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Debug::fmt(&**self, f)
    }
}

impl<T: ?Sized + PartialOrd<U>, U: ?Sized> PartialOrd<RelaxedRefMut<'_, U>>
    for RelaxedRefMut<'_, T>
{
    fn partial_cmp(&self, other: &RelaxedRefMut<'_, U>) -> Option<Ordering> {
        PartialOrd::partial_cmp(&**self, &**other)
    }

    fn lt(&self, other: &RelaxedRefMut<'_, U>) -> bool {
        PartialOrd::lt(&**self, &**other)
    }

    fn le(&self, other: &RelaxedRefMut<'_, U>) -> bool {
        PartialOrd::le(&**self, &**other)
    }

    fn gt(&self, other: &RelaxedRefMut<'_, U>) -> bool {
        PartialOrd::gt(&**self, &**other)
    }

    fn ge(&self, other: &RelaxedRefMut<'_, U>) -> bool {
        PartialOrd::ge(&**self, &**other)
    }
}

impl<T: ?Sized + Ord> Ord for RelaxedRefMut<'_, T> {
    fn cmp(&self, other: &Self) -> Ordering {
        Ord::cmp(&**self, &**other)
    }
}

impl<T: ?Sized + PartialEq<U>, U: ?Sized> PartialEq<RelaxedRefMut<'_, U>> for RelaxedRefMut<'_, T> {
    fn eq(&self, other: &RelaxedRefMut<'_, U>) -> bool {
        PartialEq::eq(&**self, &**other)
    }

    #[allow(clippy::partialeq_ne_impl)]
    fn ne(&self, other: &RelaxedRefMut<'_, U>) -> bool {
        PartialEq::ne(&**self, &**other)
    }
}

impl<T: ?Sized + Eq> Eq for RelaxedRefMut<'_, T> {}

impl<T: ?Sized + AsRef<U>, U: ?Sized> AsRef<U> for RelaxedRefMut<'_, T> {
    fn as_ref(&self) -> &U {
        (**self).as_ref()
    }
}

impl<T: ?Sized + AsMut<U>, U: ?Sized> AsMut<U> for RelaxedRefMut<'_, T> {
    fn as_mut(&mut self) -> &mut U {
        (**self).as_mut()
    }
}

impl<T: ?Sized + Hash> Hash for RelaxedRefMut<'_, T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        Hash::hash(&**self, state);
    }
}

// SAFETY: `&mut T` is `Sync` whenever `T` is `Sync`.
unsafe impl<T: ?Sized + Sync> Sync for RelaxedRefMut<'_, T> {}

// SAFETY: `&mut T` is `Send` whenever `T` is `Send`.
unsafe impl<T: ?Sized + Send> Send for RelaxedRefMut<'_, T> {}

impl<'a, T: ?Sized> From<&'a mut T> for RelaxedRefMut<'a, T> {
    fn from(reference: &'a mut T) -> Self {
        RelaxedRefMut::new(reference)
    }
}

#[allow(clippy::items_after_statements)]
#[allow(clippy::std_instead_of_alloc)]
#[allow(clippy::std_instead_of_core)]
#[allow(clippy::used_underscore_items)]
#[cfg(test)]
mod tests {
    extern crate std;
    use std::{cell::Cell, panic::UnwindSafe, rc::Rc};

    use assert_impl_trait::assert_impl;
    use static_assertions::assert_not_impl_any;

    use super::*;

    #[test]
    fn round_trip() {
        let mut i = 0;
        let ptr = std::ptr::addr_of_mut!(i);
        let mut i_rref = RelaxedRefMut::new(&mut i);
        assert_eq!(ptr, RelaxedRefMut::as_mut_ptr(&mut i_rref));
        *RelaxedRefMut::into_mut(i_rref) = 1;
        assert_eq!(i, 1);
    }

    #[test]
    fn into_relaxed_ref() {
        let mut i = 0;
        let ptr = std::ptr::addr_of!(i);
        let mut i_rref_mut = RelaxedRefMut::new(&mut i);
        *i_rref_mut = 1;
        let i_rref = RelaxedRefMut::into_relaxed_ref(i_rref_mut);
        assert_eq!(ptr, RelaxedRef::as_ptr(&i_rref));
        assert_eq!(*RelaxedRef::into_ref(i_rref), 1);
    }

    #[test]
    fn dereference_i32() {
        let mut i = 123;
        let i_rref = RelaxedRefMut::new(&mut i);
        assert_eq!(*i_rref, 123);
    }

    #[test]
    fn dereference_i32_mut() {
        let mut i = 123;
        let mut i_rref = RelaxedRefMut::new(&mut i);
        *i_rref = 456;
        assert_eq!(*i_rref, 456);
    }

    #[test]
    fn dereference_slice() {
        let slice = &mut [1, 2, 3][..];
        let slice_rref = RelaxedRefMut::new(slice);
        assert_eq!(*slice_rref, [1, 2, 3]);
    }

    #[test]
    fn dereference_slice_mut() {
        let slice = &mut [1, 2, 3][..];
        let mut slice_rref = RelaxedRefMut::new(slice);

        slice_rref[2] = 4;
        assert_eq!(slice, &[1, 2, 4]);
    }

    fn _lifetime_covariance<'short, 'long: 'short, T>(
        r: RelaxedRefMut<'long, &'long T>,
    ) -> RelaxedRefMut<'short, &'long T> {
        r
    }

    assert_not_impl_any!(RelaxedRefMut<'static, i32>: Copy, Clone, UnwindSafe);
    assert_not_impl_any!(RelaxedRefMut<'static, Cell<i32>>: Sync);
    assert_impl!(
        for<'a, T: ?Sized + Send> {
            RelaxedRefMut<'a, Cell<T>>: Send
        }
    );

    assert_not_impl_any!(RelaxedRefMut<'static, Rc<i32>>: Send, Sync);

    assert_impl!(
        for<'a, T: ?Sized + Send> {
            RelaxedRefMut<'a, T>: Send
        }
    );

    assert_impl!(
        for<'a, T: ?Sized + Sync> {
            RelaxedRefMut<'a, T>: Sync
        }
    );

    const _SAME_SIZE_AND_ALIGNMENT_AS_MUT_REF: () = {
        assert!(size_of::<RelaxedRefMut<'static, u8>>() == size_of::<&mut u8>());
        assert!(align_of::<RelaxedRefMut<'static, u8>>() == align_of::<&mut u8>());
        assert!(size_of::<RelaxedRefMut<'static, str>>() == size_of::<&mut str>());
        assert!(align_of::<RelaxedRefMut<'static, str>>() == align_of::<&mut str>());
        assert!(size_of::<Option<RelaxedRefMut<'static, u8>>>() == size_of::<&mut u8>());
        assert!(align_of::<Option<RelaxedRefMut<'static, u8>>>() == align_of::<&mut u8>());
    };

    const _NULL_POINTER_OPTIMIZATION: () = {
        assert!(
            size_of::<Option<RelaxedRefMut<'static, u8>>>()
                == size_of::<RelaxedRefMut<'static, u8>>()
        );
        assert!(
            size_of::<Option<RelaxedRefMut<'static, str>>>()
                == size_of::<RelaxedRefMut<'static, str>>()
        );
    };
}

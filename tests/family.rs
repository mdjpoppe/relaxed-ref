#![allow(clippy::std_instead_of_core)]

#[allow(dead_code)]
mod family {
    use relaxed_ref::{RelaxedRef, RelaxedRefMut};
    use std::marker::PhantomData;

    trait Family {
        type Of<T>;
    }

    struct RefFamily<'a>(PhantomData<&'a ()>);

    impl<'a> Family for RefFamily<'a> {
        type Of<T> = RelaxedRef<'a, T>;
    }

    struct RefMutFamily<'a>(PhantomData<&'a ()>);

    impl<'a> Family for RefMutFamily<'a> {
        type Of<T> = RelaxedRefMut<'a, T>;
    }

    #[test]
    fn type_checks() {}
}

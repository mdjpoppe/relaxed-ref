use relaxed_ref::RelaxedRef;

fn into_ref_requires_outlives<'a, T>(r: RelaxedRef<'a, T>) {
    let _ = RelaxedRef::<'a, T>::into_ref(r);
}

fn main() {}

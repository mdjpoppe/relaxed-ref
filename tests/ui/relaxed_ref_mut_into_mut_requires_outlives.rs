use relaxed_ref::RelaxedRefMut;

fn into_mut_requires_outlives<'a, T>(r: RelaxedRefMut<'a, T>) {
    let _ = RelaxedRefMut::<'a, T>::into_mut(r);
}

fn main() {}

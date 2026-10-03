use relaxed_ref::RelaxedRefMut;

fn type_not_covariant<'short, 'mid: 'short, 'long: 'mid, T>(r: RelaxedRefMut<'short, &'long T>) {
    let _: RelaxedRefMut<'short, &'mid T> = r;
}

fn main() {}

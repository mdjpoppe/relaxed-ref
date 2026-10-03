use relaxed_ref::RelaxedRefMut;

fn type_not_contravariant<'short, 'mid: 'short, 'long: 'mid, T>(r: RelaxedRefMut<'short, &'mid T>) {
    let _: RelaxedRefMut<'short, &'long T> = r;
}

fn main() {}

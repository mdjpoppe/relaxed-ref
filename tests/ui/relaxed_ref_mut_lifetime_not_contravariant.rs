use relaxed_ref::RelaxedRefMut;

fn lifetime_not_contravariant<'short, 'mid: 'short, 'long: 'mid, T>(
    r: RelaxedRefMut<'short, &'long T>,
) {
    let _: RelaxedRefMut<'mid, &'long T> = r;
}

fn main() {}

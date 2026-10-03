use relaxed_ref::RelaxedRef;

fn lifetime_not_contravariant<'short, 'mid: 'short, 'long: 'mid, T>(
    r: RelaxedRef<'short, &'long T>,
) {
    let _: RelaxedRef<'mid, &'long T> = r;
}

fn main() {}

use relaxed_ref::RelaxedRef;

fn type_not_contravariant<'short, 'mid: 'short, 'long: 'mid, T>(r: RelaxedRef<'short, &'mid T>) {
    let _: RelaxedRef<'short, &'long T> = r;
}

fn main() {}

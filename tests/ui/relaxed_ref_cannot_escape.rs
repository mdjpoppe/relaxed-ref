use relaxed_ref::RelaxedRef;

fn cannot_escape<'a>() -> RelaxedRef<'a, i32> {
    let i = 0;
    RelaxedRef::new(&i)
}

fn main() {}

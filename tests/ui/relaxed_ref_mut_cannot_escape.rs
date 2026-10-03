use relaxed_ref::RelaxedRefMut;

fn cannot_escape<'a>() -> RelaxedRefMut<'a, i32> {
    let mut i = 0;
    RelaxedRefMut::new(&mut i)
}

fn main() {}

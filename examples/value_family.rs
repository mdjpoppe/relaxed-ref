//! This example shows how `RelaxedRef<'a, T>` can be used in the family pattern
//! as an alternative to `&'a T`.

#![allow(clippy::std_instead_of_core)]

use std::{marker::PhantomData, ops::Deref};

use relaxed_ref::RelaxedRef;

/// Represents having access to a value regardless of how that value is stored.
trait ValueFamily {
    type Of<T>: Deref<Target = T>;
}

struct BoxFamily;
impl ValueFamily for BoxFamily {
    type Of<T> = Box<T>;
}

struct RefFamily<'a>(PhantomData<&'a ()>);
impl<'a> ValueFamily for RefFamily<'a> {
    type Of<T> = RelaxedRef<'a, T>;
}

/// Represents one of `i8`, `i16`, `i32`, and `i64`.
///
/// The enum itself only describes the "shape" of the integer, while `V`
/// determines the underlying storage (through `V::Of<_>`).
enum Int<V: ValueFamily> {
    I8(V::Of<i8>),
    I16(V::Of<i16>),
    I32(V::Of<i32>),
    I64(V::Of<i64>),
}

/// Prints the type and value of `int`.
fn print_int<V: ValueFamily>(int: &Int<V>) {
    match int {
        Int::I8(x) => println!("I8: {x}", x = **x),
        Int::I16(x) => println!("I16: {x}", x = **x),
        Int::I32(x) => println!("I32: {x}", x = **x),
        Int::I64(x) => println!("I64: {x}", x = **x),
    }
}

fn main() {
    let boxes: Vec<Int<BoxFamily>> = vec![
        Int::I8(Box::new(1)),
        Int::I16(Box::new(2)),
        Int::I32(Box::new(3)),
        Int::I64(Box::new(4)),
    ];
    println!("Boxes:");
    boxes.iter().for_each(print_int);

    let refs: Vec<Int<RefFamily<'_>>> = vec![
        Int::I8(RelaxedRef::new(&1)),
        Int::I16(RelaxedRef::new(&2)),
        Int::I32(RelaxedRef::new(&3)),
        Int::I64(RelaxedRef::new(&4)),
    ];
    println!("Refs:");
    refs.iter().for_each(print_int);
}

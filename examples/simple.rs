use serde_tuple::*;

use std::borrow::Cow;

#[derive(Debug, Serialize_tuple, Deserialize_tuple)]
pub struct Foo<'a> {
    bar: &'a str,
    baz: i32
}

// #[derive(Debug, Serialize_tuple, Deserialize_tuple)]
// pub struct Foo {
// }

#[derive(Debug, serde_derive::Serialize, serde_derive::Deserialize)]
pub struct Bar {
    count: i32
}

fn main() {
    let foo = Foo { bar: "Yes".into(), baz: 22 };

    // let json = serde_json::to_string_pretty(&foo).unwrap();

    // println!("{}", &json);

    // let foo = serde_json::from_str::<Foo<Bar>>(&json).unwrap();

    // let foo = Foo;

    println!("{:?}", &foo);
}

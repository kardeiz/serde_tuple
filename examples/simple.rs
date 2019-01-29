use serde_tuple::*;

use std::borrow::Cow;

#[derive(Debug, SerializeTuple, DeserializeMaybeTuple)]
pub struct Foo {
    #[serde(rename = "b-a-r")]
    #[serde_tuple(position = 1)]
    bar: Cow<'static, str>,
    #[serde_tuple(position = 0)]
    baz: i32
}

#[derive(serde_derive::Serialize)]
pub struct Bar {
    foo: Foo
}

fn main() {
    let foo = Foo { bar: "Yes".into(), baz: 22 };

    let json = serde_json::to_string_pretty(&foo).unwrap();

    println!("{}", &json);

    let foo = serde_json::from_str::<Foo>(&json).unwrap();

    println!("{:?}", &foo);

    let foo = serde_json::from_str::<Foo>("{\"b-a-r\": \"Yes\", \"baz\": 22 }").unwrap();

    println!("{:?}", &foo);
}

# `serde_tuple`

[![GitHub](https://img.shields.io/badge/github-kardeiz/serde_tuple-8da0cb?logo=github)](https://github.com/kardeiz/serde_tuple)
[![crates.io version](https://img.shields.io/crates/v/serde_tuple)](https://crates.io/crates/serde_tuple)
[![docs.rs](https://img.shields.io/docsrs/serde_tuple)](https://docs.rs/serde_tuple)
[![crates.io license](https://img.shields.io/crates/l/serde_tuple)](https://github.com/kardeiz/serde_tuple/blob/main/LICENSE)
[![CI build](https://github.com/kardeiz/serde_tuple/actions/workflows/ci.yml/badge.svg)](https://github.com/kardeiz/serde_tuple/actions)

Serialize and deserialize structs with named fields as an array of values using `Derive` macros.

# Usage

```rust
use serde_tuple::*;

#[derive(Serialize_tuple, Deserialize_tuple)]
pub struct Foo<'a> {
    bar: &'a str,
    baz: i32
}

fn main() {
    let foo = Foo { bar: "Yes", baz: 22 };
    let json = serde_json::to_string(&foo).unwrap();
    println!("{json}");
    // # => ["Yes",22]
}
```

See also [this issue](https://github.com/dtolnay/request-for-implementation/issues/3)

## Development

* This project is easier to develop with [just](https://github.com/casey/just#readme), a modern alternative to `make`.
  Install it with `cargo install just`.
* To get a list of available commands, run `just`.
* To run tests, use `just test`.

## License

Licensed under MIT license ([LICENSE](./LICENSE) or <https://opensource.org/licenses/MIT>)
  at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in the work by you, as defined in the
Apache-2.0 license, shall be dual-licensed as above, without any
additional terms or conditions.

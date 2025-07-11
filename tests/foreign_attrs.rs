// This tests that foreign attributes can be used with `serde_tuple`. Historically, deriving
// `Serialize_tuple` or `Deserialize_tuple` would cause compilation errors if the struct had
// foreign attributes, such as `#[derivative(Debug)]` from the `derivative` crate. This applied for
// both the entire struct and individual fields.
use derivative::Derivative;
use serde_tuple::Serialize_tuple;

#[derive(Derivative, Serialize_tuple)]
#[derivative(Debug)]
pub struct OwnStruct {
    #[derivative(Debug = "ignore")]
    value1: String,
    value2: u64,
}

#[test]
fn test_interop_attrs() {
    let own_struct = OwnStruct {
        value1: "Cthulhu".to_string(),
        value2: 42,
    };
    // This will not print `value1` due to the `Debug = "ignore"` attribute
    let debug_output = format!("{own_struct:?}");
    assert_eq!(debug_output, "OwnStruct { value2: 42 }");

    // serialize to json - should still have everything in tuple
    let json_output = serde_json::to_string(&own_struct).unwrap();
    assert_eq!(json_output, r#"["Cthulhu",42]"#);
}

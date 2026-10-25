use serde_json::Value;
use tezbar_json2ts::{convert, MAX_INPUT_BYTES, MAX_NAME_BYTES};
#[test]
fn actual_source_oracle_bytes() {
    let cases: Vec<Value> = serde_json::from_str(include_str!("oracle-cases.json")).unwrap();
    assert!(cases.len() >= 1500);
    for (i, c) in cases.iter().enumerate() {
        let result = convert(
            c["input"].as_str().unwrap(),
            c["rootName"].as_str().unwrap(),
        );
        if c["expected"]["ok"] == true {
            assert_eq!(
                result.as_ref().unwrap(),
                c["expected"]["output"].as_str().unwrap(),
                "case {i}: {c}"
            );
        } else {
            assert!(result.is_err(), "case {i}: {c}");
        }
    }
}
#[test]
fn explicit_limits_and_private_errors() {
    assert!(convert(&" ".repeat(MAX_INPUT_BYTES + 1), "r")
        .unwrap_err()
        .contains("limit"));
    assert!(convert("{}", &"r".repeat(MAX_NAME_BYTES + 1))
        .unwrap_err()
        .contains("limit"));
    assert!(
        convert(&format!("{}0{}", "[".repeat(70), "]".repeat(70)), "r")
            .unwrap_err()
            .contains("nesting")
    );
    assert!(convert(&format!("[{}]", vec!["0"; 20_001].join(",")), "r")
        .unwrap_err()
        .contains("limit"));
    let error = convert("private_secret", "r").unwrap_err();
    assert!(!error.contains("private_secret"));
}
#[test]
fn duplicate_properties_and_deliberate_original_quirks() {
    assert_eq!(
        convert("{\"a\":1,\"a\":false}", "r").unwrap(),
        "interface R {\n    a: number;\n    a: boolean;\n}\n"
    );
    assert_eq!(
        convert("[true,false,1]", "r").unwrap(),
        "type r = (boolean | boolean | number)[];\n"
    );
    assert_eq!(
        convert("{\"x\":[{\"a\":[]},{\"a\":3}]}", "r").unwrap(),
        "interface R {\n    x: XItem[];\n}\ninterface XItem {\n    a: [];\n}\n"
    );
}
#[test]
fn malformed_recovery_and_unicode_boundary_are_explicit() {
    assert_eq!(
        convert("{a:1 b:2}", "r").unwrap(),
        "interface R {\n    a: number;\n    b: number;\n}\n"
    );
    assert_eq!(convert("[1,", "r").unwrap(), "type r = number[];\n");
    assert!(convert("{\"\\uD800\":1}", "r")
        .unwrap_err()
        .contains("Unpaired"));
    assert_eq!(
        convert("{\"a\":\"\\ud800\"}", "r").unwrap(),
        "interface R {\n    a: string;\n}\n"
    );
}

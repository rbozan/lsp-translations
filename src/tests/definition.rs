use super::*;

#[test]
fn converts_en_to_us_flag() {
    let definition = Definition {
        key: "en.test_key".to_string(),
        cleaned_key: Some("test_key".to_string()),
        file: "somefile.json".to_string(),
        language: Some("en".to_string()),
        value: "some value".to_string(),
    };

    assert_eq!(definition.get_flag(), Some("🇺🇸".to_string()));
}

#[test]
fn printable_value_without_newlines() {
    let definition = Definition {
        key: "en.test_key".to_string(),
        cleaned_key: Some("test_key".to_string()),
        file: "somefile.json".to_string(),
        language: Some("en".to_string()),
        value: "\nSome value with multiple\nnewlines".to_string(),
    };

    assert_eq!(definition.get_printable_value(), "\\nSome value with multiple\\nnewlines".to_string());
}


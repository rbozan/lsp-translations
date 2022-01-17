use super::*;

#[test]
fn converts_en_to_us_flag() {
    let mut extra_data = HashMap::<String, String>::new();
    extra_data.insert("language".to_string(), "en".to_string());

    let definition = Definition {
        key: "en.test_key".to_string(),
        cleaned_key: Some("test_key".to_string()),
        extra_data,
        value: "some value".to_string(),
        ..Default::default()
    };

    assert_eq!(definition.get_flag(), Some("🇺🇸".to_string()));
}

#[test]
fn printable_value_escapes_newlines() {
    let mut extra_data = HashMap::<String, String>::new();
    extra_data.insert("language".to_string(), "en".to_string());

    let definition = Definition {
        key: "en.test_key".to_string(),
        cleaned_key: Some("test_key".to_string()),
        extra_data,
        value: "\nSome value with multiple\nnewlines".to_string(),
        ..Default::default()
    };

    assert_eq!(
        definition.get_printable_value(),
        "\\nSome value with multiple\\nnewlines"
    );
}

#[test]
fn printable_value_escapes_vertical_line() {
    let mut extra_data = HashMap::<String, String>::new();
    extra_data.insert("language".to_string(), "en".to_string());

    let definition = Definition {
        key: "en.test_key".to_string(),
        cleaned_key: Some("test_key".to_string()),
        extra_data,
        value: "Abc|defg".to_string(),
        ..Default::default()
    };

    assert_eq!(definition.get_printable_value(), "Abc\\|defg");
}

#[test]
fn printable_value_doesnt_escape_arabic() {
    let definition = Definition {
        key: "test_key".to_string(),
        value: "مهلا".to_string(),
        ..Default::default()
    };

    assert_eq!(definition.get_printable_value(), "مهلا");
}

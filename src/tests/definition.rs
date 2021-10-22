use super::*;

#[test]
fn finds_simple_translation_key() {
    let definition = Definition {
        key: "en.test_key".to_string(),
        cleaned_key: Some("test_key".to_string()),
        file: "somefile.json".to_string(),
        language: Some("en".to_string()),
        value: "blabla".to_string(),
    };

    assert_eq!(definition.get_flag(), Some("🇺🇸".to_string()));
}

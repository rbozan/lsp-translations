use super::*;

#[test]
fn finds_simple_translation_key() {
    assert_eq!(
        find_translation_key_by_position(
            &r#"
        function test() {
            translate('some-key');
        }
        "#
            .to_string(),
            &55
        ),
        Some("some-key".to_string())
    );
}

#[test]
fn finds_translation_key_in_inline_function() {
    assert_eq!(
        find_translation_key_by_position(
            &r#"
        function test() {
            test(translate('some-key'));
        }
        "#
            .to_string(),
            &59
        ),
        Some("some-key".to_string())
    );
}

#[test]
fn finds_translation_key_on_correct_position() {
    assert_eq!(
        find_translation_key_by_position(
            &r#"
        function test() {
            translate('first-key');
            translate('second-key');
        }
        "#
            .to_string(),
            &90
        ),
        Some("second-key".to_string())
    );
}

#[test]
fn finds_nothing_when_out_of_range() {
    assert_eq!(
        find_translation_key_by_position(
            &r#"
        function test() {
            translate('some-key');
        }
        "#
            .to_string(),
            &20
        ),
        None
    );
}

#[test]
fn finds_no_translation_key() {
    assert_eq!(
        find_translation_key_by_position(
            &r#"
        function test() {
            translate2('some-key');
        }
        "#
            .to_string(),
            &55
        ),
        None
    );
}

use crate::string_helper::is_editing_position;

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
        )
        .unwrap()
        .as_str(),
        "some-key"
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
        )
        .unwrap()
        .as_str(),
        "some-key"
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
        )
        .unwrap()
        .as_str(),
        "second-key"
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

#[test]
fn is_editing_position_works_in_middle() {
    assert_eq!(
        is_editing_position(
            &r#"
        function test() {
            translate('some-key');
        }
        "#
            .to_string(),
            &55
        ),
        true
    );
}

#[test]
fn is_editing_position_works_at_start() {
    assert_eq!(
        is_editing_position(
            &r#"
        function test() {
            translate('
                "#
            .to_string(),
            &50
        ),
        true
    );
}

#[test]
fn is_editing_position_works_for_empty_key() {
    assert_eq!(
        is_editing_position(
            &r#"
        function test() {
            translate('')
                "#
            .to_string(),
            &50
        ),
        true
    );
}

#[test]
fn is_editing_position_returns_false_for_other_functions() {
    assert_eq!(
        is_editing_position(
            &r#"
        function test() {
            some_func('')
                "#
            .to_string(),
            &50
        ),
        false
    );
}

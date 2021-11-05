use crate::string_helper::is_editing_position;
use lsp_document::IndexedText;

use super::*;

#[test]
fn finds_simple_translation_key() {
    assert_eq!(
        find_translation_key_by_position(
            &IndexedText::new(
                r#"
        function test() {
            translate('some-key');
        }
        "#
                .to_string()
            ),
            &Pos { line: 2, col: 24 }
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
            &IndexedText::new(
                r#"
        function test() {
            test(translate('some-key'));
        }
        "#
                .to_string()
            ),
            &Pos { line: 2, col: 28 }
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
            &IndexedText::new(
                r#"
        function test() {
            translate('first-key');
            translate('second-key');
        }
        "#
                .to_string()
            ),
            &Pos { line: 3, col: 28 }
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
            &IndexedText::new(
                r#"
        function test() {
            translate('some-key');
        }
        "#
                .to_string()
            ),
            &Pos { line: 2, col: 10 }
        ),
        None
    );
}

#[test]
fn finds_no_translation_key() {
    assert_eq!(
        find_translation_key_by_position(
            &IndexedText::new(
                r#"
        function test() {
            translate2('some-key');
        }
        "#
                .to_string()
            ),
            &Pos { line: 2, col: 24 }
        ),
        None
    );
}

#[test]
fn is_editing_position_works_in_middle() {
    assert_eq!(
        is_editing_position(
            &IndexedText::new(
                r#"
        function test() {
            translate('some-key');
        }
        "#
                .to_string()
            ),
            &Pos { line: 2, col: 27 }
        ),
        true
    );
}

#[test]
fn is_editing_position_works_at_start() {
    assert_eq!(
        is_editing_position(
            &IndexedText::new(
                r#"
        function test() {
            translate('
                "#
                .to_string()
            ),
            &Pos { line: 2, col: 23 }
        ),
        true
    );
}

#[test]
fn is_editing_position_works_for_empty_key() {
    assert_eq!(
        is_editing_position(
            &IndexedText::new(
                r#"
        function test() {
            translate('')
                "#
                .to_string()
            ),
            &Pos { line: 2, col: 23 }
        ),
        true
    );
}

#[test]
fn is_editing_position_returns_false_for_other_functions() {
    assert_eq!(
        is_editing_position(
            &IndexedText::new(
                r#"
        function test() {
            some_func('')
                "#
                .to_string()
            ),
            &Pos { line: 2, col: 23 }
        ),
        false
    );
}

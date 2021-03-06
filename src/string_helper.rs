// TODO: In the next version this should be replaced with an AST like Treesitter or Babel.

use std::ops::Range;

use itertools::Itertools;
use lsp_document::{IndexedText, Pos, TextMap};
use regex::Regex;

pub static TRANSLATION_BEGIN_CHARS: &[&str] = &["'", "\"", "`"];
pub static TRANSLATION_KEY_DIVIDER: &str = ".";

// TODO: Use TRANSLATION_BEGIN_CHARS in combination with keywords below.
static TRANSLATION_BEGIN: &[&str] = &[
    "translate('",
    "translate(\"",
    "translate(`",
    " t('",
    " t(\"",
    " t(`",
    "(t('",
    "(t(\"",
    "(t(`",
    "{t('",
    "{t(\"",
    "{t(`",
    "I18n.t('",
    "I18n.t(\"",
];

static TRANSLATION_END: &[&str] = &["'", "\"", "`"];

lazy_static! {
    static ref TRANSLATION_BEGIN_GROUP: String = format!(
        "(?:{})",
        TRANSLATION_BEGIN
            .iter()
            .map(|key| regex::escape(key))
            .join("|")
    );
    static ref TRANSLATION_END_GROUP: String = format!(
        "(?:{})",
        TRANSLATION_END
            .iter()
            .map(|key| regex::escape(key))
            .join("|")
    );
    static ref TRANSLATION_REGEX: Regex = Regex::new(
        format!(
            "{}(.+?){}",
            *TRANSLATION_BEGIN_GROUP, *TRANSLATION_END_GROUP
        )
        .as_str()
    )
    .unwrap();
    static ref TRANSLATION_EDITING_REGEX: Regex = Regex::new(
        format!(
            "{}(.*?){}",
            *TRANSLATION_BEGIN_GROUP,
            format!(
                "(?m:{}|$)",
                TRANSLATION_END
                    .iter()
                    .map(|key| regex::escape(key))
                    .join("|")
            )
        )
        .as_str()
    )
    .unwrap();
}

pub fn find_translation_key_by_position<'a>(
    indexed_text: &'a IndexedText<std::string::String>,
    pos: &Pos,
) -> Option<regex::Match<'a>> {
    for groups in TRANSLATION_REGEX.captures_iter(indexed_text.text()) {
        let result = groups.get(1).unwrap();
        let range = indexed_text.offset_range_to_range(result.range()).unwrap();
        if range.contains(pos) {
            return Some(result);
        }
    }
    None
}

pub fn get_editing_range(indexed_text: &IndexedText<String>, pos: &Pos) -> Option<Range<Pos>> {
    for groups in TRANSLATION_EDITING_REGEX.captures_iter(indexed_text.text()) {
        let result = groups.get(1).unwrap();
        let range = indexed_text.offset_range_to_range(result.range()).unwrap();

        if range.contains(pos) || &range.start == pos || &range.end == pos {
            return Some(range);
        }
    }
    None
}

pub fn is_editing_position(indexed_text: &IndexedText<String>, pos: &Pos) -> bool {
    get_editing_range(indexed_text, pos).is_some()
}

#[path = "./tests/string_helper.rs"]
#[cfg(test)]
mod test;

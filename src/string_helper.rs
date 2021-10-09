// TODO: In the next version this should be replaced with an AST like Treesitter or Babel.

use itertools::Itertools;
use regex::Regex;

static TRANSLATION_BEGIN: &[&str] = &[
    "translate('",
    "translate(\"",
    "translate(`",
    " t('",
    " t(\"",
    "(t('",
    "(t(\"",
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
}

fn find_translation_key_by_position(text: &String, pos: &usize) -> Option<String> {
    TRANSLATION_REGEX.captures(text).and_then(|groups| {
        eprintln!("Found match: {:?}", groups);
        let result = groups.get(1).unwrap();
        eprintln!("result: {:?}", result);
        if result.range().contains(pos) {
            Some(result.as_str().to_string())
        } else {
            None
        }
    })
}

#[path = "./tests/string_helper.rs"]
#[cfg(test)]
mod test;

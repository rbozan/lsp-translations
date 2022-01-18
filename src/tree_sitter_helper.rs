use std::ops::Range;
use std::collections::HashMap;

use crate::{Definition, ExtensionConfig};
use tree_sitter::{Language, Node, Parser, Query, QueryCursor, QueryMatches};

extern "C" {
    fn tree_sitter_json() -> Language;
    fn tree_sitter_yaml() -> Language;
    fn tree_sitter_php() -> Language;
}

pub fn get_language_by_extension(ext: &str) -> Option<Language> {
    match ext {
        "json" => Some(unsafe { tree_sitter_json() }),
        "yaml" | "yml" => Some(unsafe { tree_sitter_yaml() }),
        "php" => Some(unsafe { tree_sitter_php() }),
        _ => None,
    }
}

pub fn get_query_source_by_language(ext: &str) -> Option<&str> {
    match ext {
        "json" => Some(include_str!("./queries/json.scm")),
        "yaml" | "yml" => Some(include_str!("./queries/yaml.scm")),
        "php" => Some(include_str!("./queries/php.scm")),
        _ => None,
    }
}

pub fn parse_translation_structure(
    text: String,
    config: &ExtensionConfig,
    language: Language,
    query_source: &str,
) -> Option<Vec<Definition>> {
    let mut parser = Parser::new();

    parser.set_language(language).unwrap();

    let tree = parser.parse(&text, None).unwrap();

    let query = Query::new(language, query_source).unwrap();

    let mut query_cursor = QueryCursor::new();

    // Execute matches
    let mut definitions = vec![];

    let matches = query_cursor.matches(&query, tree.root_node(), text.as_bytes());
    for m in matches {
        let mut translation_key = "";
        let mut translation_value: Option<Node> = None;
        let mut translation_group: Option<Node> = None;

        for capture in m.captures {
            let capture_name = &query.capture_names()[capture.index as usize];

            if capture_name == "translation_key" {
                translation_key = &text[capture.node.byte_range()];
            } else if capture_name == "translation_value" {
                translation_value = Some(capture.node);
            } else if capture_name == "translation_group" {
                translation_group = Some(capture.node);
            } else if capture_name == "translation_error" {
                eprintln!("Found an error in the translation file");
                return None;
            }

            if !translation_key.is_empty()
                && translation_value.is_some()
                && translation_group.is_some()
            {
                let group_path = get_path_for_node(translation_value.unwrap(), &text);

                let path = if !group_path.is_empty() {
                    group_path
                } else {
                    translation_key
                        .trim_matches('\'')
                        .trim_matches('"')
                        .to_string()
                };

                let translation_value_string = text[translation_value.unwrap().byte_range()]
                    .trim_matches('\'')
                    .trim_matches('"')
                    .to_string();

                definitions.push(Definition {
                    key: path.clone(),
                    cleaned_key: get_cleaned_key_for_path(&path, config),
                    file: None,
                    extra_data: get_extra_data_for_path(&path, config),
                    value: translation_value_string,
                });

                translation_group = None;
                translation_key = "";
                translation_value = None;
            }
        }
    }

    Some(definitions)
}

/// Recursively goes through a node to fetch the path
fn get_path_for_node(initial_node: Node, text: &String) -> String {
    let mut cursor = initial_node.walk();
    let mut path = String::new();

    loop {
        let node = cursor.node();
        if node.kind() == "pair" || node.kind() == "block_mapping_pair" {
            let key = node.child_by_field_name("key").unwrap();

            let key_string_node = get_string_content_from_string(key).unwrap();

            let range = match key_string_node.kind() {
                "single_quote_scalar" | "double_quote_scalar" => {
                    let original_range = key_string_node.byte_range();
                    Range {
                        start: original_range.start + 1,
                        end: original_range.end - 1,
                    }
                }
                _ => key_string_node.byte_range(),
            };

            path = format!(".{}{}", &text[range], &path,);
        } else if node.kind() == "block_sequence_item" {
            let index = get_array_index_of_node(node).unwrap();

            path = format!("[{}]{}", index, &path);
        }

        match node.parent() {
            Some(parent_node) => cursor.reset(parent_node),
            None => break,
        }
    }

    // As the dot is always being added, it should be removed for the last match
    if path.starts_with('.') {
        path = path[1..].to_string();
    }
    path
}

static STRING_CONTENT_KINDS: &[&str] = &[
    // JSON
    "string_content",
    // YAML
    "string_scalar",
    "single_quote_scalar",
    "double_quote_scalar",
];

fn get_array_index_of_node(node: Node) -> Option<usize> {
    let parent_node = node.parent()?;

    let mut cursor = parent_node.walk();
    for (index, child_node) in parent_node.children(&mut cursor).enumerate() {
        if child_node == node {
            return Some(index);
        }
    }
    None
}

fn get_string_content_from_string(string: Node) -> Option<Node> {
    let mut value_cursor = string.walk();
    value_cursor.goto_first_child();

    loop {
        let node = value_cursor.node();
        if STRING_CONTENT_KINDS.contains(&node.kind()) {
            return Some(node);
        }

        if node.kind() == "plain_scalar" {
            let result = get_string_content_from_string(node);
            if result.is_some() {
                return result;
            };
        }

        if !value_cursor.goto_next_sibling() {
            break;
        }
    }

    None
}

fn get_cleaned_key_for_path(path: &String, config: &ExtensionConfig) -> Option<String> {
    config.key.filter.as_ref().and_then(|key_filter_regex| {
        key_filter_regex
            .captures(&path.replace("\n", ""))
            .and_then(|cap| cap.get(1).map(|group| group.as_str().to_string()))
    })
}

fn get_extra_data_for_path(
    path: &String,
    config: &ExtensionConfig,
) -> HashMap<String, String> {
    let mut extra_data = HashMap::<String, String>::new();

    if let Some(key_details_regex) = config.key.details.as_ref() {
        if let Some(cap) = key_details_regex.captures(&path) {
            for capture_group_name in key_details_regex.capture_names().flatten() {
                let capture_group_result = cap.name(capture_group_name);

                if capture_group_result.is_some() {
                    extra_data.insert(
                        capture_group_name.to_string(),
                        capture_group_result.unwrap().as_str().to_string(),
                    );
                }
            }
        }
    };

    extra_data
}

fn get_language_for_path(path: &String, config: &ExtensionConfig) -> Option<String> {
    config.key.details.as_ref().and_then(|key_details_regex| {
        key_details_regex.captures(path).and_then(|cap| {
            cap.name("language")
                .map(|matches| matches.as_str().to_string())
        })
    })
}

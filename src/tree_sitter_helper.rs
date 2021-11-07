use crate::{Definition, ExtensionConfig};
use tree_sitter::{Language, Node, Parser, Query, QueryCursor};

extern "C" {
    fn tree_sitter_json() -> Language;
    fn tree_sitter_yaml() -> Language;
}

pub fn get_language_by_extension(ext: &str) -> Option<Language> {
    match ext {
        "json" => Some(unsafe { tree_sitter_json() }),
        "yaml" | "yml" => Some(unsafe { tree_sitter_yaml() }),
        _ => None,
    }
}

pub fn get_query_source_by_language(ext: &str) -> Option<&str> {
    match ext {
        "json" => Some(include_str!("./queries/json.scm")),
        "yaml" | "yml" => Some("./queries/yaml.scm"),
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

    let query = Query::new(language, &query_source).unwrap();

    dbg!(query.capture_names());

    let mut query_cursor = QueryCursor::new();

    // Execute matches
    let mut definitions = vec![];

    for m in query_cursor.matches(&query, tree.root_node(), text.as_bytes()) {
        dbg!(&m);

        for capture in m.captures {
            dbg!(&capture);

            let capture_name = &query.capture_names()[capture.index as usize];
            dbg!(capture_name);

            if (capture_name == "translation_value") {
                let path = get_path_for_node(capture.node, &text);

                definitions.push(Definition {
                    key: path.clone(),
                    cleaned_key: get_cleaned_key_for_path(&path, config),
                    file: None,
                    language: get_language_for_path(&path, config),
                    value: text[capture.node.byte_range()].to_string(),
                })
            }
        }

        println!("---------")
    }

    Some(definitions)
}

fn get_path_for_node(initial_node: Node, text: &String) -> String {
    let mut cursor = initial_node.walk();
    let mut path = String::new();

    println!("get path for node");

    loop {
        let node = cursor.node();
        if (node.kind() == "pair") {
            println!("found a pair!");

            let key = node.child_by_field_name("key").unwrap();
            println!("key = {:#?}", key);

            let key_string_node = get_string_content_from_string(key).unwrap();

            path = format!(
                "{}{}{}",
                text[key_string_node.byte_range()].to_string(),
                if !path.is_empty() { "." } else { "" },
                &path,
            );
        }

        println!("parent node {:?}", node);

        match node.parent() {
            Some(parent_node) => cursor.reset(parent_node),
            None => break,
        }
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

fn get_string_content_from_string(string: Node) -> Option<Node> {
    let mut value_cursor = string.walk();
    value_cursor.goto_first_child();

    loop {
        if STRING_CONTENT_KINDS.contains(&value_cursor.node().kind()) {
            return Some(value_cursor.node());
        }

        println!("Is het child? {:?}", value_cursor.node());

        if (value_cursor.node().kind() == "plain_scalar") {
            let result = get_string_content_from_string(value_cursor.node());
            if (result.is_some()) {
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

fn get_language_for_path(path: &String, config: &ExtensionConfig) -> Option<String> {
    config.key.details.as_ref().and_then(|key_details_regex| {
        key_details_regex.captures(path).and_then(|cap| {
            cap.name("language")
                .map(|matches| matches.as_str().to_string())
        })
    })
}

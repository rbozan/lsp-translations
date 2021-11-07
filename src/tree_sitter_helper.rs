use crate::{Definition, ExtensionConfig};
use tree_sitter::{Language, Node, Parser};

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

pub fn parse_translation_structure(
    text: String,
    config: &ExtensionConfig,
    language: Language,
) -> Option<Vec<Definition>> {
    let mut parser = Parser::new();

    parser.set_language(language).unwrap();

    let tree = parser.parse(&text, None).unwrap();

    let definitions = parse_tree(&text, tree.root_node(), true, "".to_string(), config);
    println!("Definitions found: {:#?}", definitions);
    definitions
}

pub fn parse_tree(
    text: &String,
    node: Node,
    is_root: bool,
    path: String,
    config: &ExtensionConfig,
) -> Option<Vec<Definition>> {
    let mut definitions = vec![];

    let mut cursor = node.walk();

    if !is_root && (!cursor.goto_first_child()) {
        return Some(definitions);
    }
    loop {
        let node = cursor.node();
        let range = node.byte_range();
        let value = &text[range];
        let mut new_path = path.clone();
        let mut new_node = node;

        println!(
            "cursor node ${:?}, kind {:?}, type {:?}, utf8text {:?}",
            node,
            node.kind(),
            cursor.field_name(),
            value
        );

        if node.kind() == "pair" || node.kind() == "block_mapping_pair" {
            let key = node.child_by_field_name("key")?;
            println!("key = {:#?}", key);

            let key_string_node = get_string_content_from_string(key)?;

            let value = node.child_by_field_name("value")?;
            println!("value = {:#?}", value);
            new_node = value;

            match value.kind() {
                "string" => {
                    new_path = format!(
                        "{}{}{}",
                        &path,
                        if !path.is_empty() { "." } else { "" },
                        text[key_string_node.byte_range()].to_string()
                    );

                    definitions.push(Definition {
                        key: new_path.clone(),
                        cleaned_key: get_cleaned_key_for_path(&new_path, config),
                        file: None, // TODO: Fix this
                        language: get_language_for_path(&new_path, config),
                        value: text[get_string_content_from_string(value)?.byte_range()]
                            .to_string(),
                    });
                }
                "object" => {
                    new_path = format!(
                        "{}{}{}",
                        &path,
                        if !path.is_empty() { "." } else { "" },
                        text[key_string_node.byte_range()].to_string()
                    );
                }
                "array" => {
                    new_path = format!(
                        "{}{}{}",
                        &path,
                        if !path.is_empty() { "." } else { "" },
                        text[key_string_node.byte_range()].to_string()
                    );

                    definitions.append(&mut get_definitions_in_array(
                        value, text, &new_path, config,
                    )?);
                }
                _ => {
                    println!("Unknown type received");
                    return None;
                }
            }
        }

        println!("---");

        let mut child_definitions = parse_tree(text, new_node, false, new_path, config)?;

        definitions.append(&mut child_definitions);

        if !cursor.goto_next_sibling() {
            break;
        }
    }

    Some(definitions)
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

fn get_definitions_in_array(
    array_node: Node,
    text: &String,
    path: &String,
    config: &ExtensionConfig,
) -> Option<Vec<Definition>> {
    let mut definitions = vec![];

    let mut array_cursor = array_node.walk();
    if !array_cursor.goto_first_child() {
        return Some(definitions);
    }

    let mut i = 0;

    loop {
        let node = array_cursor.node();
        let mut new_path = path.clone();

        new_path = format!("{}[{}]", path, i);

        if node.kind() == "string" {
            definitions.push(Definition {
                key: new_path.clone(),
                cleaned_key: get_cleaned_key_for_path(&new_path, config),
                file: None, // TODO: Fix this
                language: get_language_for_path(&new_path, config),
                value: text[get_string_content_from_string(node)?.byte_range()].to_string(),
            });
        }

        if !array_cursor.goto_next_sibling() {
            break;
        }

        if node.kind() != "[" {
            i += 1;
        }
    }

    Some(definitions)
}

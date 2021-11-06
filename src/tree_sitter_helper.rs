use std::any::Any;

use crate::Definition;
use tree_sitter::{Language, Node, Parser};

extern "C" {
    fn tree_sitter_json() -> Language;
}

pub fn parse_translation_structure(text: String) -> Vec<Definition> {
    let mut parser = Parser::new();

    let language = unsafe { tree_sitter_json() };
    parser.set_language(language).unwrap();

    let tree = parser.parse(&text, None).unwrap();

    let definitions = parse_tree(&text, tree.root_node(), true, "".to_string());
    println!("Definitions found: {:#?}", definitions);
    definitions.unwrap()
}

pub fn parse_tree(
    text: &String,
    node: Node,
    is_root: bool,
    path: String,
) -> Option<Vec<Definition>> {
    let mut definitions = vec![];

    let mut cursor = node.walk();

    if !is_root {
        if (!cursor.goto_first_child()) {
            return Some(definitions);
        }
    }
    loop {
        let node = cursor.node();
        let range = node.byte_range();
        let value = &text[range];
        let mut new_path = path.clone();

        println!(
            "cursor node ${:?}, kind {:?}, type {:?}, utf8text {:?}",
            node,
            node.kind(),
            cursor.field_name(),
            value
        );
        if node.kind() == "pair" {
            let key = node.child_by_field_name("key")?;
            println!("key = {:#?}", key);

            let key_string_node = get_string_content_from_string(key)?;

            let value = node.child_by_field_name("value")?;
            println!("value = {:#?}", value);
            if (value.kind() == "string") {
                definitions.push(Definition {
                    key: format!(
                        "{}{}{}",
                        &path,
                        if !path.is_empty() { "." } else { "" },
                        text[key_string_node.byte_range()].to_string()
                    ),
                    cleaned_key: None, // TODO: Fix this
                    file: None, // TODO: Fix this
                    language: None, // TODO: Fix this
                    value: text[get_string_content_from_string(value)?.byte_range()].to_string(),
                });
            } else if (value.kind() == "object") {
                new_path = format!(
                    "{}{}{}",
                    &path,
                    if !path.is_empty() { "." } else { "" },
                    text[key_string_node.byte_range()].to_string()
                );
            }
        }

        println!("---");

        let mut child_definitions = parse_tree(&text, node, false, new_path)?;

        definitions.append(&mut child_definitions);

        if (!cursor.goto_next_sibling()) {
            break;
        }
    }

    Some(definitions)
}

fn get_string_content_from_string(string: Node) -> Option<Node> {
    let mut value_cursor = string.walk();
    value_cursor.goto_first_child();

    loop {
        if value_cursor.node().kind() == "string_content" {
            return Some(value_cursor.node());
        }

        if !value_cursor.goto_next_sibling() {
            break;
        }
    }

    None
}

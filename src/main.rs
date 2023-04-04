use serde_derive::{Deserialize, Serialize};
use std::fs;
use tree_sitter::{Language, Parser, Query, QueryCursor};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Node {
    pub node_type: String,
    pub element_text: String,
}

impl Default for Node {
    fn default() -> Node {
        Node {
            node_type: String::from("none"),
            element_text: String::from(""),
        }
    }
}

fn process_input() -> Vec<Node> {
    let mut parser = Parser::new();
    let rust_lang: Language = tree_sitter_rust::language();
    parser.set_language(rust_lang).unwrap();

    let code = r#"unsafe {
    let dst = s.as_mut_ptr().add(range.start);
    let src = s.as_ptr().add(range.end);
    let count = range_end - range.end;
    std::ptr::copy(src, dst, count);
    println!("{}", s);
}"#;

    let tree = parser.parse(&code, None).unwrap();
    let query_string = fs::read_to_string("./src/language/rust/highlights.scm")
        .expect("Failed to read the highlight.scm file");

    let query = Query::new(rust_lang, &query_string).unwrap();

    let mut query_cursor = QueryCursor::new();
    let captures = query_cursor.captures(&query, tree.root_node(), code.as_bytes());

    let mut captured_elements: Vec<Node> = Vec::new();

    let mut last_capture_end_byte = 0;
    for (mat, ix) in captures {
        let mut node: Node = Node::default();
        let capture = &mat.captures[ix];
        let capture_name = &query.capture_names()[capture.index as usize];
        let start_byte = capture.node.start_byte();
        let end_byte = capture.node.end_byte();
        let element_text = &code[start_byte..end_byte];
        if start_byte > last_capture_end_byte {
            let text = &code[last_capture_end_byte..start_byte];
            node.element_text = String::from(text);
            captured_elements.push(node.clone());
        }
        last_capture_end_byte = end_byte;
        node.element_text = String::from(element_text);
        node.node_type = String::from(capture_name);
        captured_elements.push(node);
    }

    captured_elements
}

fn split_newline_nodes(nodes: Vec<Node>) -> Vec<Node> {
    let mut new_nodes: Vec<Node> = Vec::new();

    for node in nodes {
        if node.element_text.contains('\n') {
            let mut remaining_text = node.element_text.clone();
            let mut start = 0;

            while let Some(pos) = remaining_text.find('\n') {
                let end = start + pos;
                if start != end {
                    new_nodes.push(Node {
                        node_type: node.node_type.clone(),
                        element_text: node.element_text[start..end].to_string(),
                    });
                }

                new_nodes.push(Node {
                    node_type: String::from("newline"),
                    element_text: String::from("\n"),
                });

                remaining_text = remaining_text[pos + 1..].to_string();
                start = end + 1;
            }

            if !remaining_text.is_empty() {
                new_nodes.push(Node {
                    node_type: node.node_type,
                    element_text: remaining_text,
                });
            }
        } else {
            new_nodes.push(node);
        }
    }

    new_nodes
}

fn main() {
    let nodes = process_input();
    let split_nodes = split_newline_nodes(nodes);
    let nodes_json = serde_json::to_string(&split_nodes).unwrap();
    println!("{}", nodes_json);
}

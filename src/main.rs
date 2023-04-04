use serde_derive::{Deserialize, Serialize};
use std::fs;
use std::io::Error;
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

// TODO: When a node matches multiple patterns only keep the last one
// TODO: Look into injections to correctly highlight macro invocations, etc

/// Takes a string and returns a vector of Nodes.
///
/// Currently assumes the code is in rust. Later this will become a property.
///
/// Uses tree-sitter to parse the code, and a tree-sitter query slice
/// the code into captures and non-captures.
///
/// The captures are then converted into Nodes with a node_type which will
/// be used to determine the color of the text when highlighting.
///
/// The non-captures are converted into Nodes with the node_type "none",
/// which is used to indicate plain text.
fn process_input(code: &str) -> Vec<Node> {
    let mut parser = Parser::new();

    // Hard code rust as the only language for now
    let rust_lang: Language = tree_sitter_rust::language();
    parser.set_language(rust_lang).unwrap();

    let tree = parser.parse(&code, None).unwrap();

    // Load the query from /language/* (static until support for multiple languages is added)
    let query_string = fs::read_to_string("./src/language/rust/highlights.scm")
        .expect("Failed to read the highlight.scm file");

    let query = Query::new(rust_lang, &query_string).unwrap();

    let mut query_cursor = QueryCursor::new();

    // Get an array of captures from the query
    // We won't get the non-captured nodes from this
    let captures = query_cursor.captures(&query, tree.root_node(), code.as_bytes());

    let mut captured_elements: Vec<Node> = Vec::new();

    // The last capture's end byte so we can keep track of the
    // non-captured text between each capture
    let mut last_capture_end_byte = 0;

    for (mat, ix) in captures {
        let mut node: Node = Node::default();

        let capture = &mat.captures[ix];

        // The name of the capture, will be "node_type" in Node
        // For example: "function", "comment", "string", etc.
        let capture_name = &query.capture_names()[capture.index as usize];

        let start_byte = capture.node.start_byte();
        let end_byte = capture.node.end_byte();

        // The text captured by the query, will be "element_text" in Node
        let element_text = &code[start_byte..end_byte];

        // Get the text between captures and add it as a node with the node_type "none"
        if start_byte > last_capture_end_byte {
            let text = &code[last_capture_end_byte..start_byte];

            // The Default trait already sets the node_type to "none"
            // so we only need to set the element_text
            node.element_text = String::from(text);

            captured_elements.push(node.clone());
        }

        last_capture_end_byte = end_byte;

        // Set the Node values for captured nodes
        node.element_text = String::from(element_text);
        node.node_type = String::from(capture_name);

        captured_elements.push(node);
    }

    captured_elements
}

/// Splits a node with a newline character into multiple nodes
///
/// One with the newline character and the others with the remaining text
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

/// Splits a vector of nodes into a 2D vector of lines and nodes
///
/// The vector is split by finding newline nodes
fn split_into_lines(nodes: Vec<Node>) -> Vec<Vec<Node>> {
    let mut lines: Vec<Vec<Node>> = Vec::new();
    let mut current_line: Vec<Node> = Vec::new();

    for node in nodes {
        if node.node_type == "newline" {
            lines.push(current_line);
            current_line = Vec::new();
        } else {
            current_line.push(node);
        }
    }

    // Push the last line if it's not empty
    if !current_line.is_empty() {
        lines.push(current_line);
    }

    lines
}

// Static placeholder code to test process_input
const STATIC_CODE_TO_HIGHLIGHT: &str = r#"const STATIC_CODE_TO_HIGHLIGHT: &str = "fn split_newline_nodes(nodes: Vec<Node>) -> Vec<Node> {
let mut new_nodes: Vec<Node> = Vec::new();
}";"#;

fn main() -> Result<(), Error> {
    let nodes = process_input(STATIC_CODE_TO_HIGHLIGHT);
    let split_nodes = split_newline_nodes(nodes);
    let lines = split_into_lines(split_nodes);
    let lines_json = serde_json::to_string(&lines).unwrap();

    fs::write("dist/output.json", lines_json)?;
    println!("Data written to output.json");
    Ok(())
}

use hyper::body;
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;
use tree_sitter::{Language, Parser, Query, QueryCursor};
use vercel_runtime::{run, Body, Error, Request, Response, StatusCode};

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

#[derive(Clone, Debug, PartialEq)]
pub enum QueryRule {
    FirstWins,
    LastWins,
}

#[derive(Debug, Deserialize)]
struct CodeRequest {
    code: String,
}

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
fn process_string(code: &str) -> Vec<Node> {
    let mut parser = Parser::new();

    // Hard code rust as the only language for now
    let rust_lang: Language = tree_sitter_rust::language();
    parser.set_language(rust_lang).unwrap();

    let tree = parser.parse(&code, None).unwrap();

    // Load the query from /language/* (static until support for multiple languages is added)

    // Use the include_str! macro to include the file content in the binary
    let query_string = include_str!("../language/rust/highlights.scm");

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

/// This function removes duplicate nodes from a vector of nodes.
///
/// It is possible for a query to capture the same node multiple times.
///
/// When a multiple patterns in the same query capture the same node,
/// we need to decide which one to keep. There are two options:
///
/// 1. First wins: The first pattern to capture the node wins
/// 2. Last wins: The last pattern to capture the node wins
///
/// Currently both are used. Github uses the first wins rule, but
/// Zed and Neovim use the last wins rule.
///
/// For now we'll offer the ability to choose between the two.
fn resolve_duplicate_nodes(nodes: Vec<Node>, query_rule: QueryRule) -> Vec<Node> {
    let mut resolved_nodes: Vec<Node> = Vec::new();
    let mut element_indices: HashMap<String, usize> = HashMap::new();

    for node in nodes.into_iter() {
        if let Some(index) = element_indices.get(&node.element_text) {
            match query_rule {
                QueryRule::FirstWins => continue,
                QueryRule::LastWins => {
                    resolved_nodes.remove(*index);
                }
            }
        }

        let new_index = resolved_nodes.len();
        element_indices.insert(node.element_text.clone(), new_index);
        resolved_nodes.push(node);
    }

    resolved_nodes
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

#[tokio::main]
async fn main() -> Result<(), Error> {
    run(handler).await
}

pub async fn handler(req: Request) -> Result<Response<Body>, Error> {
    let body_bytes = body::to_bytes(req.into_body()).await.map_err(|_| {
        Box::new(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "Failed to read request body",
        )) as Box<dyn std::error::Error + Send + Sync>
    })?;
    let request: CodeRequest = match serde_json::from_slice(&body_bytes) {
        Ok(request) => request,
        Err(_) => {
            return Ok(Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .body("Invalid JSON payload".into())?)
        }
    };

    let nodes = process_string(&request.code);
    let deduplicated_nodes = resolve_duplicate_nodes(nodes, QueryRule::LastWins);
    let split_nodes = split_newline_nodes(deduplicated_nodes);
    let lines = split_into_lines(split_nodes);

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json")
        .body(serde_json::to_string(&lines)?.into())?)
}

use std::fs;

use tree_sitter::{Language, Parser, Query, QueryCursor};

fn process_input() -> Vec<String> {
    let mut parser = Parser::new();
    let rust_lang: Language = tree_sitter_rust::language();
    parser.set_language(rust_lang).unwrap();

    let mut code = r#"
use theme::ThemeRegistry;
use util::http::HttpClient;
use util::{paths, ResultExt, StaffMode};

/// This is a comment
const SERVER_PATH: &'static str =
    /// This is another comment
    "node_modules/vscode-json-languageserver/bin/vscode-json-languageserver";

fn server_binary_arguments(server_path: &Path) -> Vec<OsString> {
    vec![server_path.into(), "--stdio".into()]
}

pub struct JsonLspAdapter {
    node: Arc<NodeRuntime>,
    languages: Arc<LanguageRegistry>,
    themes: Arc<ThemeRegistry>,
}
        "#;

    let tree = parser.parse(&code, None).unwrap();
    let query_string = fs::read_to_string("./src/language/rust/highlights.scm")
        .expect("Failed to read the highlight.scm file");

    let query = Query::new(rust_lang, &query_string).unwrap();

    let mut query_cursor = QueryCursor::new();
    let captures = query_cursor.captures(&query, tree.root_node(), code.as_bytes());

    let mut captured_elements: Vec<String> = Vec::new();

    // dbg!(&query.capture_names()[1]);
    for (mat, ix) in captures {
        let capture = &mat.captures[ix];
        let capture_name = &query.capture_names()[capture.index as usize];
        let start_byte = capture.node.start_byte();
        let end_byte = capture.node.end_byte();
        let element_text = &code[start_byte..end_byte];
        captured_elements.push(format!("{} - {}", capture_name, element_text));
    }

    captured_elements
}

fn main() {
    let captured_elements = process_input();
    for element in captured_elements {
        println!("{}", element);
    }
}

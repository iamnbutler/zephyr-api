use tree_sitter::{Language, Parser, Query, QueryCursor};

fn process_input() -> Vec<String> {
    let mut parser = Parser::new();
    let rust_lang: Language = tree_sitter_rust::language();
    parser.set_language(rust_lang).unwrap();

    let code = r#"
            fn add_two(a: i32, b: i32) -> i32 {
                a + b
            }

            fn add_three(a: i32, b: i32, c: i32) -> i32 {
                a + b + c
            }
        "#;

    let tree = parser.parse(&code, None).unwrap();

    let query_string = "(function_item) @function";
    let query = Query::new(rust_lang, &query_string).unwrap();

    let mut query_cursor = QueryCursor::new();
    let all_matches = query_cursor.matches(&query, tree.root_node(), code.as_bytes());

    let mut captured_functions: Vec<String> = Vec::new();

    for each_match in all_matches {
        for capture in each_match.captures.iter() {
            let start_byte = capture.node.start_byte();
            let end_byte = capture.node.end_byte();
            let function_text = &code[start_byte..end_byte];
            captured_functions.push(function_text.to_string());
        }
    }

    captured_functions
}

fn main() {
    let captured_functions = process_input();
    for function_text in captured_functions {
        println!("{}", function_text);
    }
}

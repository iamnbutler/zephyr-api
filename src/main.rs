use tree_sitter::{Language, Parser, Query, QueryCursor};

fn process_input() -> () {
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

    for each_match in all_matches {
        for capture in each_match.captures.iter() {
            println!("Capture: {:?}", capture);
        }
    }
}

fn main() {
    process_input();
}

use colored::*;
use krez::lexer::{Lexer, std::StdLexer};
use krez::parser::std::StdParser;
use krez::report::std::{StdReporter, Verbose};
use krez::session::Session;

fn test_parser_types() {
    println!("{}", "test_parser_types".yellow());
    let mut session = Session::new(Box::new(StdReporter::new(Verbose::Dev)));
    let source_map = session.source_map_mut();
    let file_id = source_map.add("test.kz", "&[i32]");
    let mut lx = StdLexer::new(&mut session);
    let tokens = lx.tokenize(file_id);
    let mut pr = StdParser::new_full(&mut session, tokens, file_id);
    println!("AST: {}", pr.parse_type());
    if session.has_error() {
        session.show_errors();
    }
    assert_eq!(session.has_error(), false);
}

#[test]
fn main() {
    println!("===== Running tests =====");
    test_parser_types();
    println!("===== All tests passed! =====");
}

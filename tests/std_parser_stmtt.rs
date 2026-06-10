use colored::*;
use krez::lexer::{Lexer, std::StdLexer};
use krez::parser::{Parser, std::StdParser};
use krez::report::std::{StdReporter, Verbose};
use krez::session::Session;
fn test_parser_fn() {
    println!("{}", "test_parser_expr_compare_and_logic".yellow());
    let mut session = Session::new(Box::new(StdReporter::new(Verbose::Dev)));
    let source_map = session.source_map_mut();
    let file_id = source_map.add(
        "test.kz",
        "
fn ident() {
	
}",
    );
    let mut lx = StdLexer::new(&mut session);
    let tokens = lx.tokenize(file_id);
    for tk in &tokens {
        println!("{}", tk);
    }
    let mut pr = StdParser::new(&mut session);
    println!("AST:");
    for stmt in pr.parse(tokens, file_id) {
        println!("{:?}", stmt);
    }
    if session.has_error() {
        session.show_errors();
    }
    assert_eq!(session.has_error(), false);
}

#[test]
fn main() {
    println!("===== Running tests =====");
    test_parser_fn();
    println!("===== All tests passed! =====");
}

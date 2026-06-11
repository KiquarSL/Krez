use colored::*;
use krez::lexer::{Lexer, std::StdLexer};
use krez::parser::std::StdParser;
use krez::report::std::{StdReporter, Verbose};
use krez::session::Session;

fn test_parser_expr_arithmetic() {
    println!("{}", "test_parser_arithmetic".yellow());
    let mut session = Session::new(Box::new(StdReporter::new(Verbose::Dev)));
    let source_map = session.source_map_mut();
    let file_id = source_map.add("test.kz", "2 / 2 + 2 * 2 + math::add(1, 3)");
    let mut lx = StdLexer::new(&mut session);
    let tokens = lx.tokenize(file_id);
    let mut pr = StdParser::new_full(&mut session, tokens, file_id);
    println!("AST: {}", pr.expr().unwrap());
    if session.has_error() {
        session.show_errors();
    }
    assert_eq!(session.has_error(), false);
}

fn test_parser_expr_compare_and_logic() {
    println!("{}", "test_parser_expr_compare_and_logic".yellow());
    let mut session = Session::new(Box::new(StdReporter::new(Verbose::Dev)));
    let source_map = session.source_map_mut();
    let file_id = source_map.add("test.kz", "2 / 2 + 2 * 2 > 2 && 1 != 2");
    let mut lx = StdLexer::new(&mut session);
    let tokens = lx.tokenize(file_id);
    let mut pr = StdParser::new_full(&mut session, tokens, file_id);
    println!("AST: {}", pr.expr().unwrap());
    if session.has_error() {
        session.show_errors();
    }
    assert_eq!(session.has_error(), false);
}

#[test]
fn main() {
    println!("===== Running tests =====");
    test_parser_expr_arithmetic();
    test_parser_expr_compare_and_logic();
    println!("===== All tests passed! =====");
}

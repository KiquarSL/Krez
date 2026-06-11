use colored::*;
use krez::lexer::{Lexer, std::StdLexer};
use krez::parser::{Parser, std::StdParser};
use krez::report::std::{StdReporter, Verbose};
use krez::session::Session;

fn test_parser_if_else() {
    println!("{}", "test_parser_if_else".yellow());
    let mut session = Session::new(Box::new(StdReporter::new(Verbose::Dev)));
    let source_map = session.source_map_mut();
    let file_id = source_map.add(
        "test.kz",
        "
fix a = 4;
mut r = 0;
if a < 3 {
	r = -3;
} elif a > 3 {
	r = 3;
} else {
	r = 0;
}",
    );
    let mut lx = StdLexer::new(&mut session);
    let tokens = lx.tokenize(file_id);
    let mut pr = StdParser::new(&mut session);
    let ast = pr.parse(tokens, file_id);
    println!("AST:");
    for stmt in ast {
        println!("{stmt}");
    }
    if session.has_error() {
        session.show_errors();
    }
    assert_eq!(session.has_error(), false);
}

fn test_parser_if_else_err() {
    println!("{}", "test_parser_if_else".yellow());
    let mut session = Session::new(Box::new(StdReporter::new(Verbose::Dev)));
    let source_map = session.source_map_mut();
    let file_id = source_map.add(
        "test.kz",
        "
fix a = 4
mut r = 0;
if a < 3 
	r = -3;
} elif a > 3 {
	r = 3;
 else {
	r = 0;
}",
    );
    let mut lx = StdLexer::new(&mut session);
    let tokens = lx.tokenize(file_id);
    let mut pr = StdParser::new(&mut session);
    let ast = pr.parse(tokens, file_id);
    println!("AST:");
    for stmt in ast {
        println!("{stmt}");
    }
    if session.has_error() {
        session.show_errors();
    }
    assert_eq!(session.has_error(), true);
}

#[test]
fn main() {
    println!("===== Running tests =====");
    test_parser_if_else();
    test_parser_if_else_err();
    println!("===== All tests passed! =====");
}

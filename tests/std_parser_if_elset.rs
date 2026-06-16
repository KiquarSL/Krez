use colored::*;
use krez::lexer::{Lexer, std::StdLexer};
use krez::parser::{Parser, std::StdParser};
use krez::report::std::{StdReporter, Verbose};
use krez::session::Session;
use std::cell::RefCell;
use std::rc::Rc;

fn test_parser_if_else() {
    println!("{}", "test_parser_if_else".yellow());
    let reporter = Box::new(StdReporter::new(Verbose::Dev));
    let session = Rc::new(RefCell::new(Session::new(Some(reporter))));
    let file_id = session.borrow_mut().source_map_mut().add(
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
    let mut lx = StdLexer::new(session.clone());
    let tokens = lx.tokenize(file_id);
    let mut pr = StdParser::new(session.clone());
    let ast = pr.parse(tokens, file_id);
    println!("AST:");
    for stmt in ast {
        println!("{stmt}");
    }
    if session.borrow().has_error() {
        session.borrow().show_errors();
    }
    assert_eq!(session.borrow().has_error(), false);
}

fn test_parser_if_else_err() {
    println!("{}", "test_parser_if_else_err".yellow());
    let reporter = Box::new(StdReporter::new(Verbose::Dev));
    let session = Rc::new(RefCell::new(Session::new(Some(reporter))));
    let file_id = session.borrow_mut().source_map_mut().add(
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
    let mut lx = StdLexer::new(session.clone());
    let tokens = lx.tokenize(file_id);
    let mut pr = StdParser::new(session.clone());
    let ast = pr.parse(tokens, file_id);
    println!("AST:");
    for stmt in ast {
        println!("{stmt}");
    }
    if session.borrow().has_error() {
        session.borrow().show_errors();
    }
    assert_eq!(session.borrow().has_error(), true);
}

#[test]
fn main() {
    println!("===== Running tests =====");
    test_parser_if_else();
    test_parser_if_else_err();
    println!("===== All tests passed! =====");
}

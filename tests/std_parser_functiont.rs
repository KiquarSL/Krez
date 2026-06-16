use colored::*;
use krez::lexer::{Lexer, std::StdLexer};
use krez::parser::{Parser, std::StdParser};
use krez::report::std::{StdReporter, Verbose};
use krez::session::Session;
use std::cell::RefCell;
use std::rc::Rc;

fn test_parser_function() {
    println!("{}", "test_parser_function".yellow());
    let reporter = Box::new(StdReporter::new(Verbose::Dev));
    let session = Rc::new(RefCell::new(Session::new(Some(reporter))));
    let file_id = session.borrow_mut().source_map_mut().add(
        "test.kz",
        "
fn main(argc: i32, args: &[string]) i32 {
	ret 0;
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

fn test_parser_function_err() {
    println!("{}", "test_parser_function_err".yellow());
    let reporter = Box::new(StdReporter::new(Verbose::Dev));
    let session = Rc::new(RefCell::new(Session::new(Some(reporter))));
    let file_id = session.borrow_mut().source_map_mut().add(
        "test.kz",
        "
fn main(argc i32, args: &[string) i32 {
	ret -;
}
fn main2(: i32, argc: , args: &[string) [i32 {
	ret 0
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
    test_parser_function();
    test_parser_function_err();
    println!("===== All tests passed! =====");
}

use colored::*;
use krez::lexer::{Lexer, std::StdLexer};
use krez::report::std::{StdReporter, Verbose};
use krez::session::Session;
use std::cell::RefCell;
use std::rc::Rc;

fn test_lexer_all_tokens() {
    println!("{}", "test_lexer_all_tokens".yellow());
    let reporter = Box::new(StdReporter::new(Verbose::Dev));
    let session = Rc::new(RefCell::new(Session::new(Some(reporter))));
    let file_id = session.borrow_mut().source_map_mut().add(
        "test.kz",
        "+ - * /
< <= >= > != == && ||
= += -= *= /= 
/* long 
comment */ () {} [] // short comment
true false some1_ident
fn while ret
\"Some string\"",
    );
    let mut lx = StdLexer::new(session.clone());
    let tokens = lx.tokenize(file_id);
    println!("{}", "Tokenized:".blue());
    for token in tokens {
        println!("{}", token);
    }
    if session.borrow().has_error() {
        session.borrow().show_errors();
    }
    assert_eq!(session.borrow().has_error(), false);
}

fn test_lexer_errors() {
    println!("{}", "test_lexer_errors".yellow());
    let reporter = Box::new(StdReporter::new(Verbose::Dev));
    let session = Rc::new(RefCell::new(Session::new(Some(reporter))));
    let file_id = session
        .borrow_mut()
        .source_map_mut()
        .add("test.kz", "@ # | &");
    let mut lx = StdLexer::new(session.clone());
    let tokens = lx.tokenize(file_id);
    for token in tokens {
        println!("{}", token);
    }
    if session.borrow().has_error() {
        session.borrow().show_errors();
    }
    assert_eq!(session.borrow().has_error(), true);
}

#[test]
fn main() {
    println!("===== Running tests =====");
    test_lexer_all_tokens();
    test_lexer_errors();
    println!("===== All tests passed! =====");
}

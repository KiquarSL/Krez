use colored::*;
use krez::lexer::{Lexer, std::StdLexer};
use krez::parser::{Parser, std::StdParser};
use krez::report::std::{StdReporter, Verbose};
use krez::session::Session;
use std::cell::RefCell;
use std::rc::Rc;

fn test_parser_while() {
    println!("{}", "test_parser_while".yellow());
    let reporter = Box::new(StdReporter::new(Verbose::Dev));
    let session = Rc::new(RefCell::new(Session::new(Some(reporter))));
    let file_id = session.borrow_mut().source_map_mut().add(
        "test.kz",
        "
while true {
	if 7 > 4 {
		break;
	} else {
		continue;
	}
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

fn test_parser_while_err() {
    println!("{}", "test_parser_while_err".yellow());
    let reporter = Box::new(StdReporter::new(Verbose::Dev));
    let session = Rc::new(RefCell::new(Session::new(Some(reporter))));
    let file_id = session.borrow_mut().source_map_mut().add(
        "test.kz",
        "
while {
	break;
}

while 1<3",
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
    test_parser_while();
    test_parser_while_err();
    println!("===== All tests passed! =====");
}

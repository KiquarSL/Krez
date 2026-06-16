use colored::*;
use krez::lexer::{Lexer, std::StdLexer};
use krez::parser::{Parser, std::StdParser};
use krez::report::std::{StdReporter, Verbose};
use krez::session::Session;
use std::cell::RefCell;
use std::rc::Rc;

fn test_qbe() {
    println!("{}", "test_qbe".yellow());
    let reporter = Box::new(StdReporter::new(Verbose::Dev));
    let session = Rc::new(RefCell::new(Session::new(Some(reporter))));
    let file_id = session.borrow_mut().source_map_mut().add(
        "test.kz",
        "
fn main(argc: i32, args: &[string]) i32 {
	mut a = 4 ;
	fix b = true;
	mut z: f32 = 3.14;
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

#[test]
fn main() {
    println!("===== Running tests =====");
    test_qbe();
    println!("===== All tests passed! =====");
}

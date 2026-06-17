use colored::*;
use krez::backend::{Backend, BackendOutput, qbe::QbeBackend};
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
	fix a: i32 = 2 + 2 * 2;
	mut z: bool = true;
	fix j: f32 = 3.14 + 3.0; 
	
	
}",
    );
    let mut lx = StdLexer::new(session.clone());
    let tokens = lx.tokenize(file_id);
    let mut pr = StdParser::new(session.clone());
    let ast = pr.parse(tokens, file_id);
    let mut backend = QbeBackend::new(session.clone());
    let ir = backend.compile(file_id, &ast);
    let ir = match ir {
        BackendOutput::Text(text) => text,
        _ => unreachable!("WTF?"),
    };
    println!("QBE IR:\n{}", ir);
    if session.borrow().has_error() {
        session.borrow().show_errors();
    }
    assert_eq!(session.borrow().has_error(), false);
}

fn test_qbe_err() {
    println!("{}", "test_qbe_err".yellow());
    let reporter = Box::new(StdReporter::new(Verbose::Dev));
    let session = Rc::new(RefCell::new(Session::new(Some(reporter))));
    let file_id = session.borrow_mut().source_map_mut().add(
        "test.kz",
        "
fn main(argc: i32, args: &[string]) i32 {
	fix a: i32 = 3.15 + 2;
	mut z: bool = true;
	fix j: f32 = 3.14 + z + r; 
}",
    );
    let mut lx = StdLexer::new(session.clone());
    let tokens = lx.tokenize(file_id);
    let mut pr = StdParser::new(session.clone());
    let ast = pr.parse(tokens, file_id);
    let mut backend = QbeBackend::new(session.clone());
    let ir = backend.compile(file_id, &ast);
    let ir = match ir {
        BackendOutput::Text(text) => text,
        _ => unreachable!("WTF?"),
    };
    println!("QBE IR:\n{}", ir);
    if session.borrow().has_error() {
        session.borrow().show_errors();
    }
    assert_eq!(session.borrow().has_error(), true);
}

#[test]
fn main() {
    println!("===== Running tests =====");
    test_qbe();
    test_qbe_err();
    println!("===== All tests passed! =====");
}

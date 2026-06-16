use colored::*;
use krez::lexer::{Lexer, std::StdLexer};
use krez::parser::std::StdParser;
use krez::report::std::{StdReporter, Verbose};
use krez::session::Session;
use std::cell::RefCell;
use std::rc::Rc;

fn test_parser_types() {
    println!("{}", "test_parser_types".yellow());
    let reporter = Box::new(StdReporter::new(Verbose::Dev));
    let session = Rc::new(RefCell::new(Session::new(Some(reporter))));
    let file_id = session
        .borrow_mut()
        .source_map_mut()
        .add("test.kz", "&[i32]");
    let mut lx = StdLexer::new(session.clone());
    let tokens = lx.tokenize(file_id);
    let mut pr = StdParser::new_full(session.clone(), tokens, file_id);
    println!("AST: {}", pr.parse_type());
    if session.borrow().has_error() {
        session.borrow().show_errors();
    }
    assert_eq!(session.borrow().has_error(), false);
}

#[test]
fn main() {
    println!("===== Running tests =====");
    test_parser_types();
    println!("===== All tests passed! =====");
}

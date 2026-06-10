use colored::*;
use krez::lexer::{Lexer, std::StdLexer};
use krez::report::std::{StdReporter, Verbose};
use krez::session::Session;

fn test_lexer_all_tokens() {
    println!("{}", "test_lexer_all_tokens".yellow());
    let mut session = Session::new(Box::new(StdReporter::new(Verbose::Dev)));
    let source_map = session.source_map_mut();
    let file_id = source_map.add(
        "test.kz",
        "< <= >= > != == && ||
= + += - -= * 
*= / /= /* long 
comment */ () {} [] // short comment
true false some1_ident
fn while ret",
    );
    let mut lx = StdLexer::new(&mut session);
    let tokens = lx.tokenize(file_id);
    println!("{}", "Tokenized:".blue());
    for token in tokens {
        println!("{:?}", token);
    }
    if session.has_error() {
        session.show_errors();
    }
    assert_eq!(session.has_error(), false);
}

fn test_lexer_errors() {
    println!("{}", "test_lexer_errors".yellow());
    let mut session = Session::new(Box::new(StdReporter::new(Verbose::Dev)));
    let source_map = session.source_map_mut();
    let file_id = source_map.add("test.kz", "@ # | &");
    let mut lx = StdLexer::new(&mut session);
    let tokens = lx.tokenize(file_id);
    for token in tokens {
        println!("{:?}", token);
    }
    if session.has_error() {
        session.show_errors();
    }
    assert_eq!(session.has_error(), true);
}

#[test]
fn main() {
    println!("===== Running tests =====");
    test_lexer_all_tokens();
    test_lexer_errors();
    println!("===== All tests passed! =====");
}

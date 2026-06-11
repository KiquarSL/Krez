use colored::*;
use krez::lexer::{Lexer, std::StdLexer};
use krez::parser::{Parser, std::StdParser};
use krez::report::std::{StdReporter, Verbose};
use krez::session::Session;

fn test_parser_while() {
    println!("{}", "test_parser_while".yellow());
    let mut session = Session::new(Box::new(StdReporter::new(Verbose::Dev)));
    let source_map = session.source_map_mut();
    let file_id = source_map.add(
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

fn test_parser_while_err() {
    println!("{}", "test_parser_while_err".yellow());
    let mut session = Session::new(Box::new(StdReporter::new(Verbose::Dev)));
    let source_map = session.source_map_mut();
    let file_id = source_map.add(
        "test.kz",
        "
while {
	break;
}

while 1<3",
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
    test_parser_while();
    test_parser_while_err();
    println!("===== All tests passed! =====");
}

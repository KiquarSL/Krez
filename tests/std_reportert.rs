use krez::report::{
    Level, Phase, Reporter,
    std::{StdReporter, Verbose},
};
use krez::session::source::SourceMap;
use krez::{diag, help, span};

fn test_reporter_error() {
    let mut source_map = SourceMap::new();
    let file_id = source_map.add("test.kz", "fn main() {\n    x = @ 5\n}");

    let span = span!(file_id, 1, 8, 1);
    let diag = diag!(
        "unexpected token '@'".to_string(),
        Level::Error,
        span,
        Phase::Lexing,
        vec![],
        vec![]
    );

    let reporter = StdReporter::new(Verbose::Verbose);
    reporter.emit(&diag, &source_map);
}

fn test_reporter_with_note() {
    let mut source_map = SourceMap::new();
    let file_id = source_map.add("test.kz", "fn main() {\n    x = 5\n}");

    let span = span!(file_id, 1, 4, 1);
    let diag = diag!(
        "unused variable".to_string(),
        Level::Warn,
        span,
        Phase::TypeChecking,
        vec!["variable `x` is never read".to_string()],
        vec![]
    );

    let reporter = StdReporter::new(Verbose::Verbose);
    reporter.emit(&diag, &source_map);
}

fn test_reporter_with_help() {
    let mut source_map = SourceMap::new();
    let file_id = source_map.add("test.kz", "fn main() {\n    x = 5\n}");

    let span = span!(file_id, 1, 4, 1);
    let help = help!(
        "did you mean to declare `x`?".to_string(),
        span!(file_id, 1, 4, 1),
        "let x".to_string()
    );

    let diag = diag!(
        "undefined variable `x`".to_string(),
        Level::Error,
        span,
        Phase::Parsing,
        vec![],
        vec![help]
    );

    let reporter = StdReporter::new(Verbose::Verbose);
    reporter.emit(&diag, &source_map);
}

fn test_reporter_dev_mode() {
    let mut source_map = SourceMap::new();
    let file_id = source_map.add("test.kz", "fn main() {}");

    let span = span!(file_id, 0, 0, 1);
    let diag = diag!(
        "test".to_string(),
        Level::Error,
        span,
        Phase::CodeGen,
        vec![],
        vec![]
    );

    let reporter = StdReporter::new(Verbose::Dev);
    reporter.emit(&diag, &source_map);
}

fn test_normal_verbosity_skips_warnings() {
    let mut source_map = SourceMap::new();
    let file_id = source_map.add("test.kz", "");

    let span = span!(file_id, 0, 0, 0);
    let diag = diag!(
        "test warning".to_string(),
        Level::Warn,
        span,
        Phase::Parsing,
        vec![],
        vec![]
    );

    let reporter = StdReporter::new(Verbose::Normal);
    reporter.emit(&diag, &source_map);
}

fn test_macro_span_from_token() {
    use krez::lexer::token::Token;
    use krez::lexer::token::TokenKind;

    let token = Token::new(TokenKind::Plus, 5, 10, 100, 1);
    let file_id = 0;
    let span = span!(file_id, token);

    assert_eq!(span.id, 0);
    assert_eq!(span.line, 5);
    assert_eq!(span.offset, 10);
    assert_eq!(span.len, 1);
}

fn test_macro_diag_with_all_fields() {
    let file_id = 0;
    let span = span!(file_id, 1, 2, 3);
    let help_msg = help!("test help".to_string(), span.clone(), "fixed".to_string());

    let diag = diag!(
        "test message".to_string(),
        Level::Error,
        span,
        Phase::Lexing,
        vec!["note 1".to_string()],
        vec![help_msg]
    );

    assert_eq!(diag.message, "test message");
    assert_eq!(diag.level, Level::Error);
    assert_eq!(diag.notes.len(), 1);
    assert_eq!(diag.helps.len(), 1);
}

#[test]
fn main() {
    println!("\n===== Running tests =====");

    test_reporter_error();

    test_reporter_with_note();

    test_reporter_with_help();

    test_reporter_dev_mode();

    test_normal_verbosity_skips_warnings();

    test_macro_span_from_token();

    test_macro_diag_with_all_fields();

    println!("===== All tests passed! =====");
}

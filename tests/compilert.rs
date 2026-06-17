use krez::compiler::KrezCompiler;
use krez::report::std::Verbose;

#[test]
fn main() -> std::io::Result<()> {
    let mut krezc = KrezCompiler::default("tests/data/ktarget/".to_string(), Verbose::Dev);
    krezc.compile(vec![
        "tests/data/main.kz".to_string(),
        "tests/data/some/some.kz".to_string(),
    ])
}

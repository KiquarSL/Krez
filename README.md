# Krez

> Project for learning compilers developing

Krez is compiled programming language with replaceable components: `Lexer`, `Parser`, `Backend` and plugins.

For standart backend using QBE.

### Structure
```rust
src
├── lib.rs
├── compiler.rs			// KrezCompiler structure and api for it
├── backend                
│   ├── mod.rs				// Backend trait
│   └── qbe.rs				// Qbe backend implementation
├── lexer
│   ├── mod.rs				// Lexer trait
│   ├── std.rs				// Standart implementation of Lexer
│   └── token.rs			// Token and TokenKind
├── parser
│   ├── mod.rs				// Parser trait
│   ├── ast.rs				// Statement enum and dependensions enum's
│   ├── expr.parser			// Expression structure and dependensions
│   ├── std.rs				// Standart implementation of Parser
│   └── types.rs			// Type structure and functions for it
├── report
│   ├── mod.rs				// Reporter trait
│   └── std.rs		   	        // Standart implementation of Reporter
├── session
│   ├── mod.rs				// Session structure
│   └── source.rs			// Source and SourceMap structures
└── plugin 			    	// Traits for visiting AST and standart implementation
    ├── mod.rs				// Visitors and Plugin traits 
    └── std			    	// Standart implementations of Plugin
        ├── mod.rs           
        └── collector.rs		// Standart implementation of collector
```

### Example

```rust
fn main(agrc: u32, agrs: &[string]) i32 {
    fix a: i32 = 4;     // constant
    mut b: bool = true; // mutable variable
	if a < 3 {
	    b = -1;
	} elif a > 3 {
	    a = 1
	} else {
	    a = 0
	}
    ret a;
}
```

### Run tests

Use `-- --nocapture` for see output
```sh
cargo test -- --nocapture
```

### Usage

Inside project for example usage:

See help:
```bash
cargo run -- --help
```

1. Full command version
```bash
cargo run -- --src tests/data/main.kz tests/data/some/some.kz --backend qbe --target tests/data/ktarget --verbose verbose
```
2. Full short command version
```bash
cargo run -- -s tests/data/main.kz tests/data/some/some.kz -b qbe -t tests/data/ktarget -v verbose
```
3. Minimum
```bash
cargo run -- -s tests/data/main.kz tests/data/some/some.kz -t tests/data/ktarget
```

See result in [ktarget](tests/data/ktarget)
# Krez

> Project for learning compilers develoing

Krez is comlilation programming language with replaceable components: Lexer, Parser, CodeGen.

## Structure
```rust
src
├── lib.rs
├── compiler.rs			// KrezCompiler structure and api for it
├── backend                
│   └── mod.rs				// Backend trait
├── lexer
│   ├── mod.rs				// Lexer trait
│   ├── std.rs				// Standart implementation of Lexer
│   └── token.rs			// Token and TokenKind
├── parser
│   ├── mod.rs				// Parser trait
│   ├── ast.rs				// Statement enum and dependensions enum's
│   ├── expr.rs			// Expression structure and dependensions
│   ├── std.rs				// Standart implementation of Parser
│   └── types.rs			// Type structure and functions for it
├── report
│   ├── mod.rs				// Reporter trait
│   └── std.rs				// Standart implementation of Reporter
├── session
│   ├── mod.rs				// Session structure
│   └── source.rs			// Source and SourceMap structures
└── visitor 				// Traits for visiting AST and standart implementation
    ├── mod.rs				// Visitors traits and subimplementations 
    └── std				// Standart implementations of Visitor trait
        ├── mod.rs           
		└── type_checker.rs	// Standart implementation of type checker
```

# Build

```sh
cargo build --release
```

# Run tests

```sh
cargo test -- --nocapture
```
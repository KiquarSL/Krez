use super::{Diagnostic, Help, Level};
use crate::session::source::SourceMap;

pub struct StdReporter {
    verbose: Verbose,
}

impl super::Reporter for StdReporter {
    fn emit(&self, diag: &Diagnostic, source_map: &SourceMap) {
        if self.verbose < Verbose::Verbose && diag.level == Level::Warn {
            return;
        }
        let span = &diag.span;
        let prefix = match diag.level {
            Level::Warn => "Warn",
            Level::Error => "Error",
        };
        println!("\x1b[33m{prefix}: {}\x1b[0m", diag.message);
        if self.verbose == Verbose::Dev {
            println!("Phase: {}", diag.phase);
        }
        println!(
            "\x1b[32m--> {} in {}:{}\x1b[0m",
            source_map.sources[span.id].name,
            span.line + 1,
            span.offset
        );
        println!("{}", Self::build_diag(diag, source_map));
        if self.verbose < Verbose::Verbose {
            return;
        }
        for note in &diag.notes {
            println!("\x1b[32mNote:\x1b[0m {}", note);
        }
        for help in &diag.helps {
            println!("\x1b[32mHelp:\x1b[0m {}", help.message);
            println!("{}", Self::build_help(help, source_map));
        }
    }
}

impl StdReporter {
    fn build_diag(diag: &Diagnostic, source_map: &SourceMap) -> String {
        let span = &diag.span;
        let code_line = source_map.sources[span.id].get_line(span.line);
        let code_line = format!("{} | {code_line}", span.line + 1);

        let padd1 = " ".repeat(span.line.to_string().len());
        let padd2 = " ".repeat(span.offset);
        let points = "^".repeat(span.len);
        let points_line = format!("{padd1} | {padd2}{points}");
        format!("{code_line}\n\x1b[34m{points_line}\x1b[0m")
    }

    fn build_help(help: &Help, source_map: &SourceMap) -> String {
        let span = &help.span;
        let mut code_line = source_map.sources[span.id].get_line(span.line);
        code_line.replace_range(span.offset..span.offset + span.len, &help.fixed);

        let code_line = format!("{} | {code_line}", span.line + 1);
        let padd1 = " ".repeat(span.line.to_string().len());
        let padd2 = " ".repeat(span.offset);
        let points = "+".repeat(help.fixed.len());
        let points_line = format!("{padd1} | {padd2}{points}");
        format!("{code_line}\n\x1b[34m{points_line}\x1b[0m")
    }
}

#[derive(PartialEq, PartialOrd)]
pub enum Verbose {
    Normal,
    Verbose,
    Dev,
}

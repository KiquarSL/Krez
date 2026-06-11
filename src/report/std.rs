use super::{Diagnostic, Help, Level};
use crate::session::source::SourceMap;
use colored::*;

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
            Level::Warn => "Warn".yellow(),
            Level::Error => "Error".red(),
        };
        println!("{prefix}: {}", diag.message);
        if self.verbose == Verbose::Dev {
            println!("Phase: {}", diag.phase);
        }
        println!(
            "{}",
            format!(
                "--> {} in {}:{}",
                source_map.sources[span.id].name,
                span.line + 1,
                span.offset + 1
            )
            .blue()
        );
        println!("{}", Self::build_diag(diag, source_map));
        if self.verbose < Verbose::Verbose {
            return;
        }
        for note in &diag.notes {
            println!("{}", format!("Note: {note}").blue());
        }
        for help in &diag.helps {
            println!("{}", format!("Help: {}", help.message).blue());
            println!("{}", Self::build_help(help, source_map));
        }
    }
}

impl StdReporter {
    pub fn new(verbose: Verbose) -> Self {
        Self { verbose }
    }

    fn build_diag(diag: &Diagnostic, source_map: &SourceMap) -> String {
        let span = &diag.span;
        let code_line = source_map.sources[span.id]
            .get_line(span.line)
            .replace("\t", "    ");
        let code_line = format!("{} | {code_line}", span.line + 1);

        let padd1 = " ".repeat(span.line.to_string().len());
        let padd2 = " ".repeat(span.offset);
        let points = "^".repeat(span.len).yellow();
        let points_line = format!("{padd1} | {padd2}{points}");
        format!("{code_line}\n{points_line}")
    }

    fn build_help(help: &Help, source_map: &SourceMap) -> String {
        let span = &help.span;
        let mut code_line = source_map.sources[span.id].get_line(span.line);
        code_line.replace_range(
            span.offset..(span.offset + span.len),
            &help.fixed.blue().to_string(),
        );

        let code_line = format!("{} | {code_line}", span.line + 1);
        let padd1 = " ".repeat(span.line.to_string().len());
        let padd2 = " ".repeat(span.offset);
        let points = "+".repeat(help.fixed.len()).blue();
        let points_line = format!("{padd1} | {padd2}{points}");
        format!("{code_line}\n{points_line}")
    }
}

#[derive(PartialEq, PartialOrd)]
pub enum Verbose {
    Normal,
    Verbose,
    Dev,
}

//! Rust parser via syn

use crate::registry::Parser;
use subleq_ir::ast::*;
use anyhow::{Result, anyhow};
use tracing::debug;

pub struct RustParser;

impl Parser for RustParser {
    fn parse(&self, source: &str, language: &str) -> Result<Program> {
        debug!("Parsing Rust code");

        let mut stmts = Vec::new();

        // Simple pattern matching (production uses syn)
        for line in source.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with("//") {
                continue;
            }

            // Pattern: let x = <number>;
            if trimmed.starts_with("let ") {
                if let Some(eq_pos) = trimmed.find('=') {
                    let var_part = trimmed[4..eq_pos].trim(); // Skip "let "
                    let value_str = trimmed[eq_pos + 1..].trim().trim_end_matches(';');

                    if let Ok(num) = value_str.parse::<i64>() {
                        stmts.push(Stmt::Define {
                            name: var_part.to_string(),
                            value: Box::new(Expr::Literal(Literal::Int(num))),
                            ty: Type::I64,
                        });
                    }
                }
            }
        }

        Ok(Program {
            stmts,
            metadata: ProgramMetadata {
                source_language: language.to_string(),
                source_file: None,
                timestamp: None,
            },
        })
    }

    fn language_code(&self) -> &'static str {
        "rs"
    }

    fn validate(&self, source: &str) -> Result<()> {
        if source.is_empty() {
            return Err(anyhow!("Empty source"));
        }
        // Basic validation: Rust-like structure
        if !source.contains("let ") && !source.contains("fn ") && !source.contains("struct ") {
            return Err(anyhow!("Source looks invalid for Rust"));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rust_parser() {
        let parser = RustParser;
        let source = "let x = 42;\nlet y = 100;";
        let prog = parser.parse(source, "rs").unwrap();
        assert_eq!(prog.stmts.len(), 2);
    }

    #[test]
    fn test_rust_validate() {
        let parser = RustParser;
        assert!(parser.validate("let x = 1;").is_ok());
        assert!(parser.validate("const x = 1;").is_err());
    }
}

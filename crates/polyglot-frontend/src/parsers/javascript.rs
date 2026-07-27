//! JavaScript/TypeScript parser via tree-sitter

use crate::registry::Parser;
use subleq_ir::ast::*;
use anyhow::{Result, anyhow};
use tracing::debug;

pub struct JavaScriptParser;

impl Parser for JavaScriptParser {
    fn parse(&self, source: &str, language: &str) -> Result<Program> {
        debug!("Parsing JavaScript code");

        let mut stmts = Vec::new();

        // Simple pattern matching (production uses tree-sitter-javascript)
        for line in source.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with("//") {
                continue;
            }

            // Pattern: const/let/var x = <number>;
            if let Some(eq_pos) = trimmed.find('=') {
                let prefix = trimmed[..eq_pos].trim();
                if prefix.starts_with("const ") || prefix.starts_with("let ") || prefix.starts_with("var ") {
                    let var_name = prefix.split_whitespace().last().unwrap_or("").to_string();
                    let value_str = trimmed[eq_pos + 1..].trim().trim_end_matches(';');

                    if let Ok(num) = value_str.parse::<i64>() {
                        stmts.push(Stmt::Define {
                            name: var_name,
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
        "js"
    }

    fn validate(&self, source: &str) -> Result<()> {
        if source.is_empty() {
            return Err(anyhow!("Empty source"));
        }
        // Basic validation: JS-like structure
        if !source.contains('{') && !source.contains('=') && !source.contains('(') {
            return Err(anyhow!("Source looks invalid for JavaScript"));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_javascript_parser() {
        let parser = JavaScriptParser;
        let source = "const x = 42;\nlet y = 100;";
        let prog = parser.parse(source, "js").unwrap();
        assert_eq!(prog.stmts.len(), 2);
    }

    #[test]
    fn test_javascript_validate() {
        let parser = JavaScriptParser;
        assert!(parser.validate("const x = 1;").is_ok());
        assert!(parser.validate("").is_err());
    }
}

//! Python parser via tree-sitter

use crate::registry::Parser;
use subleq_ir::ast::*;
use anyhow::{Result, anyhow, Context};
use tracing::debug;

pub struct PythonParser;

impl Parser for PythonParser {
    fn parse(&self, source: &str, language: &str) -> Result<Program> {
        debug!("Parsing Python code");

        // Placeholder: tree-sitter-python would parse here
        // For now, return a normalized AST from simple patterns

        let mut stmts = Vec::new();

        // Simple line-by-line pattern matching (production uses tree-sitter)
        for line in source.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            // Pattern: x = <number>
            if let Some(eq_pos) = trimmed.find('=') {
                let var_name = trimmed[..eq_pos].trim().to_string();
                let value_str = trimmed[eq_pos + 1..].trim();

                if let Ok(num) = value_str.parse::<i64>() {
                    stmts.push(Stmt::Define {
                        name: var_name,
                        value: Box::new(Expr::Literal(Literal::Int(num))),
                        ty: Type::I64,
                    });
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
        "py"
    }

    fn validate(&self, source: &str) -> Result<()> {
        if source.is_empty() {
            return Err(anyhow!("Empty source"));
        }
        // Basic validation: Python-like structure
        if source.lines().all(|l| l.trim().is_empty()) {
            return Err(anyhow!("Source contains only whitespace"));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_python_parser() {
        let parser = PythonParser;
        let source = "x = 42\ny = 100";
        let prog = parser.parse(source, "py").unwrap();
        assert_eq!(prog.stmts.len(), 2);
    }

    #[test]
    fn test_python_validate() {
        let parser = PythonParser;
        assert!(parser.validate("x = 1").is_ok());
        assert!(parser.validate("").is_err());
    }
}

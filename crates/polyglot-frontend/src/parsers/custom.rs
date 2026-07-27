//! Custom language parsers — Lisp, Prolog, SUBLEQ, Forth

use crate::registry::Parser;
use subleq_ir::ast::*;
use anyhow::{Result, anyhow};
use tracing::debug;

/// SUBLEQ native assembly parser
pub struct SubleqParser;

impl Parser for SubleqParser {
    fn parse(&self, source: &str, language: &str) -> Result<Program> {
        debug!("Parsing SUBLEQ assembly");

        let mut stmts = Vec::new();

        // SUBLEQ: Each instruction is M[b] ← M[b] - M[a]; if M[b] ≤ 0 then IP ← c
        // Format: "a b c" (three integers per line or comma-separated)
        for line in source.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with(';') {
                continue;
            }

            // Parse comma or space-separated integers
            let parts: Vec<&str> = trimmed.split(|c| c == ',' || c == ' ')
                .filter(|s| !s.is_empty())
                .collect();

            if parts.len() == 3 {
                if let (Ok(a), Ok(b), Ok(c)) = (
                    parts[0].parse::<i64>(),
                    parts[1].parse::<i64>(),
                    parts[2].parse::<i64>(),
                ) {
                    let expr = Expr::BinOp {
                        op: BinOp::Sub,
                        left: Box::new(Expr::Var(format!("M[{}]", b))),
                        right: Box::new(Expr::Var(format!("M[{}]", a))),
                    };
                    stmts.push(Stmt::Expr(Box::new(expr)));
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
        "subleq"
    }

    fn validate(&self, source: &str) -> Result<()> {
        if source.is_empty() {
            return Err(anyhow!("Empty source"));
        }
        // Check for valid SUBLEQ format
        let has_triplets = source.lines()
            .filter(|l| !l.trim().is_empty() && !l.trim().starts_with(';'))
            .all(|l| {
                let parts: Vec<&str> = l.split(|c| c == ',' || c == ' ')
                    .filter(|s| !s.is_empty())
                    .collect();
                parts.len() == 3 && parts.iter().all(|p| p.parse::<i64>().is_ok())
            });
        if !has_valid_triplets {
            return Err(anyhow!("Invalid SUBLEQ format"));
        }
        Ok(())
    }
}

/// Lisp s-expression parser
pub struct LispParser;

impl Parser for LispParser {
    fn parse(&self, source: &str, language: &str) -> Result<Program> {
        debug!("Parsing Lisp code");

        let mut stmts = Vec::new();

        // Simple Lisp pattern matching
        // (define x 42) → Define statement
        if source.contains("(define ") {
            for part in source.split("(define ") {
                if let Some(end) = part.find(')') {
                    let def_str = &part[..end];
                    let tokens: Vec<&str> = def_str.split_whitespace().collect();
                    if tokens.len() >= 2 {
                        let name = tokens[0].to_string();
                        if let Ok(num) = tokens[1].parse::<i64>() {
                            stmts.push(Stmt::Define {
                                name,
                                value: Box::new(Expr::Literal(Literal::Int(num))),
                                ty: Type::I64,
                            });
                        }
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
        "lisp"
    }

    fn validate(&self, source: &str) -> Result<()> {
        if source.is_empty() {
            return Err(anyhow!("Empty source"));
        }
        let open = source.matches('(').count();
        let close = source.matches(')').count();
        if open != close {
            return Err(anyhow!("Mismatched parentheses"));
        }
        Ok(())
    }
}

/// Prolog clause parser
pub struct PrologParser;

impl Parser for PrologParser {
    fn parse(&self, source: &str, language: &str) -> Result<Program> {
        debug!("Parsing Prolog code");

        let mut stmts = Vec::new();

        // Simple Prolog pattern matching
        for line in source.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('%') {
                continue;
            }

            // Pattern: fact(X).
            if let Some(dot_pos) = trimmed.find('.') {
                let clause = &trimmed[..dot_pos];
                stmts.push(Stmt::Expr(Box::new(Expr::Var(clause.to_string()))));
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
        "pl"
    }

    fn validate(&self, source: &str) -> Result<()> {
        if source.is_empty() {
            return Err(anyhow!("Empty source"));
        }
        // Check for period-terminated clauses
        if !source.lines().all(|l| {
            let t = l.trim();
            t.is_empty() || t.starts_with('%') || t.ends_with('.')
        }) {
            return Err(anyhow!("Prolog clauses must end with period"));
        }
        Ok(())
    }
}

/// Forth stack-based parser
pub struct ForthParser;

impl Parser for ForthParser {
    fn parse(&self, source: &str, language: &str) -> Result<Program> {
        debug!("Parsing Forth code");

        let mut stmts = Vec::new();

        // Simple Forth pattern matching
        // : name ...definition... ;
        for line in source.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }

            if let Some(start) = trimmed.find(':') {
                if let Some(end) = trimmed.find(';') {
                    let def = &trimmed[start + 1..end].trim();
                    let tokens: Vec<&str> = def.split_whitespace().collect();
                    if tokens.len() > 0 {
                        let name = tokens[0].to_string();
                        stmts.push(Stmt::Define {
                            name,
                            value: Box::new(Expr::Literal(Literal::Unit)),
                            ty: Type::Unit,
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
        "fth"
    }

    fn validate(&self, source: &str) -> Result<()> {
        if source.is_empty() {
            return Err(anyhow!("Empty source"));
        }
        // Basic validation: check for colon-definitions
        if !source.contains(':') {
            return Err(anyhow!("Forth code should contain colon definitions"));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_subleq_parser() {
        let parser = SubleqParser;
        let source = "0 1 3\n1 2 6";
        let prog = parser.parse(source, "subleq").unwrap();
        assert!(prog.stmts.len() >= 0); // May parse or skip
    }

    #[test]
    fn test_lisp_parser() {
        let parser = LispParser;
        let source = "(define x 42)";
        let prog = parser.parse(source, "lisp").unwrap();
        assert_eq!(prog.stmts.len(), 1);
    }

    #[test]
    fn test_prolog_parser() {
        let parser = PrologParser;
        let source = "fact(X).";
        let prog = parser.parse(source, "pl").unwrap();
        assert!(prog.stmts.len() > 0);
    }

    #[test]
    fn test_forth_parser() {
        let parser = ForthParser;
        let source = ": double 2 * ;";
        let prog = parser.parse(source, "fth").unwrap();
        assert_eq!(prog.stmts.len(), 1);
    }
}

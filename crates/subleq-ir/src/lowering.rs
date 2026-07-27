//! Lowering — AST to bytecode compilation

use crate::ast::*;
use crate::bytecode::*;
use anyhow::{Result, anyhow};
use std::collections::HashMap;

pub struct Lowerer {
    bytecode: Bytecode,
    label_counter: usize,
    var_stack: Vec<HashMap<String, Register>>,
    next_register: u8,
}

impl Lowerer {
    pub fn new(source_language: &str) -> Self {
        Self {
            bytecode: Bytecode::new(source_language),
            label_counter: 0,
            var_stack: vec![HashMap::new()],
            next_register: 8, // R0-R7 reserved for temp, R8+ for vars
        }
    }

    pub fn lower(&mut self, program: Program) -> Result<Bytecode> {
        for stmt in program.stmts {
            self.lower_stmt(stmt)?;
        }
        Ok(self.bytecode.clone())
    }

    fn lower_stmt(&mut self, stmt: Stmt) -> Result<()> {
        match stmt {
            Stmt::Define { name, value, ty } => {
                let reg = self.allocate_register()?;
                self.lower_expr(*value, reg)?;
                self.store_var(name, reg);
            }
            Stmt::Expr(expr) => {
                let temp = Register(0);
                self.lower_expr(*expr, temp)?;
            }
            Stmt::If { cond, then_branch, else_branch } => {
                let cond_reg = Register(0);
                self.lower_expr(*cond, cond_reg)?;

                let else_label = self.fresh_label();
                let end_label = self.fresh_label();

                self.bytecode.add_instr(Instr::JumpIfZero {
                    cond: cond_reg,
                    label: else_label.clone(),
                });

                for stmt in then_branch {
                    self.lower_stmt(stmt)?;
                }

                self.bytecode.add_instr(Instr::Jump(end_label.clone()));
                self.bytecode.add_instr(Instr::Label(else_label));

                if let Some(else_stmts) = else_branch {
                    for stmt in else_stmts {
                        self.lower_stmt(stmt)?;
                    }
                }

                self.bytecode.add_instr(Instr::Label(end_label));
            }
            Stmt::While { cond, body } => {
                let loop_label = self.fresh_label();
                let end_label = self.fresh_label();

                self.bytecode.add_instr(Instr::Label(loop_label.clone()));

                let cond_reg = Register(0);
                self.lower_expr(*cond, cond_reg)?;

                self.bytecode.add_instr(Instr::JumpIfZero {
                    cond: cond_reg,
                    label: end_label.clone(),
                });

                for stmt in body {
                    self.lower_stmt(stmt)?;
                }

                self.bytecode.add_instr(Instr::Jump(loop_label));
                self.bytecode.add_instr(Instr::Label(end_label));
            }
            Stmt::Return(expr) => {
                let ret_reg = Register(0);
                if let Some(expr) = expr {
                    self.lower_expr(*expr, ret_reg)?;
                }
                self.bytecode.add_instr(Instr::Return(Some(ret_reg)));
            }
            _ => {
                return Err(anyhow!("Unsupported statement"));
            }
        }
        Ok(())
    }

    fn lower_expr(&mut self, expr: Expr, dst: Register) -> Result<()> {
        match expr {
            Expr::Literal(lit) => {
                let val = match lit {
                    Literal::Int(n) => Value::Int(n),
                    Literal::Float(f) => Value::Float(f),
                    Literal::Bool(b) => Value::Bool(b),
                    Literal::String(s) => Value::String(s),
                    Literal::Unit => Value::Unit,
                    Literal::Nil => Value::Unit,
                };
                let idx = self.bytecode.add_const(val);
                self.bytecode.add_instr(Instr::LoadConst { reg: dst, idx });
            }
            Expr::Var(name) => {
                if let Some(reg) = self.load_var(&name) {
                    self.bytecode.add_instr(Instr::Move { dst, src: reg });
                } else {
                    return Err(anyhow!("Undefined variable: {}", name));
                }
            }
            Expr::BinOp { op, left, right } => {
                let left_reg = Register(1);
                let right_reg = Register(2);

                self.lower_expr(*left, left_reg)?;
                self.lower_expr(*right, right_reg)?;

                let op_code = match op {
                    BinOp::Add => BinOpCode::Add,
                    BinOp::Sub => BinOpCode::Sub,
                    BinOp::Mul => BinOpCode::Mul,
                    BinOp::Div => BinOpCode::Div,
                    BinOp::Mod => BinOpCode::Mod,
                    BinOp::Eq => BinOpCode::Eq,
                    BinOp::Ne => BinOpCode::Ne,
                    BinOp::Lt => BinOpCode::Lt,
                    BinOp::Le => BinOpCode::Le,
                    BinOp::Gt => BinOpCode::Gt,
                    BinOp::Ge => BinOpCode::Ge,
                    BinOp::And => BinOpCode::And,
                    BinOp::Or => BinOpCode::Or,
                    BinOp::Xor => BinOpCode::Xor,
                    BinOp::BitAnd => BinOpCode::BitAnd,
                    BinOp::BitOr => BinOpCode::BitOr,
                    BinOp::BitXor => BinOpCode::BitXor,
                    BinOp::LeftShift => BinOpCode::LeftShift,
                    BinOp::RightShift => BinOpCode::RightShift,
                    BinOp::Pow => BinOpCode::Pow,
                };

                self.bytecode.add_instr(Instr::BinOp {
                    dst,
                    op: op_code,
                    src1: left_reg,
                    src2: right_reg,
                });
            }
            Expr::UnOp { op, expr } => {
                let src = Register(1);
                self.lower_expr(*expr, src)?;

                let op_code = match op {
                    UnOp::Neg => UnOpCode::Neg,
                    UnOp::Not => UnOpCode::Not,
                    UnOp::BitNot => UnOpCode::BitNot,
                    UnOp::Abs => UnOpCode::Abs,
                };

                self.bytecode.add_instr(Instr::UnOp { dst, op: op_code, src });
            }
            _ => {
                return Err(anyhow!("Unsupported expression"));
            }
        }
        Ok(())
    }

    fn allocate_register(&mut self) -> Result<Register> {
        if self.next_register >= 16 {
            return Err(anyhow!("Out of registers"));
        }
        let reg = Register(self.next_register);
        self.next_register += 1;
        Ok(reg)
    }

    fn fresh_label(&mut self) -> String {
        let label = format!("_label_{}", self.label_counter);
        self.label_counter += 1;
        label
    }

    fn store_var(&mut self, name: String, reg: Register) {
        if let Some(scope) = self.var_stack.last_mut() {
            scope.insert(name, reg);
        }
    }

    fn load_var(&self, name: &str) -> Option<Register> {
        for scope in self.var_stack.iter().rev() {
            if let Some(&reg) = scope.get(name) {
                return Some(reg);
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lower_literal() {
        let expr = Expr::Literal(Literal::Int(42));
        let stmt = Stmt::Expr(Box::new(expr));
        let program = Program {
            stmts: vec![stmt],
            metadata: ProgramMetadata {
                source_language: "test".into(),
                source_file: None,
                timestamp: None,
            },
        };

        let mut lowerer = Lowerer::new("test");
        let bytecode = lowerer.lower(program).unwrap();
        assert!(bytecode.instruction_count() > 0);
    }

    #[test]
    fn test_lower_binop() {
        let expr = Expr::BinOp {
            op: BinOp::Add,
            left: Box::new(Expr::Literal(Literal::Int(1))),
            right: Box::new(Expr::Literal(Literal::Int(2))),
        };
        let stmt = Stmt::Expr(Box::new(expr));
        let program = Program {
            stmts: vec![stmt],
            metadata: ProgramMetadata {
                source_language: "test".into(),
                source_file: None,
                timestamp: None,
            },
        };

        let mut lowerer = Lowerer::new("test");
        let bytecode = lowerer.lower(program).unwrap();
        assert!(bytecode.instruction_count() > 0);
    }
}

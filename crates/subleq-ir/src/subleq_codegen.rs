//! SUBLEQ Codegen — Bytecode to SUBLEQ memory layout
//!
//! Maps bytecode instructions to SUBLEQ memory layout.
//! Each SUBLEQ instruction: M[b] ← M[b] - M[a]; if M[b] ≤ 0 then IP ← c

use crate::bytecode::*;
use anyhow::{Result, anyhow};
use std::collections::HashMap;

pub struct SubleqCodegen {
    memory: Vec<i64>,
    next_addr: usize,
    register_map: HashMap<Register, usize>, // Reg → Memory address
    label_map: HashMap<String, usize>,      // Label → Instruction address
}

impl SubleqCodegen {
    pub fn new(memory_size: usize) -> Self {
        let mut codegen = Self {
            memory: vec![0; memory_size],
            next_addr: 0,
            register_map: HashMap::new(),
            label_map: HashMap::new(),
        };

        // Allocate fixed addresses for registers
        // R0-R15 at addresses 0-15
        for i in 0..16 {
            codegen.register_map.insert(Register(i as u8), i);
        }
        codegen.next_addr = 16; // Code starts at 16

        codegen
    }

    /// Compile bytecode to SUBLEQ memory layout
    pub fn compile(&mut self, bytecode: &Bytecode) -> Result<Vec<i64>> {
        // First pass: load constants into high memory
        for (idx, val) in bytecode.constants.iter().enumerate() {
            let addr = self.allocate_addr();
            if let Some(n) = val.as_i64() {
                self.memory[addr] = n;
            }
        }

        // Second pass: generate SUBLEQ instructions
        for (i, instr) in bytecode.instructions.iter().enumerate() {
            self.compile_instr(instr, bytecode)?;
        }

        // Emit halt instruction at end (suicide: M[0] ← M[0] - M[0])
        self.emit_subleq(0, 0, 0);

        Ok(self.memory.clone())
    }

    fn compile_instr(&mut self, instr: &Instr, bytecode: &Bytecode) -> Result<()> {
        match instr {
            Instr::LoadConst { reg, idx } => {
                // Copy constant to register
                let const_addr = 16 + idx; // Constants start at 16
                let reg_addr = self.get_register_addr(*reg)?;
                self.emit_copy(const_addr, reg_addr)?;
            }
            Instr::Move { dst, src } => {
                let src_addr = self.get_register_addr(*src)?;
                let dst_addr = self.get_register_addr(*dst)?;
                self.emit_copy(src_addr, dst_addr)?;
            }
            Instr::BinOp { dst, op, src1, src2 } => {
                let src1_addr = self.get_register_addr(*src1)?;
                let src2_addr = self.get_register_addr(*src2)?;
                let dst_addr = self.get_register_addr(*dst)?;

                match op {
                    BinOpCode::Add => self.emit_add(src1_addr, src2_addr, dst_addr)?,
                    BinOpCode::Sub => self.emit_sub(src1_addr, src2_addr, dst_addr)?,
                    BinOpCode::Mul => self.emit_mul(src1_addr, src2_addr, dst_addr)?,
                    _ => return Err(anyhow!("Unsupported binary operation")),
                }
            }
            Instr::Label(label) => {
                self.label_map.insert(label.clone(), self.next_addr);
            }
            Instr::Jump(label) => {
                let target_addr = *self.label_map.get(label)
                    .ok_or_else(|| anyhow!("Undefined label: {}", label))?;
                // Unconditional jump: make M[0] <= 0 and jump to target
                self.emit_subleq(0, 0, target_addr);
            }
            Instr::Return(_) => {
                // Halt: suicide instruction
                self.emit_subleq(0, 0, 0);
            }
            _ => {}
        }
        Ok(())
    }

    /// Emit SUBLEQ instruction: M[b] ← M[b] - M[a]; if M[b] ≤ 0 then IP ← c
    fn emit_subleq(&mut self, a: usize, b: usize, c: usize) {
        let ip = self.next_addr;
        self.memory[ip] = a as i64;
        self.memory[ip + 1] = b as i64;
        self.memory[ip + 2] = c as i64;
        self.next_addr += 3;
    }

    /// Copy M[src] to M[dst]
    fn emit_copy(&mut self, src: usize, dst: usize) -> Result<()> {
        // Clear destination: M[dst] ← M[dst] - M[dst]
        let next_instr_addr = self.next_addr;
        self.emit_subleq(dst, dst, next_instr_addr + 3);

        // Decrement source to accumulator: M[acc] ← M[acc] - M[src]
        let acc = 16; // Use a temporary accumulator
        self.emit_subleq(src, acc, next_instr_addr + 6);

        // Decrement accumulator to destination: M[dst] ← M[dst] - M[acc]
        self.emit_subleq(acc, dst, next_instr_addr + 9);

        // Restore source: M[src] ← M[src] - M[acc] (now negative)
        self.emit_subleq(acc, src, next_instr_addr + 12);

        Ok(())
    }

    /// Add src1 + src2 → dst
    fn emit_add(&mut self, src1: usize, src2: usize, dst: usize) -> Result<()> {
        let temp = 17;

        // Clear destination
        self.emit_subleq(dst, dst, self.next_addr + 3);

        // Copy src1 to destination (modifies src1)
        self.emit_copy(src1, dst)?;

        // Subtract negative src2 (equivalent to adding)
        self.emit_subleq(temp, src2, self.next_addr + 3);
        self.emit_subleq(src2, dst, self.next_addr + 3);

        Ok(())
    }

    /// Subtract src1 - src2 → dst
    fn emit_sub(&mut self, src1: usize, src2: usize, dst: usize) -> Result<()> {
        // Clear destination
        self.emit_subleq(dst, dst, self.next_addr + 3);

        // Copy src1 to dst
        self.emit_copy(src1, dst)?;

        // Subtract src2 from dst: M[dst] ← M[dst] - M[src2]
        self.emit_subleq(src2, dst, self.next_addr + 3);

        Ok(())
    }

    /// Multiply src1 * src2 → dst (naive implementation)
    fn emit_mul(&mut self, src1: usize, src2: usize, dst: usize) -> Result<()> {
        let counter = 18;
        let temp = 19;

        // Clear destination
        self.emit_subleq(dst, dst, self.next_addr + 3);

        // Copy src2 to counter
        self.emit_copy(src2, counter)?;

        // Loop: while counter > 0, add src1 to dst
        let loop_label = self.next_addr;
        self.emit_subleq(counter, counter, self.next_addr + 3);

        // If counter <= 0, jump to end
        let end_label = self.next_addr + 6;
        self.emit_subleq(counter, temp, end_label);

        // Add src1 to dst
        self.emit_subleq(src1, temp, loop_label);

        // Jump back
        self.emit_subleq(temp, temp, loop_label);

        Ok(())
    }

    fn get_register_addr(&self, reg: Register) -> Result<usize> {
        self.register_map.get(&reg)
            .copied()
            .ok_or_else(|| anyhow!("Invalid register"))
    }

    fn allocate_addr(&mut self) -> usize {
        let addr = self.next_addr;
        self.next_addr += 1;
        addr
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_codegen_create() {
        let codegen = SubleqCodegen::new(1024);
        assert_eq!(codegen.next_addr, 16);
    }

    #[test]
    fn test_emit_subleq() {
        let mut codegen = SubleqCodegen::new(1024);
        codegen.emit_subleq(0, 1, 3);
        assert_eq!(codegen.memory[0], 0);
        assert_eq!(codegen.memory[1], 1);
        assert_eq!(codegen.memory[2], 3);
    }
}

//! Memory Model — Von Neumann unified address space

use anyhow::{Result, anyhow};
use std::fmt;

pub type Address = usize;
pub type Word = i64;

/// Flat 64-bit memory model with code and data unified
#[derive(Clone)]
pub struct Memory {
    cells: Vec<Word>,
    max_addr: Address,
}

impl Memory {
    pub fn new(size: Address) -> Self {
        Self {
            cells: vec![0; size],
            max_addr: size - 1,
        }
    }

    pub fn read(&self, addr: Address) -> Option<Word> {
        if addr <= self.max_addr {
            Some(self.cells[addr])
        } else {
            None
        }
    }

    pub fn write(&mut self, addr: Address, value: Word) -> Result<()> {
        if addr > self.max_addr {
            return Err(anyhow!("Address out of bounds: {} (max {})", addr, self.max_addr));
        }
        self.cells[addr] = value;
        Ok(())
    }

    pub fn size(&self) -> Address {
        self.cells.len()
    }

    pub fn as_slice(&self) -> &[Word] {
        &self.cells
    }

    pub fn as_mut_slice(&mut self) -> &mut [Word] {
        &mut self.cells
    }
}

impl fmt::Debug for Memory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Memory")
            .field("size", &self.cells.len())
            .field("sample", &self.cells.iter().take(10).collect::<Vec<_>>())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_basic() {
        let mut mem = Memory::new(1024);
        mem.write(0, 42).unwrap();
        assert_eq!(mem.read(0), Some(42));
    }

    #[test]
    fn test_memory_bounds() {
        let mem = Memory::new(10);
        assert_eq!(mem.read(5), Some(0));
        assert_eq!(mem.read(15), None);
    }
}

//! The core of the emulator that is responsible for executing
//! RISC-V code.

use crate::{memory::Memory, Architecture, Extension, Instruction};

/// Representation of a single physical CPU.
pub struct Cpu {
    arch: Architecture,
    mem: Memory,
}

impl Cpu {
    /// Create a new CPU.
    pub fn new(arch: Architecture, mem: Memory) -> Self {
        Self { arch, mem }
    }

    /// Perfom one step inside the CPU, that will fetch an instrution, decode it, and then execute
    /// it.
    pub fn step(&mut self) -> Option<()> {
        let pc = self.arch.base.read_register(2.into());
        let inst = self.mem.read::<u32>(pc)?;
        let inst = self.arch.base.parse_instruction(inst)?;
        inst.exec(&mut self.arch.base);

        Some(())
    }

    /// Return a reference to the underyling architecture of this CPU.
    pub fn arch(&mut self) -> &mut Architecture {
        &mut self.arch
    }
}

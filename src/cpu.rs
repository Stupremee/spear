//! The core of the emulator that is responsible for executing
//! RISC-V code.

use crate::{memory::Memory, Address, Architecture, Continuation, Extension, Instruction};
use bytemuck::Pod;
use object::Object;

/// Helper providing mutable access to either a CPU, or a extension inside a CPU.
pub struct CpuOrExtension<'cpu, Ext> {
    cpu: &'cpu mut Cpu,
    get_ext: fn(&mut Cpu) -> &mut Ext,
}

impl<'cpu, Ext> CpuOrExtension<'cpu, Ext> {
    /// Create a new helper that can provide mutable access to either the cpu, or the extension
    /// returned by the closure.
    pub fn new(cpu: &'cpu mut Cpu, get_ext: fn(&mut Cpu) -> &mut Ext) -> Self {
        Self { cpu, get_ext }
    }

    /// Get mutable access to the CPU.
    pub fn cpu(&mut self) -> &mut Cpu {
        self.cpu
    }

    /// Get mutable access to the extension.
    pub fn ext(&mut self) -> &mut Ext {
        (self.get_ext)(self.cpu)
    }
}

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

    /// Create a CPU that will execute the given object file.
    pub fn from_object(object: object::File<'_>) -> object::Result<Self> {
        let mut arch = Architecture::rv32i();

        // set the entry point
        arch.base().set_pc(object.entry().into());

        // load the file into memory
        let mut mem = Memory::new();
        mem.load_object(object)?;

        Ok(Self::new(arch, mem))
    }

    /// Perfom one step inside the CPU, that will fetch an instrution, decode it, and then execute
    /// it.
    pub fn step(&mut self) -> Option<()> {
        let pc = self.arch.base.get_pc();
        let inst = self.mem.read::<u32>(pc)?;
        let inst = self.arch.base.parse_instruction(inst)?;
        let new_pc = pc + inst.len() as u64;

        match inst.exec(self) {
            Continuation::Next => self.arch.base.set_pc(new_pc),
            Continuation::Jump => {}
        }

        Some(())
    }

    /// Read a `T` from the given address.
    pub fn read<T: Pod>(&self, addr: Address) -> Option<T> {
        self.mem.read(addr)
    }

    /// Write a `T` to the given address.
    pub fn write<T: Pod>(&mut self, addr: Address, item: T) -> Option<()> {
        self.mem.write(addr, item)
    }

    /// Return a reference to the underyling architecture of this CPU.
    pub fn arch(&mut self) -> &mut Architecture {
        &mut self.arch
    }

    /// Return a reference to the underyling memory of this CPU.
    pub fn mem(&mut self) -> &mut Memory {
        &mut self.mem
    }
}

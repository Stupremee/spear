//! The core of the emulator that is responsible for executing
//! RISC-V code.

use crate::{
    memory::Memory,
    trap::{Exception, Result},
    Address, Architecture, Continuation, Extension, Instruction,
};
use bytemuck::Pod;
use object::Object;

/// Different privilege modes a CPU core can be in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrivilegeMode {
    /// The highest privilege mode.
    Machine,
    /// The supervisor privilege mode.
    Supervisor,
    /// The user privilege mode.
    User,
}

impl PrivilegeMode {
    /// Get the first two bits from the byte and turn them into a privilege mode
    /// according to the specification.
    ///
    /// # Bits
    ///
    /// - `0b00`: User mode
    /// - `0b01`: Supervisor mode
    /// - `0b11`: Machine mode
    pub fn from_bits(bits: u8) -> Self {
        match bits & 0b11 {
            0b00 => PrivilegeMode::User,
            0b01 => PrivilegeMode::Supervisor,
            0b11 => PrivilegeMode::Machine,
            _ => unreachable!(),
        }
    }

    /// Check if this privilege mode has higher privileges than the given mode.
    pub fn can_access(self, other: PrivilegeMode) -> bool {
        use PrivilegeMode::*;

        match (self, other) {
            (Machine, _) => true,
            (Supervisor, Supervisor | User) => true,
            (User, User) => true,
            _ => false,
        }
    }
}

/// Representation of a single physical CPU.
pub struct Cpu {
    arch: Architecture,
    mem: Memory,
    mode: PrivilegeMode,
}

impl Cpu {
    /// Create a new CPU.
    pub fn new(arch: Architecture, mem: Memory) -> Self {
        Self {
            arch,
            mem,
            mode: PrivilegeMode::Machine,
        }
    }

    /// Create a CPU that will execute the given object file.
    pub fn from_object(object: object::File<'_>) -> object::Result<Self> {
        let mut arch = Architecture::rv32i();

        // set the entry point
        arch.base().set_pc((object.entry() as u32).into());

        // load the file into memory
        let mut mem = Memory::new();
        mem.load_object(object)?;

        Ok(Self::new(arch, mem))
    }

    /// Perfom one step inside the CPU, that will fetch an instrution, decode it, and then execute
    /// it.
    pub fn step(&mut self) -> Result<()> {
        let pc = self.arch.base.get_pc();
        let inst = self.mem.read::<u32>(pc)?;

        // check alignment of instruction
        if u64::from(pc) & 3 != 0 {
            return Err(Exception::InstructionAddressMisaligned);
        }

        let (len, c) = self.parse_and_exec(inst)?;
        let new_pc = pc + len;

        match c {
            Continuation::Next => self.arch.base.set_pc(new_pc),
            Continuation::Jump => {}
        }

        Ok(())
    }

    // FIXME: Write macro or something else to make this better.
    fn parse_and_exec(&mut self, inst: u32) -> Result<(u32, Continuation)> {
        if let Some(inst) = self.arch.base.parse_instruction(inst) {
            Ok((inst.len(), inst.exec(self)?))
        } else if let Some(inst) = self
            .arch
            .zicsr
            .as_ref()
            .and_then(|ext| ext.parse_instruction(inst))
        {
            Ok((inst.len(), inst.exec(self)?))
        } else {
            Err(Exception::IllegalInstruction)
        }
    }

    /// Read a `T` from the given address.
    pub fn read<T: Pod>(&self, addr: Address) -> Result<T> {
        self.mem.read(addr)
    }

    /// Write a `T` to the given address.
    pub fn write<T: Pod>(&mut self, addr: Address, item: T) -> Result<()> {
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

    /// Return the current privilege mode of this CPU.
    pub fn mode(&self) -> PrivilegeMode {
        self.mode
    }
}

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

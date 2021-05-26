//! The core of the emulator that is responsible for executing
//! RISC-V code.

use crate::{
    extensions::zicsr::csr,
    memory::Memory,
    trap::{Exception, Interrupt, Result},
    Address, Architecture, Continuation, Extension, Instruction,
};
use bytemuck::Pod;

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

    /// Turn this mode into the raw representation of bits according to the specification.
    pub fn to_bits(self) -> u8 {
        match self {
            PrivilegeMode::User => 0b00,
            PrivilegeMode::Supervisor => 0b01,
            PrivilegeMode::Machine => 0b11,
        }
    }

    /// Check if this privilege mode has higher privileges than the given mode.
    pub fn can_access(self, other: PrivilegeMode) -> bool {
        use PrivilegeMode::*;

        matches!(
            (self, other),
            (Machine, _) | (Supervisor, Supervisor | User) | (User, User)
        )
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

    /// Perfom one step inside the CPU, that will fetch an instrution, decode it, and then execute
    /// it.
    pub fn step(&mut self) -> Result<()> {
        if let Some(int) = self.check_pending_interrupt() {
            int.take_trap(self);
        }

        let pc = self.arch.base.get_pc();
        let inst = self.mem.read::<u32>(pc)?;
        println!("{:#x?}", u64::from(pc));

        // check alignment of instruction
        if u64::from(pc) & 3 != 0 {
            return Err(Exception::InstructionAddressMisaligned);
        }

        let (len, c) = self.parse_and_exec(inst)?;
        let new_pc = pc + len;

        match c {
            Continuation::Next => self.arch.base.set_pc(new_pc),
            // WFI is implemeted by just executing the `wfi` instruction over and over again.
            // This is really expensive but it's simple and it works
            Continuation::WaitForInterrupt => self.arch.base.set_pc(pc),
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
            Err(Exception::IllegalInstruction(inst as u64))
        }
    }

    fn check_pending_interrupt(&self) -> Option<Exception> {
        let zicsr = self.arch.zicsr.as_ref()?;

        // first, check if interrupts are globally enabled
        let status = match self.mode() {
            PrivilegeMode::Machine => Some((zicsr.force_read_csr(csr::MSTATUS), 3)),
            PrivilegeMode::Supervisor => Some((zicsr.force_read_csr(csr::SSTATUS), 1)),
            _ => None,
        };

        if let Some((status, ie_bit)) = status {
            if !status.get_bit(ie_bit) {
                return None;
            }
        }

        let mut pending = zicsr.force_read_csr(csr::MIE) & zicsr.force_read_csr(csr::MIP);
        println!("{:x?}", pending);
        let (bit, int) = if pending.get_bit(11) {
            // MEIP bit
            (11, Interrupt::MachineExternalInterrupt)
        } else if pending.get_bit(7) {
            // MTIP bit
            (7, Interrupt::MachineTimerInterrupt)
        } else if pending.get_bit(3) {
            // MSIP bit
            (3, Interrupt::MachineSoftwareInterrupt)
        } else if pending.get_bit(9) {
            // SEIP bit
            (9, Interrupt::SupervisorExternalInterrupt)
        } else if pending.get_bit(7) {
            // STIP bit
            (5, Interrupt::SupervisorTimerInterrupt)
        } else if pending.get_bit(1) {
            // SSIP bit
            (1, Interrupt::SupervisorSoftwareInterrupt)
        } else {
            return None;
        };
        pending.set_bit(bit, false);

        Some(Exception::Interrupt(int))
    }

    /// Read a `T` from the given address.
    pub fn read<T: Pod>(&self, addr: Address) -> Result<T> {
        self.mem.read(addr)
    }

    /// Write a `T` to the given address.
    pub fn write<T: Pod>(&mut self, addr: Address, item: T) -> Result<()> {
        self.mem.write(addr, item)
    }

    /// Set the program counter to the given value.
    pub fn set_pc(&mut self, pc: Address) {
        self.arch.base.set_pc(pc);
    }

    /// Update the privilege mode to the given mode.
    pub fn set_mode(&mut self, new: PrivilegeMode) {
        self.mode = new;
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

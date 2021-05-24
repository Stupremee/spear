//! Implementation of handling a trap and all different kinds of exceptions.

use crate::cpu::{Cpu, PrivilegeMode};

/// The result type used for everything that can throw a trap.
pub type Result<T> = std::result::Result<T, Exception>;

/// This enum represents the kind of an exception, and how it should be handled.
#[derive(Debug, Clone, Copy)]
pub enum Trap {
    /// The trap is visible to, and handled by, software running inside the execution
    /// environment.
    Contained,
    /// The trap is a synchronous exception that is an explicit call to the execution
    /// environment requesting an action on behalf of software inside the execution environment.
    Requested,
    /// The trap is handled transparently by the execution environment and execution
    /// resumes normally after the trap is handled.
    Invisible,
    /// The trap represents a fatal failure and causes the execution environment to terminate
    /// execution.
    Fatal,
}

/// All the exception kinds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(missing_docs)]
pub enum Exception {
    InstructionAddressMisaligned,
    InstructionAccessFault,
    IllegalInstruction,
    Breakpoint,
    LoadAddressMisaligned,
    StoreAddressMisaligned,
    LoadAccessFault,
    StoreAccessFault,
    /// An environment call taken from U-mode.
    UserEcall,
    /// An environment call taken from S-mode.
    SupervisorEcall,
    /// An environment call taken from M-mode.
    MachineEcall,
    InstructionPageFault,
    LoadPageFault,
    StorePageFault,
}

impl Exception {
    fn cause(self) -> u64 {
        match self {
            Exception::InstructionAddressMisaligned => 0,
            Exception::InstructionAccessFault => 1,
            Exception::IllegalInstruction => 2,
            Exception::Breakpoint => 3,
            Exception::LoadAddressMisaligned => 4,
            Exception::LoadAccessFault => 5,
            Exception::StoreAddressMisaligned => 6,
            Exception::StoreAccessFault => 7,
            Exception::UserEcall => 8,
            Exception::SupervisorEcall => 9,
            Exception::MachineEcall => 11,
            Exception::InstructionPageFault => 12,
            Exception::LoadPageFault => 13,
            Exception::StorePageFault => 15,
        }
    }

    /// Take this trap according to the exception kind.
    pub fn take_trap(self, cpu: &mut Cpu) -> Trap {
        let zicsr = cpu
            .arch()
            .zicsr
            .as_mut()
            .expect("can not take trap if Zicsr extension is disabled");
        let cause = self.cause();
        let prv_mode = cpu.mode();

        if prv_mode.to_bits() <= PrivilegeMode::Supervisor.to_bits() && () {}

        todo!()
    }
}

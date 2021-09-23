//! Implementation of handling a trap.

#![allow(dead_code)]

use crate::Address;

/// The result type used for everything that can throw a trap.
pub type Result<T> = std::result::Result<T, Exception>;

/// All the interrupt kinds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(missing_docs)]
pub enum Interrupt {
    UserSoftwareInterrupt,
    SupervisorSoftwareInterrupt,
    MachineSoftwareInterrupt,

    UserTimerInterrupt,
    SupervisorTimerInterrupt,
    MachineTimerInterrupt,

    UserExternalInterrupt,
    SupervisorExternalInterrupt,
    MachineExternalInterrupt,
}

impl Interrupt {
    fn cause(self) -> u32 {
        match self {
            Interrupt::UserSoftwareInterrupt => 0,
            Interrupt::SupervisorSoftwareInterrupt => 1,
            Interrupt::MachineSoftwareInterrupt => 3,
            Interrupt::UserTimerInterrupt => 4,
            Interrupt::SupervisorTimerInterrupt => 5,
            Interrupt::MachineTimerInterrupt => 7,
            Interrupt::UserExternalInterrupt => 8,
            Interrupt::SupervisorExternalInterrupt => 9,
            Interrupt::MachineExternalInterrupt => 11,
        }
    }
}

/// All the exception kinds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(missing_docs)]
pub enum Exception {
    InstructionAddressMisaligned(Address),
    InstructionAccessFault,
    IllegalInstruction(u64),
    Breakpoint,
    LoadAddressMisaligned(Address),
    StoreAddressMisaligned(Address),
    LoadAccessFault,
    StoreAccessFault,
    /// An environment call taken from U-mode.
    UserEcall,
    /// An environment call taken from S-mode.
    SupervisorEcall,
    /// An environment call taken from M-mode.
    MachineEcall,
    InstructionPageFault(Address),
    LoadPageFault(Address),
    StorePageFault(Address),

    Interrupt(Interrupt),
}

impl Exception {
    fn cause(self) -> u32 {
        match self {
            Exception::InstructionAddressMisaligned(..) => 0,
            Exception::InstructionAccessFault => 1,
            Exception::IllegalInstruction(..) => 2,
            Exception::Breakpoint => 3,
            Exception::LoadAddressMisaligned(..) => 4,
            Exception::LoadAccessFault => 5,
            Exception::StoreAddressMisaligned(..) => 6,
            Exception::StoreAccessFault => 7,
            Exception::UserEcall => 8,
            Exception::SupervisorEcall => 9,
            Exception::MachineEcall => 11,
            Exception::InstructionPageFault(..) => 12,
            Exception::LoadPageFault(..) => 13,
            Exception::StorePageFault(..) => 15,
            Exception::Interrupt(int) => int.cause(),
        }
    }

    fn trap_value(&self, pc: Address) -> Address {
        match self {
            Exception::InstructionAccessFault
            | Exception::Breakpoint
            | Exception::LoadAccessFault
            | Exception::StoreAccessFault => pc,
            Exception::InstructionPageFault(val)
            | Exception::InstructionAddressMisaligned(val)
            | Exception::LoadAddressMisaligned(val)
            | Exception::StoreAddressMisaligned(val)
            | Exception::LoadPageFault(val)
            | Exception::StorePageFault(val) => *val,
            Exception::IllegalInstruction(val) => (*val).into(),
            _ => Address::zero(),
        }
    }
}

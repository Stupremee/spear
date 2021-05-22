//! Implementation of handling a trap and all different kinds of exceptions.

/// The result type used for everything that can throw a trap.
pub type Result<T> = std::result::Result<T, Exception>;

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

//! todo
#![forbid(unsafe_code)]
#![deny(broken_intra_doc_links, rust_2018_idioms, missing_docs)]

mod address;
pub use address::{Address, AddressKind};

pub mod cpu;
pub mod extensions;
pub mod memory;

/// Outcomes of an instruction execution which influences the behaviour of the CPU after
/// an instruction was executed.
#[derive(Debug, Clone, Copy)]
pub enum Continuation {
    /// The instruction was a jump, thus, the PC is not increased.
    Jump,
    /// This was a "normal" instruction and the PC can be increased normally to point to the next
    /// instruction.
    Next,
}

/// Trait for representing an extension-independent instruction.
#[allow(clippy::len_without_is_empty)]
pub trait Instruction {
    /// The length in bytes of this instruction.
    fn len(&self) -> u32;

    /// Execute this instruction on the given CPU, with the context of the associated extension.
    fn exec(self, cpu: &mut cpu::Cpu) -> Continuation;
}

/// A trait that represents a RISC-V ISA extension.
///
/// This trait allows dynamic changing of the available extensions at runtime.
pub trait Extension {
    /// The instruction type of this extension
    type Inst: Instruction;

    /// Try to parse an instruction present in this extension, from the raw bytes.
    fn parse_instruction(&self, _: u32) -> Option<Self::Inst>;
}

/// An architecture is a collection of RISC-V extensions that are enabled and will
/// be used to run the emulator.
pub struct Architecture {
    pub(crate) base: extensions::rv32i::Extension,
    pub(crate) zicsr: Option<extensions::zicsr::Extension>,
}

impl Architecture {
    /// Create an architecture that uses the RV32I extension.
    pub fn rv32i() -> Self {
        Self {
            base: extensions::rv32i::Extension::new_32bit(),
            zicsr: Some(extensions::zicsr::Extension::new_32bit()),
        }
    }

    /// Return a mutable reference to the base extension.
    pub fn base(&mut self) -> &mut extensions::rv32i::Extension {
        &mut self.base
    }
}

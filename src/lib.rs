//! todo
#![forbid(unsafe_code)]
#![deny(broken_intra_doc_links, rust_2018_idioms, missing_docs)]

mod address;
pub use address::Address;

pub mod cpu;
pub mod extensions;
pub mod memory;

/// Trait for representing an extension-independent instruction.
pub trait Instruction {
    /// Execute this instruction on the given CPU, with the context of the associated extension.
    fn exec(self, cpu: &mut cpu::Cpu);
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
}

impl Architecture {
    /// Create an architecture that uses the RV32I extension.
    pub fn rv32i() -> Self {
        Self {
            base: extensions::rv32i::Extension::new_32bit(),
        }
    }

    /// Return a mutable reference to the base extension.
    pub fn base(&mut self) -> &mut extensions::rv32i::Extension {
        &mut self.base
    }
}

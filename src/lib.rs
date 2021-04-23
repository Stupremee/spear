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
    /// The associated extension for this instruction.
    type Ext: Extension;

    /// Execute this instruction on the given CPU, with the context of the associated extension.
    fn exec(self, ext: &mut Self::Ext, cpu: &mut cpu::Cpu);
}

/// A trait that represents a RISC-V ISA extension.
///
/// This trait allows dynamic changing of the available extensions at runtime.
pub trait Extension {
    /// The instruction type of this extension
    type Inst: Instruction<Ext = Self>;

    /// Try to parse an instruction present in this extension, from the raw bytes.
    fn parse_instruction(&self, _: u32) -> Option<Self::Inst>;
}

/// Represents a RISC-V base extension.
///
/// The difference to a normal [`Extension`] is, that there can only be one active base
/// extension but multiple normal extension.
#[allow(clippy::upper_case_acronyms)]
pub enum BaseExtension {
    /// The RV32I base extension.
    RV32I(extensions::rv32i::Extension),
}

/// An architecture is a collection of RISC-V extensions that are enabled and will
/// be used to run the emulator.
pub struct Architecture {
    base: BaseExtension,
}

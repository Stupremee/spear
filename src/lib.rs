//! A very good RISC-V emulator.
#![forbid(unsafe_code)]
#![deny(rustdoc::broken_intra_doc_links, missing_docs)]

pub mod device;
pub mod instruction;
pub mod trap;

mod address;
pub use address::Address;

/// Defines the base ISA for an RISC-V CPU.
#[derive(Debug)]
pub enum Base {
    /// RV32I Base Integer Instruction Set.
    RV32I,
}

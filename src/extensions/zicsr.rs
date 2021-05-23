//! Implementation of the `Zicsr` extension as specified in chapter 9 in the unprivileged
//! specification.

mod registers;
pub use registers::CsrAddress;

/// Common CSR constants.
pub mod csr {
    pub use super::registers::*;
}

use super::rv32i::IType;
use crate::{
    cpu::{self, PrivilegeMode},
    trap::{Exception, Result},
    Address, Continuation,
};
use derive_more::Display;

/// Number of CSR registers available.
pub const CSR_COUNT: usize = 4096;

/// The Zicsr extension.
#[derive(Debug)]
pub struct Extension {
    csrs: Box<[Address; CSR_COUNT]>,
}

impl Extension {
    /// Create a new `Zicsr` extension that will store 32-bit wide CSR.
    pub fn new_32bit() -> Self {
        Self {
            csrs: Box::new([Address::from(0u32); CSR_COUNT]),
        }
    }

    /// Try to write the given `val` to the CSR with the given index.
    pub fn write_csr(&mut self, idx: usize, val: Address, mode: PrivilegeMode) -> Result<()> {
        let csr = CsrAddress::try_new(idx).ok_or(Exception::IllegalInstruction)?;

        if !csr.writeable_in(mode) {
            return Err(Exception::IllegalInstruction);
        }

        let reg = &mut self.csrs[csr.0];
        assert_eq!(
            std::mem::discriminant(&val.kind()),
            std::mem::discriminant(&reg.kind()),
            "tried to store invalid address kind into CSR"
        );
        *reg = val;

        Ok(())
    }

    /// Try to read a given CSR.
    pub fn read_csr(&mut self, idx: usize, mode: PrivilegeMode) -> Result<Address> {
        let csr = CsrAddress::try_new(idx).ok_or(Exception::IllegalInstruction)?;

        if !csr.readable_in(mode) {
            return Err(Exception::IllegalInstruction);
        }

        Ok(self.csrs[csr.0])
    }
}

impl crate::Extension for Extension {
    type Inst = Instruction;

    fn parse_instruction(&self, inst: u32) -> Option<Self::Inst> {
        parse(inst)
    }
}

/// Top level function for parsing a Zicsr instruction.
pub fn parse(inst: u32) -> Option<Instruction> {
    // the opcode is `0b1110011` for every Zicsr instruction
    if inst & 0x7F != 0b1110011 {
        return None;
    }

    let (funct3, ty) = IType::parse(inst);
    let inst = match funct3 {
        0b001 => Instruction::CSRRW,
        0b010 => Instruction::CSRRS,
        0b011 => Instruction::CSRRC,
        0b101 => Instruction::CSRRWI,
        0b110 => Instruction::CSRRSI,
        0b111 => Instruction::CSRRCI,
        _ => return None,
    };
    Some(inst(ty))
}

/// The instruction type of the Zicsr base extension.
#[derive(Debug, Display)]
#[allow(clippy::upper_case_acronyms)]
#[allow(missing_docs)]
pub enum Instruction {
    #[display(fmt = "csrrw {}", "_0")]
    CSRRW(IType),
    #[display(fmt = "csrrs {}", "_0")]
    CSRRS(IType),
    #[display(fmt = "csrrc {}", "_0")]
    CSRRC(IType),
    #[display(fmt = "csrrwi {}", "_0")]
    CSRRWI(IType),
    #[display(fmt = "csrrsi {}", "_0")]
    CSRRSI(IType),
    #[display(fmt = "csrrci {}", "_0")]
    CSRRCI(IType),
}

impl crate::Instruction for Instruction {
    fn exec(self, cpu: &mut cpu::Cpu) -> Result<Continuation> {
        fn ext(cpu: &mut cpu::Cpu) -> &mut Extension {
            cpu.arch().zicsr.as_mut().unwrap()
        }

        fn base(cpu: &mut cpu::Cpu) -> &mut super::rv32i::Extension {
            cpu.arch().base()
        }

        fn inst<F: FnOnce(Address, Address) -> Address>(
            cpu: &mut cpu::Cpu,
            src: Address,
            op: IType,
            f: F,
        ) -> Result<()> {
            let mode = cpu.mode();
            let ext = ext(cpu);
            let old_csr = ext.read_csr(op.val as usize, mode)?;
            let res = f(src, old_csr);
            ext.write_csr(op.val as usize, res, mode)?;
            base(cpu).write_register(op.rd, old_csr);
            Ok(())
        }

        fn reg_inst<F: FnOnce(Address, Address) -> Address>(
            cpu: &mut cpu::Cpu,
            op: IType,
            f: F,
        ) -> Result<()> {
            let src = base(cpu).read_register(op.rs);
            inst(cpu, src, op, f)?;
            Ok(())
        }

        fn imm_inst<F: FnOnce(Address, Address) -> Address>(
            cpu: &mut cpu::Cpu,
            op: IType,
            f: F,
        ) -> Result<()> {
            let src = u8::from(op.rs) as u32;
            inst(cpu, src.into(), op, f)?;
            Ok(())
        }

        match self {
            Instruction::CSRRW(op) => reg_inst(cpu, op, |src, _| src),
            Instruction::CSRRS(op) => reg_inst(cpu, op, |src, old_csr| src | old_csr),
            Instruction::CSRRC(op) => reg_inst(cpu, op, |src, old_csr| old_csr & !src),

            Instruction::CSRRWI(op) => imm_inst(cpu, op, |src, _| src),
            Instruction::CSRRSI(op) => imm_inst(cpu, op, |src, old_csr| src | old_csr),
            Instruction::CSRRCI(op) => imm_inst(cpu, op, |src, old_csr| old_csr & !src),
        }
        .map(|_| Continuation::Next)
    }

    fn len(&self) -> u32 {
        4
    }
}

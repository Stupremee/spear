//! Implementation of the `Zicsr` extension as specified in chapter 9 in the unprivileged
//! specification.

use super::rv32i::IType;
use crate::{cpu, trap::Result, Address, Continuation};
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
        ) {
            let ext = ext(cpu);
            let old_csr = ext.csrs[op.val as usize];
            let res = f(src, old_csr);
            let reg = &mut ext.csrs[op.val as usize];

            assert_eq!(
                std::mem::discriminant(&res.kind()),
                std::mem::discriminant(&reg.kind()),
                "tried to store invalid address kind into CSR"
            );

            *reg = res;
            base(cpu).write_register(op.rd, old_csr);
        }

        fn reg_inst<F: FnOnce(Address, Address) -> Address>(cpu: &mut cpu::Cpu, op: IType, f: F) {
            let src = base(cpu).read_register(op.rs);
            inst(cpu, src, op, f);
        }

        fn imm_inst<F: FnOnce(Address, Address) -> Address>(cpu: &mut cpu::Cpu, op: IType, f: F) {
            let src = u8::from(op.rs) as u32;
            inst(cpu, src.into(), op, f);
        }

        match self {
            Instruction::CSRRW(op) => reg_inst(cpu, op, |src, _| src),
            Instruction::CSRRS(op) => reg_inst(cpu, op, |src, old_csr| src | old_csr),
            Instruction::CSRRC(op) => reg_inst(cpu, op, |src, old_csr| old_csr & !src),

            Instruction::CSRRWI(op) => imm_inst(cpu, op, |src, _| src),
            Instruction::CSRRSI(op) => imm_inst(cpu, op, |src, old_csr| src | old_csr),
            Instruction::CSRRCI(op) => imm_inst(cpu, op, |src, old_csr| old_csr & !src),
        }

        Ok(Continuation::Next)
    }

    fn len(&self) -> u32 {
        4
    }
}

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
    extensions::rv32i::Register,
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
        let mut this = Self {
            csrs: Box::new([Address::from(0u32); CSR_COUNT]),
        };

        // FIXME: Generate `misa` from architecture
        let misa: u64 = (1 << 30) |
            (1 << 20) | // Extensions[20]: (User mode implemented)
            (1 << 18) | // Extensions[18]: (Supervisor mode implemented)
            (1 << 8); // Extensions[8] (RV32I/64I/128I base ISA)

        this.force_write_csr(csr::MISA, misa.into());
        this
    }

    /// Try to write the given `val` to the CSR with the given index.
    pub fn write_csr(&mut self, csr: CsrAddress, val: Address, mode: PrivilegeMode) -> Result<()> {
        if !csr.writeable_in(mode) {
            return Err(Exception::IllegalInstruction(0));
        }

        match csr {
            csr::SSTATUS => {
                self.csrs[csr::MSTATUS.0] =
                    (self.csrs[csr::MSTATUS.0] & !csr::SSTATUS_MASK) | (val & csr::SSTATUS_MASK)
            }
            csr => {
                let reg = &mut self.csrs[csr.0];
                *reg = reg.to_self_kind(val);
            }
        }

        Ok(())
    }

    /// Try to read a given CSR.
    pub fn read_csr(&self, csr: CsrAddress, mode: PrivilegeMode) -> Result<Address> {
        if !csr.readable_in(mode) {
            return Err(Exception::IllegalInstruction(0));
        }

        match csr {
            csr::SSTATUS => Ok(self.csrs[csr::MSTATUS.0] & csr::SSTATUS_MASK),
            csr::CYCLE => Ok(dbg!(self.csrs[csr.0])),
            csr => Ok(self.csrs[csr.0]),
        }
    }

    /// Write the given `val` to the CSR with the given index, without checking permissions.
    pub fn force_write_csr(&mut self, csr: CsrAddress, val: Address) {
        let reg = &mut self.csrs[csr.0];
        *reg = reg.to_self_kind(val);
    }

    /// Read a given CSR without checking permissions.
    pub fn force_read_csr(&self, csr: CsrAddress) -> Address {
        self.csrs[csr.0]
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
        0b000 if ty.val & 0x1F == 0b10 => match ty.val >> 5 {
            0b00000 => Instruction::URET,
            0b01000 => Instruction::SRET,
            0b11000 => Instruction::MRET,
            _ => return None,
        },
        0b000 if ty.val & 0x1F == 0b101 => match ty.val >> 5 {
            0b01000 => Instruction::WFI,
            _ => return None,
        },
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

    #[display(fmt = "uret")]
    URET(IType),
    #[display(fmt = "sret")]
    SRET(IType),
    #[display(fmt = "mret")]
    MRET(IType),
    #[display(fmt = "wfi")]
    WFI(IType),
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
            write: bool,
        ) -> Result<()> {
            let csr =
                CsrAddress::try_new(op.val as usize).ok_or(Exception::IllegalInstruction(0))?;

            let mode = cpu.mode();
            let ext = ext(cpu);
            let old_csr = ext.read_csr(csr, mode)?;
            let res = f(src, old_csr);
            if write {
                ext.write_csr(csr, res, mode)?;
            }
            println!("writing {:x?} to {}", old_csr, op.rd);
            base(cpu).write_register(op.rd, old_csr);
            Ok(())
        }

        fn reg_inst<F: FnOnce(Address, Address) -> Address>(
            cpu: &mut cpu::Cpu,
            op: IType,
            f: F,
        ) -> Result<()> {
            let src = base(cpu).read_register(op.rs);
            let write = op.rs != Register::ZERO;
            inst(cpu, src, op, f, write)?;
            Ok(())
        }

        fn imm_inst<F: FnOnce(Address, Address) -> Address>(
            cpu: &mut cpu::Cpu,
            op: IType,
            f: F,
        ) -> Result<()> {
            let src = u8::from(op.rs) as u32;
            inst(cpu, src.into(), op, f, true)?;
            Ok(())
        }

        match self {
            Instruction::CSRRW(op) => reg_inst(cpu, op, |src, _| src),
            Instruction::CSRRS(op) => reg_inst(cpu, op, |src, old_csr| src | old_csr),
            Instruction::CSRRC(op) => reg_inst(cpu, op, |src, old_csr| old_csr & !src),

            Instruction::CSRRWI(op) => imm_inst(cpu, op, |src, _| src),
            Instruction::CSRRSI(op) => imm_inst(cpu, op, |src, old_csr| src | old_csr),
            Instruction::CSRRCI(op) => imm_inst(cpu, op, |src, old_csr| old_csr & !src),

            Instruction::WFI(_) => return Ok(Continuation::WaitForInterrupt),
            Instruction::MRET(_) => {
                if cpu.mode() != PrivilegeMode::Machine {
                    return Err(Exception::IllegalInstruction(0));
                }
                let ext = ext(cpu);

                let new_pc = ext.read_csr(csr::MEPC, PrivilegeMode::Machine)?;
                let mut status = ext.read_csr(csr::MSTATUS, PrivilegeMode::Machine)?;

                // extract the MPP field from `mstatus`
                let mpp = status.get_bits(11..=12);

                // set the new MIE field to the MPIE field
                status.set_bit(3, status.get_bit(7));
                // set the MPIE field to 1
                status.set_bit(7, true);
                // set the MPP field to U-mode
                status.set_bits(11..=12, PrivilegeMode::User.to_bits() as u64);

                ext.write_csr(csr::MSTATUS, status, PrivilegeMode::Machine)?;
                cpu.set_pc(new_pc);
                cpu.set_mode(PrivilegeMode::from_bits(u64::from(mpp) as u8));

                return Ok(Continuation::Jump);
            }
            _ => todo!(),
        }
        .map(|_| Continuation::Next)
    }

    fn len(&self) -> u32 {
        4
    }
}

//! Implementation of the `Zicsr` extension as specified in chapter 9 in the unprivileged
//! specification.

mod registers;
pub use registers::CsrAddress;

/// Common CSR constants.
pub mod csr {
    pub use super::registers::*;
}

use super::rv32i::{IType, RType};
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
            // trap when in S-mode and TVM=1
            csr::SATP
                if mode == PrivilegeMode::Supervisor
                    && self.force_read_csr(csr::MSTATUS).get_bit(20) =>
            {
                Err(Exception::IllegalInstruction(0))
            }
            csr => {
                self.force_write_csr(csr, val);
                Ok(())
            }
        }
    }

    /// Try to read a given CSR.
    pub fn read_csr(&self, csr: CsrAddress, mode: PrivilegeMode) -> Result<Address> {
        if !csr.readable_in(mode) {
            return Err(Exception::IllegalInstruction(0));
        }

        match csr {
            // trap when in S-mode and TVM=1
            csr::SATP
                if mode == PrivilegeMode::Supervisor
                    && self.force_read_csr(csr::MSTATUS).get_bit(20) =>
            {
                Err(Exception::IllegalInstruction(0))
            }
            csr => Ok(self.force_read_csr(csr)),
        }
    }

    /// Write the given `val` to the CSR with the given index, without checking permissions.
    pub fn force_write_csr(&mut self, csr: CsrAddress, val: Address) {
        match csr {
            csr::SSTATUS => {
                self.csrs[csr::MSTATUS.0] =
                    (self.csrs[csr::MSTATUS.0] & !csr::SSTATUS_MASK) | (val & csr::SSTATUS_MASK)
            }
            csr::SIE => {
                let mideleg = self.csrs[csr::MIDELEG.0];
                self.csrs[csr::MIE.0] = (self.csrs[csr::MIE.0] & !mideleg) | (val & mideleg);
            }
            csr::SIP => {
                let mask = self.csrs[csr::MIDELEG.0] & (1u32 << 1);
                self.csrs[csr::MIP.0] = (self.csrs[csr::MIP.0] & !mask) | (val & mask);
            }
            csr => {
                let reg = &mut self.csrs[csr.0];
                *reg = reg.to_self_kind(val);
            }
        }
    }

    /// Read a given CSR without checking permissions.
    pub fn force_read_csr(&self, csr: CsrAddress) -> Address {
        match csr {
            csr::SSTATUS => self.csrs[csr::MSTATUS.0] & csr::SSTATUS_MASK,
            csr::SIE => self.csrs[csr::MIE.0] & self.csrs[csr::MIDELEG.0],
            csr::SIP => self.csrs[csr::MIP.0] & self.csrs[csr::MIDELEG.0],
            csr => self.csrs[csr.0],
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
    Some(match funct3 {
        0b000 => {
            let (funct3, funct7, ty) = RType::parse(inst);
            if funct3 != 0b000 {
                return None;
            }

            match u8::from(ty.rs2) {
                0b00010 => match funct7 {
                    0b00000 => Instruction::URET(ty),
                    0b01000 => Instruction::SRET(ty),
                    0b11000 => Instruction::MRET(ty),
                    _ => return None,
                },
                0b00101 if funct7 == 0b0001000 => Instruction::WFI(ty),
                _ if funct7 == 0b0001001 => Instruction::SFENCE_VMA(ty),
                _ => return None,
            }
        }
        0b001 => Instruction::CSRRW(ty),
        0b010 => Instruction::CSRRS(ty),
        0b011 => Instruction::CSRRC(ty),
        0b101 => Instruction::CSRRWI(ty),
        0b110 => Instruction::CSRRSI(ty),
        0b111 => Instruction::CSRRCI(ty),
        _ => return None,
    })
}

/// The instruction type of the Zicsr base extension.
#[derive(Debug, Display)]
#[allow(clippy::upper_case_acronyms, non_camel_case_types)]
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
    URET(RType),
    #[display(fmt = "sret")]
    SRET(RType),
    #[display(fmt = "mret")]
    MRET(RType),
    #[display(fmt = "wfi")]
    WFI(RType),
    #[display(fmt = "sfence.vma {}", _0)]
    SFENCE_VMA(RType),
}

impl crate::Instruction for Instruction {
    fn exec(self, cpu: &mut cpu::Cpu) -> Result<Continuation> {
        fn ext(cpu: &mut cpu::Cpu) -> &mut Extension {
            cpu.arch.zicsr.as_mut().unwrap()
        }

        fn base(cpu: &mut cpu::Cpu) -> &mut super::rv32i::Extension {
            cpu.arch.base()
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

            // it's fine for WFI to be implemented as a no-op
            Instruction::WFI(_) => Ok(()),
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

                let mode = PrivilegeMode::from_bits(u64::from(mpp) as u8);
                cpu.set_mode(mode);
                log::debug!("mret: jumping to {} in {:?} mode", new_pc, mode);

                return Ok(Continuation::Jump);
            }
            Instruction::SRET(_) => {
                // get the required privilege mode for executing this instruction
                let req_prv = if ext(cpu).force_read_csr(csr::MSTATUS).get_bit(22) {
                    PrivilegeMode::Machine
                } else {
                    PrivilegeMode::Supervisor
                };

                if cpu.mode() < req_prv {
                    return Err(Exception::IllegalInstruction(0));
                }

                let ext = ext(cpu);

                let new_pc = ext.read_csr(csr::SEPC, PrivilegeMode::Supervisor)?;
                let mut status = ext.force_read_csr(csr::MSTATUS);

                // extract the SPP field from `sstatus`
                let mpp = status.get_bit(8);

                // set the new SIE field to the SPIE field
                status.set_bit(1, status.get_bit(5));
                // set the SPIE field to 1
                status.set_bit(5, true);
                // set the SPP field to U-mode
                status.set_bit(8, false);

                ext.force_write_csr(csr::MSTATUS, status);
                cpu.set_pc(new_pc);

                let mode = PrivilegeMode::from_bits(u64::from(mpp) as u8);
                cpu.set_mode(mode);
                log::debug!("sret: jumping to {} in {:?} mode", new_pc, mode);

                return Ok(Continuation::Jump);
            }
            Instruction::SFENCE_VMA(_) => {
                // trap when TVM=1 and executing in S-mode
                if cpu.mode() == PrivilegeMode::Supervisor
                    && ext(cpu).force_read_csr(csr::MSTATUS).get_bit(20)
                {
                    return Err(Exception::IllegalInstruction(0));
                }

                log::warn!("tried to execute sfence.vma, which is a nop currently");
                Ok(())
            }
            Instruction::URET(_) => todo!(),
        }
        .map(|_| Continuation::Next)
    }

    fn len(&self) -> u32 {
        4
    }
}

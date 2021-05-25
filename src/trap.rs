//! Implementation of handling a trap and all different kinds of exceptions.

use crate::cpu::{Cpu, CpuOrExtension, PrivilegeMode};
use crate::extensions::zicsr::csr;
use crate::Address;

/// The result type used for everything that can throw a trap.
pub type Result<T> = std::result::Result<T, Exception>;

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

/// All the exception kinds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(missing_docs)]
pub enum Exception {
    InstructionAddressMisaligned,
    InstructionAccessFault,
    IllegalInstruction(u64),
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
    InstructionPageFault(Address),
    LoadPageFault(Address),
    StorePageFault(Address),
}

impl Exception {
    fn cause(self) -> u32 {
        match self {
            Exception::InstructionAddressMisaligned => 0,
            Exception::InstructionAccessFault => 1,
            Exception::IllegalInstruction(..) => 2,
            Exception::Breakpoint => 3,
            Exception::LoadAddressMisaligned => 4,
            Exception::LoadAccessFault => 5,
            Exception::StoreAddressMisaligned => 6,
            Exception::StoreAccessFault => 7,
            Exception::UserEcall => 8,
            Exception::SupervisorEcall => 9,
            Exception::MachineEcall => 11,
            Exception::InstructionPageFault(..) => 12,
            Exception::LoadPageFault(..) => 13,
            Exception::StorePageFault(..) => 15,
        }
    }

    fn trap_value(&self, pc: Address) -> Address {
        match self {
            Exception::InstructionAddressMisaligned
            | Exception::InstructionAccessFault
            | Exception::Breakpoint
            | Exception::LoadAddressMisaligned
            | Exception::LoadAccessFault
            | Exception::StoreAddressMisaligned
            | Exception::StoreAccessFault => pc,
            Exception::InstructionPageFault(val)
            | Exception::LoadPageFault(val)
            | Exception::StorePageFault(val) => *val,
            Exception::IllegalInstruction(val) => (*val).into(),
            _ => Address::from(0u32),
        }
    }

    /// Take this trap according to the exception kind.
    pub fn take_trap(self, cpu: &mut Cpu) -> Trap {
        let pc = cpu.arch().base.get_pc();
        let tval = self.trap_value(pc);
        let prv_mode = cpu.mode();
        let cause = self.cause();

        let mut cpu = CpuOrExtension::new(cpu, |cpu| {
            cpu.arch()
                .zicsr
                .as_mut()
                .expect("can not take trap if Zicsr extension is disabled")
        });
        println!("taking trap: {:x?} epc: {:x?}", self, pc);

        if prv_mode.to_bits() <= PrivilegeMode::Supervisor.to_bits()
            && cpu.ext().force_read_csr(csr::MEDELEG).get_bit(cause)
        {
            // handle the trap in S-mode
            cpu.cpu().set_mode(PrivilegeMode::Supervisor);

            // read the address of the trap handler
            let trap_pc = cpu.ext().force_read_csr(csr::STVEC) >> 1 << 1;
            cpu.cpu().set_pc(trap_pc);

            let ext = cpu.ext();

            // write the old PC into `sepc` register
            ext.force_write_csr(csr::SEPC, pc);
            ext.force_write_csr(csr::SCAUSE, cause.into());
            ext.force_write_csr(csr::STVAL, tval);

            // set the previous SPIE to the value of SIE and set SIE to 0
            let mut status = ext.force_read_csr(csr::SSTATUS);
            status.set_bit(5, status.get_bit(1));
            status.set_bit(1, false);

            // set SPP to 0 if taken from user mode
            match prv_mode {
                PrivilegeMode::User => status.set_bit(8, false),
                PrivilegeMode::Supervisor => status.set_bit(8, true),
                _ => unreachable!(),
            }

            ext.force_write_csr(csr::SSTATUS, status);
        } else {
            // handle the trap in M-mode
            cpu.cpu().set_mode(PrivilegeMode::Machine);

            // read the address of the trap handler
            let trap_pc = cpu.ext().force_read_csr(csr::MTVEC) >> 1 << 1;
            cpu.cpu().set_pc(trap_pc);

            let ext = cpu.ext();

            ext.force_write_csr(csr::MEPC, pc);
            ext.force_write_csr(csr::MCAUSE, cause.into());
            ext.force_write_csr(csr::MTVAL, tval);

            // set the previous MPIE to the value of MIE and set MIE to 0
            let mut status = ext.force_read_csr(csr::MSTATUS);
            status.set_bit(7, status.get_bit(3));
            status.set_bit(3, false);

            // set MPP to the mode which took the trap
            status.set_bits(11..=12, prv_mode.to_bits() as u64);

            ext.force_write_csr(csr::MSTATUS, status);
            println!("{:x?}", ext.force_read_csr(csr::MSTATUS));
        }

        match self {
            Exception::LoadAddressMisaligned
            | Exception::InstructionAddressMisaligned
            | Exception::InstructionAccessFault
            | Exception::LoadAccessFault
            | Exception::StoreAddressMisaligned
            | Exception::StoreAccessFault => Trap::Fatal,

            Exception::Breakpoint
            | Exception::UserEcall
            | Exception::SupervisorEcall
            | Exception::MachineEcall => Trap::Requested,

            Exception::IllegalInstruction(_)
            | Exception::InstructionPageFault(_)
            | Exception::LoadPageFault(_)
            | Exception::StorePageFault(_) => Trap::Invisible,
        }
    }
}

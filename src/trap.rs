//! Implementation of handling a trap and all different kinds of exceptions.

use crate::cpu::{Cpu, CpuOrExtension, PrivilegeMode};
use crate::extensions::zicsr::csr;
use crate::Address;
use log::debug;

/// The result type used for everything that can throw a trap.
pub type Result<T> = std::result::Result<T, Exception>;

/// All the interrupt kinds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(missing_docs)]
pub enum Interrupt {
    UserSoftwareInterrupt,
    SupervisorSoftwareInterrupt,
    MachineSoftwareInterrupt,

    UserTimerInterrupt,
    SupervisorTimerInterrupt,
    MachineTimerInterrupt,

    UserExternalInterrupt,
    SupervisorExternalInterrupt,
    MachineExternalInterrupt,
}

impl Interrupt {
    fn cause(self) -> u32 {
        match self {
            Interrupt::UserSoftwareInterrupt => 0,
            Interrupt::SupervisorSoftwareInterrupt => 1,
            Interrupt::MachineSoftwareInterrupt => 3,
            Interrupt::UserTimerInterrupt => 4,
            Interrupt::SupervisorTimerInterrupt => 5,
            Interrupt::MachineTimerInterrupt => 7,
            Interrupt::UserExternalInterrupt => 8,
            Interrupt::SupervisorExternalInterrupt => 9,
            Interrupt::MachineExternalInterrupt => 11,
        }
    }
}

/// All the exception kinds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(missing_docs)]
pub enum Exception {
    InstructionAddressMisaligned(Address),
    InstructionAccessFault,
    IllegalInstruction(u64),
    Breakpoint,
    LoadAddressMisaligned(Address),
    StoreAddressMisaligned(Address),
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

    Interrupt(Interrupt),
}

impl Exception {
    fn cause(self) -> u32 {
        match self {
            Exception::InstructionAddressMisaligned(..) => 0,
            Exception::InstructionAccessFault => 1,
            Exception::IllegalInstruction(..) => 2,
            Exception::Breakpoint => 3,
            Exception::LoadAddressMisaligned(..) => 4,
            Exception::LoadAccessFault => 5,
            Exception::StoreAddressMisaligned(..) => 6,
            Exception::StoreAccessFault => 7,
            Exception::UserEcall => 8,
            Exception::SupervisorEcall => 9,
            Exception::MachineEcall => 11,
            Exception::InstructionPageFault(..) => 12,
            Exception::LoadPageFault(..) => 13,
            Exception::StorePageFault(..) => 15,
            Exception::Interrupt(int) => int.cause(),
        }
    }

    fn trap_value(&self, pc: Address) -> Address {
        match self {
            Exception::InstructionAccessFault
            | Exception::Breakpoint
            | Exception::LoadAccessFault
            | Exception::StoreAccessFault => pc,
            Exception::InstructionPageFault(val)
            | Exception::InstructionAddressMisaligned(val)
            | Exception::LoadAddressMisaligned(val)
            | Exception::StoreAddressMisaligned(val)
            | Exception::LoadPageFault(val)
            | Exception::StorePageFault(val) => *val,
            Exception::IllegalInstruction(val) => (*val).into(),
            _ => Address::from(0u32),
        }
    }

    /// Take this trap according to the exception kind.
    pub fn take_trap(self, cpu: &mut Cpu) {
        let pc = cpu.arch.base.get_pc();
        let tval = self.trap_value(pc);
        let prv_mode = cpu.mode();
        let cause = self.cause();

        let int_bit = matches!(self, Exception::Interrupt(..)) as u32;
        let int_bit = int_bit << (cpu.arch.xlen - 1);

        let mut cpu = CpuOrExtension::new(cpu, |cpu| {
            cpu.arch
                .zicsr
                .as_mut()
                .expect("can not take trap if Zicsr extension is disabled")
        });

        debug!("taking trap {:x?} from {}", self, pc);

        let deleg = if matches!(self, Exception::Interrupt(_)) {
            csr::MIDELEG
        } else {
            csr::MEDELEG
        };

        if prv_mode.to_bits() <= PrivilegeMode::Supervisor.to_bits()
            && cpu.ext().force_read_csr(deleg).get_bit(cause)
        {
            // handle the trap in S-mode
            cpu.cpu().set_mode(PrivilegeMode::Supervisor);

            // read the address of the trap handler
            let tvec = cpu.ext().force_read_csr(csr::STVEC);
            let trap_pc = tvec >> 1 << 1;
            let vector = match tvec.get_bit(0) {
                true => 4 * cause,
                false => 0,
            };

            cpu.cpu().set_pc(trap_pc + vector);

            let ext = cpu.ext();

            // write the old PC into `sepc` register
            ext.force_write_csr(csr::SEPC, pc);
            ext.force_write_csr(csr::SCAUSE, (cause | int_bit).into());
            ext.force_write_csr(csr::STVAL, tval);

            // set the previous SPIE to the value of SIE and set SIE to 0
            let mut status = ext.force_read_csr(csr::MSTATUS);
            status.set_bit(5, status.get_bit(1));
            status.set_bit(1, false);

            // set SPP to 0 if taken from user mode
            match prv_mode {
                PrivilegeMode::User => status.set_bit(8, false),
                PrivilegeMode::Supervisor => status.set_bit(8, true),
                _ => unreachable!(),
            }

            ext.force_write_csr(csr::MSTATUS, status);
        } else {
            // handle the trap in M-mode
            cpu.cpu().set_mode(PrivilegeMode::Machine);

            // read the address of the trap handler
            let tvec = cpu.ext().force_read_csr(csr::MTVEC);
            let trap_pc = tvec >> 1 << 1;
            let vector = match tvec.get_bit(0) {
                true => 4 * cause,
                false => 0,
            };

            cpu.cpu().set_pc(trap_pc + vector);

            let ext = cpu.ext();

            ext.force_write_csr(csr::MEPC, pc);
            ext.force_write_csr(csr::MCAUSE, (cause | int_bit).into());
            ext.force_write_csr(csr::MTVAL, tval);

            // set the previous MPIE to the value of MIE and set MIE to 0
            let mut status = ext.force_read_csr(csr::MSTATUS);
            status.set_bit(7, status.get_bit(3));
            status.set_bit(3, false);

            // set MPP to the mode which took the trap
            status.set_bits(11..=12, prv_mode.to_bits() as u64);

            ext.force_write_csr(csr::MSTATUS, status);
        }
    }
}

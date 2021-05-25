use crate::cpu::PrivilegeMode;

macro_rules! csr_addresses {
    ($(
        $(#[$attr:meta])*
        $name:ident = $num:expr
    ),*$(,)?) => {
        $(
            $(#[$attr])*
            #[allow(missing_docs)]
            pub const $name: CsrAddress = CsrAddress($num);
        )*
    };
}

/// An index into the list of CSR inside the Zicsr extension.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(transparent)]
pub struct CsrAddress(pub(super) usize);

impl CsrAddress {
    /// Try to create a new [`CsrAddress`]. Returns `None` if the given index is out of bounds.
    pub fn try_new(idx: usize) -> Option<Self> {
        (idx < super::CSR_COUNT).then(|| Self(idx))
    }

    /// Check if this CSR can be read from the given privilege mode.
    pub fn readable_in(self, mode: PrivilegeMode) -> bool {
        // get the lowest privilege level that can access the CSR
        let x = (self.0 >> 8) & 0b11;
        let x = PrivilegeMode::from_bits(x as u8);
        mode.can_access(x)
    }

    /// Check if this CSR can be written to from the given privilege mode.
    pub fn writeable_in(self, mode: PrivilegeMode) -> bool {
        // checl if this CSR is read only
        if self.0 >> 10 == 0b11 {
            return false;
        }

        // get the lowest privilege level that can access the CSR
        let x = (self.0 >> 8) & 0b11;
        let x = PrivilegeMode::from_bits(x as u8);
        mode.can_access(x)
    }
}

/// This bit mask can be applied to the MSTATUS register, to get all bits that are valid in the
/// SSTATUS register.
pub const SSTATUS_MASK: u64 = 0x80000003000de162;

#[rustfmt::skip]
csr_addresses![
    /// User status register.
    USTATUS = 0x000,
    /// User interrupt-enable register.
    UIE = 0x004,
    /// User trap handler base address.
    UTVEC = 0x005,
        
    /// Scratch register for user trap handlers.
    USCRATCH = 0x040,
    /// User exception program counter.
    UEPC = 0x041,
    /// User trap cause.
    UCAUSE = 0x042,
    /// User bad address or instruction.
    UTVAL = 0x043,
    /// User interrupt pending.
    UIP = 0x044,

    /// Floating-Point Accrued Exceptions.
    FFLAGS = 0x001,
    /// Floating-Point Dynamic Rounding Mode.
    FRM = 0x002,
    /// Floating-Point Control and Status Register (frm+fflags).
    FCSR = 0x003,

    /// Cycle counter for RDCYCLE instruction.
    CYCLE = 0xC00,
    /// Timer for RDTIME instruction.
    TIME = 0xC01,
    /// Instructions-retired counter for RDINSTRET instruction.
    INSTRET = 0xC02,

    /// Performance-monitoring counter.
    HPMCOUNTER3 = 0xC03,
    /// Performance-monitoring counter.
    HPMCOUNTER4 = 0xC04,
    /// Performance-monitoring counter.
    HPMCOUNTER31 = 0xC1F,

    /// Upper 32 bits of cycle, RV32I only.
    CYCLEH = 0xC80,
    /// Upper 32 bits of time, RV32I only.
    TIMEH = 0xC81,
    /// Upper 32 bits of instret, RV32I only.
    INSTRETH = 0xC82,

    /// Upper 32 bits of hpmcounter3, RV32I only.
    HPMCOUNTER3H = 0xC83,
    /// Upper 32 bits of hpmcounter4, RV32I only.
    HPMCOUNTER4H = 0xC84,
    /// Upper 32 bits of hpmcounter31, RV32I only.
    HPMCOUNTER31H = 0xC9F,

    /// Supervisor status register.
    SSTATUS = 0x100,
    /// Supervisor exception delegation register.
    SEDELEG = 0x102,
    /// Supervisor interrupt delegation register.
    SIDELEG = 0x103,
    /// Supervisor interrupt-enable register.
    SIE = 0x104,
    /// Supervisor trap handler base address.
    STVEC = 0x105,
    /// Supervisor counter enable.
    SCOUNTEREN = 0x106,

    /// Scratch register for supervisor trap handlers.
    SSCRATCH = 0x140,
    /// Supervisor exception program counter.
    SEPC = 0x141,
    /// Supervisor trap cause.
    SCAUSE = 0x142,
    /// Supervisor bad address or instruction.
    STVAL = 0x143,
    /// Supervisor interrupt pending.
    SIP = 0x144,

    /// Supervisor address translation and protection.        
    SATP = 0x180,


    /// Vendor ID.
    MVENDORID = 0xF11,
    /// Architecture ID.
    MARCHID = 0xF12,
    /// Implementation ID.
    MIMPID = 0xF13,
    /// Hardware thread ID.
    MHARTID = 0xF14,

    /// Machine status register.
    MSTATUS = 0x300,
    /// ISA and extensions
    MISA = 0x301,
    /// Machine exception delegation register.
    MEDELEG = 0x302,
    /// Machine interrupt delegation register.
    MIDELEG = 0x303,
    /// Machine interrupt-enable register.
    MIE = 0x304,
    /// Machine trap-handler base address.
    MTVEC = 0x305,
    /// Machine counter enable.
    MCOUNTEREN = 0x306,

    /// Scratch register for machine trap handlers.
    MSCRATCH = 0x340,
    /// Machine exception program counter.
    MEPC = 0x341,
    /// Machine trap cause.
    MCAUSE = 0x342,
    /// Machine bad address or instruction.
    MTVAL = 0x343,
    /// Machine interrupt pending.
    MIP = 0x344,

    /// Physical memory protection configuration.
    PMPCFG0 = 0x3A0,
    /// Physical memory protection configuration, RV32 only.
    PMPCFG1 = 0x3A1,
    /// Physical memory protection configuration.
    PMPCFG2 = 0x3A2,
    /// Physical memory protection configuration, RV32 only.
    PMPCFG3 = 0x3A3,
    /// Physical memory protection address register.
    PMPADDR0 = 0x3B0,
    /// Physical memory protection address register.
    PMPADDR1 = 0x3B1,
    /// Physical memory protection address register.
    PMPADDR2 = 0x3B2,
    /// Physical memory protection address register.
    PMPADDR3 = 0x3B3,
    /// Physical memory protection address register.
    PMPADDR4 = 0x3B4,
    /// Physical memory protection address register.
    PMPADDR5 = 0x3B5,
    /// Physical memory protection address register.
    PMPADDR6 = 0x3B6,
    /// Physical memory protection address register.
    PMPADDR7 = 0x3B7,
    /// Physical memory protection address register.
    PMPADDR8 = 0x3B8,
    /// Physical memory protection address register.
    PMPADDR9 = 0x3B9,
    /// Physical memory protection address register.
    PMPADDR10 = 0x3BA,
    /// Physical memory protection address register.
    PMPADDR11 = 0x3BB,
    /// Physical memory protection address register.
    PMPADDR12 = 0x3BC,
    /// Physical memory protection address register.
    PMPADDR13 = 0x3BD,
    /// Physical memory protection address register.
    PMPADDR14 = 0x3BE,
    /// Physical memory protection address register.
    PMPADDR15 = 0x3BF,
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_readable() {
        let csr = CsrAddress(0x300);
        assert!(csr.readable_in(PrivilegeMode::Machine));
        assert!(!csr.readable_in(PrivilegeMode::Supervisor));
        assert!(!csr.readable_in(PrivilegeMode::User));

        let csr = CsrAddress(0x9FF);
        assert!(csr.readable_in(PrivilegeMode::Machine));
        assert!(csr.readable_in(PrivilegeMode::Supervisor));
        assert!(!csr.readable_in(PrivilegeMode::User));

        let csr = CsrAddress(0xC7F);
        assert!(csr.readable_in(PrivilegeMode::Machine));
        assert!(csr.readable_in(PrivilegeMode::Supervisor));
        assert!(csr.readable_in(PrivilegeMode::User));
    }

    #[test]
    fn check_writeable() {
        let csr = CsrAddress(0x300);
        assert!(csr.writeable_in(PrivilegeMode::Machine));
        assert!(!csr.writeable_in(PrivilegeMode::Supervisor));
        assert!(!csr.writeable_in(PrivilegeMode::User));

        let csr = CsrAddress(0xF11);
        assert!(!csr.writeable_in(PrivilegeMode::Machine));
        assert!(!csr.writeable_in(PrivilegeMode::Supervisor));
        assert!(!csr.writeable_in(PrivilegeMode::User));

        let csr = CsrAddress(0x5BF);
        assert!(csr.writeable_in(PrivilegeMode::Machine));
        assert!(csr.writeable_in(PrivilegeMode::Supervisor));
        assert!(!csr.writeable_in(PrivilegeMode::User));

        let csr = CsrAddress(0x4FF);
        assert!(csr.writeable_in(PrivilegeMode::Machine));
        assert!(csr.writeable_in(PrivilegeMode::Supervisor));
        assert!(csr.writeable_in(PrivilegeMode::User));

        let csr = CsrAddress(0xCFF);
        assert!(!csr.writeable_in(PrivilegeMode::Machine));
        assert!(!csr.writeable_in(PrivilegeMode::Supervisor));
        assert!(!csr.writeable_in(PrivilegeMode::User));
    }
}

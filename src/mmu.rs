//! Implementation of the Memory Management Unit which is used for virtual memory.

use crate::{
    cpu::{Cpu, PrivilegeMode},
    extensions::zicsr::csr,
    trap::Result,
    Address, Architecture,
};

/// Constant that will shift away all offset bits from a virtual address.
pub const PAGE_SHIFT: u32 = 12;

/// Different access types that memory can be accessed for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccessType {
    /// Read memory
    Read,
    /// Write to memory
    Write,
    /// Fetch an instruction
    Fetch,
}

/// The memory management unit of a RISC-V hart.
pub struct Mmu {}

impl Mmu {
    /// Create a new Memory Management Unit.
    pub fn new() -> Self {
        Self {}
    }

    /// Translate a physical address to a virtual address, by walking the page table found
    /// in the `xATP` register using the given access type.
    pub fn translate(&self, cpu: &Cpu, addr: Address, mode: AccessType) -> Result<Address> {
        let csr = match cpu.arch.zicsr.as_ref() {
            Some(csr) => csr,
            None => return Ok(addr),
        };
        let mstatus = csr.force_read_csr(csr::MSTATUS);
        let mxr = mstatus.get_bit(19);
        let mut prv = cpu.mode();

        // if the access mode is read or write, and the MPRV bit is set, modify the privilege mode
        // to the privilege mode to the mode in the MPP register
        if mode != AccessType::Fetch && mstatus.get_bit(17) {
            let mpp = mstatus.get_bits(11..=12);
            prv = PrivilegeMode::from_bits(u64::from(mpp) as u8);
        }

        // now walk the page tables to translate the address
        let satp = csr.force_read_csr(csr::SATP);
        let info = match decode_satp(satp, prv, &cpu.arch) {
            Some(info) => info,
            None => return Ok(addr),
        };

        for lvl in (0..info.levels).rev() {
            // get the bits to shift for this VPN index
            let ptshift = lvl * info.idxbits;
            let idx = (addr >> (PAGE_SHIFT + ptshift)) & ((1u64 << info.idxbits) - 1);

            let pte = match info.ptesize {
                4 => cpu.mem.read::<u32>(addr)?,
                _ => unreachable!(),
            };
        }

        todo!()
    }
}

struct VirtualInfo {
    levels: u32,
    idxbits: u32,
    widenbits: u32,
    ptesize: u32,
    ptbase: Address,
}

impl VirtualInfo {
    fn new(levels: u32, idxbits: u32, widenbits: u32, ptesize: u32, ptbase: Address) -> Self {
        Self {
            levels,
            idxbits,
            widenbits,
            ptesize,
            ptbase,
        }
    }
}

fn decode_satp(satp: Address, prv: PrivilegeMode, arch: &Architecture) -> Option<VirtualInfo> {
    const PPN32_MASK: u32 = 0x003FFFFF;

    match prv {
        PrivilegeMode::Machine => None,
        PrivilegeMode::Supervisor | PrivilegeMode::User if arch.xlen == 32 => {
            // read the MODE bit from the satp
            let mode = u64::from(satp & (1u32 << 31));
            match mode {
                0 => None,
                1 => Some(VirtualInfo::new(
                    2,
                    10,
                    0,
                    4,
                    (satp & PPN32_MASK) << PAGE_SHIFT,
                )),
                _ => unreachable!(),
            }
        }
        PrivilegeMode::Supervisor | PrivilegeMode::User if arch.xlen == 64 => {
            todo!()
        }
        _ => unreachable!(),
    }
}

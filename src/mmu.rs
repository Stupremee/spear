//! Implementation of the Memory Management Unit which is used for virtual memory.

use crate::{
    cpu::{Cpu, PrivilegeMode},
    extensions::zicsr::csr,
    trap::{Exception, Result},
    Address, Architecture,
};

/// Constant that will shift away all offset bits from a virtual address.
pub const PAGE_SHIFT: u32 = 12;

/// The amount of bits to shift to get the PPN from a PTE.
pub const PPN_SHIFT: u32 = 10;

const BIT_V: u64 = 0x01;
const BIT_R: u64 = 0x02;
const BIT_W: u64 = 0x04;
const BIT_X: u64 = 0x08;
const BIT_U: u64 = 0x10;
const BIT_G: u64 = 0x20;
const BIT_A: u64 = 0x40;
const BIT_D: u64 = 0x80;

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
        fn error_for_mode(addr: Address, mode: AccessType) -> Exception {
            match mode {
                AccessType::Read => Exception::LoadPageFault(addr),
                AccessType::Write => Exception::StorePageFault(addr),
                AccessType::Fetch => Exception::InstructionPageFault(addr),
            }
        }

        let csr = match cpu.arch.zicsr.as_ref() {
            Some(csr) => csr,
            None => return Ok(addr),
        };
        let mstatus = csr.force_read_csr(csr::MSTATUS);
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

        let mut base = info.ptbase;
        for lvl in (0..info.levels).rev() {
            // get the bits to shift to get the current VPN
            let ptshift = lvl * info.idxbits;

            // get the virtual page number
            let idx = (addr >> (PAGE_SHIFT + ptshift)) & ((1u64 << info.idxbits) - 1);

            // read the PTE from the current page table
            let pte = match info.ptesize {
                4 => cpu
                    .mem
                    .read::<u32>(base + idx * info.ptesize)
                    .map_err(|_| error_for_mode(addr, mode))? as u64,
                _ => unreachable!(),
            };

            match Entry::from_raw(pte) {
                // if this PTE points to the next table, update the base pointer
                // and continue
                Entry::Branch(next) => base = next,
                // found an address to translate
                Entry::Leaf(phys) => {
                    // check if the PTE has the correct permissions
                    // FIXME: `SUM` and `MXR`
                    let valid = match mode {
                        AccessType::Read => pte & BIT_R != 0,
                        AccessType::Write => pte & BIT_W != 0,
                        AccessType::Fetch => pte & BIT_X != 0,
                    };

                    return valid
                        .then(|| {
                            // add the offset inside the page and return the physical address
                            let off = addr & 0xFFFu32;
                            phys + off
                        })
                        .ok_or_else(|| error_for_mode(addr, mode));
                }
                // entry is invalid, throw exception
                Entry::Invalid => break,
            }
        }

        Err(error_for_mode(addr, mode))
    }
}

/// Representation of a page table entry.
pub enum Entry {
    /// This entry points to the next page table level.
    Branch(Address),
    /// Translation was successful and the physical address was found.
    Leaf(Address),
    /// This is entry is not valid, thus should throw an exception.
    Invalid,
}

impl Entry {
    /// Convert a raw PTE into a parsed version.
    pub fn from_raw(x: u64) -> Entry {
        if x & (BIT_V | BIT_R | BIT_W | BIT_X) == BIT_V {
            Entry::Branch(Address::from(x >> PPN_SHIFT << PAGE_SHIFT))
        } else if x & BIT_V == BIT_V {
            Entry::Leaf(Address::from(x >> PPN_SHIFT << PAGE_SHIFT))
        } else {
            Entry::Invalid
        }
    }
}

/// Structure for describing the currently enabled virtual memory mode.
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
                _ => Some(VirtualInfo::new(
                    2,
                    10,
                    0,
                    4,
                    (satp & PPN32_MASK) << PAGE_SHIFT,
                )),
            }
        }
        PrivilegeMode::Supervisor | PrivilegeMode::User if arch.xlen == 64 => {
            todo!()
        }
        _ => unreachable!(),
    }
}

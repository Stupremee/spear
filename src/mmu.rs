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
// const BIT_G: u64 = 0x20;
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
        let error = match mode {
            AccessType::Read => Exception::LoadPageFault(addr),
            AccessType::Write => Exception::StorePageFault(addr),
            AccessType::Fetch => Exception::InstructionPageFault(addr),
        };

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
        let (mut table, info) = match decode_satp(satp, prv, &cpu.arch) {
            Some(info) => info,
            None => return Ok(addr),
        };

        // loop through each level, starting from the highest level
        for lvl in (0..info.levels()).rev() {
            // get the VPN for the current level
            let vpn = info.vpn(addr, lvl);

            // read the PTE at the current VPN
            let pte = info.read_entry(cpu, table, vpn)?.ok_or(error)?;

            match pte {
                Entry::Branch(next) => {
                    // go to the next page table level
                    table = next;
                    continue;
                }
                Entry::Leaf { ppn, entry } => {
                    let bit_u = entry & BIT_U != 0;
                    let bit_r = entry & BIT_R != 0;
                    let bit_w = entry & BIT_W != 0;
                    let bit_x = entry & BIT_X != 0;
                    let bit_a = entry & BIT_A != 0;
                    let bit_d = entry & BIT_D != 0;

                    let mxr = mstatus.get_bit(19);

                    // compare permissions of the PTE with the access type
                    #[rustfmt::skip]
                    let mut allowed = match (mode, bit_r as u8, bit_w as u8, bit_x as u8) {
                        (AccessType::Read,    1, _, _)
                        | (AccessType::Write, _, 1, _)
                        | (AccessType::Fetch, _, _, 1) => true,

                        // read is allowed if R=0, X=1 and MXR=1
                        (AccessType::Read,    0, _, 1) if mxr => true,
                        _ => false,
                    };

                    // check if S-mode is permitted to access a U-page
                    let sum = mstatus.get_bit(18);
                    if bit_u
                        && prv == PrivilegeMode::Supervisor
                        && (mode == AccessType::Fetch || !sum)
                    {
                        allowed = false;
                    }

                    // check if U-mode tries to access without U-bit
                    if prv == PrivilegeMode::User && !bit_u {
                        allowed = false;
                    }

                    // If A=0, or if the memory access is a store and D=0 raise an exception
                    if (bit_a || mode == AccessType::Write) && !bit_d {
                        allowed = false;
                    }

                    if !allowed {
                        return Err(error);
                    }

                    // FIXME: step 6, check if the last VPN is 0

                    // calculate the physical address and return it
                    return Ok(ppn + info.offset_of(addr, lvl));
                }
            }
        }

        Err(error)
    }

    /// Check if the access of `len` bytes at `addr` is valid according the the PMP.
    fn check_pmp(addr: Address, len: usize, mode: AccessType, prv: PrivilegeMode) -> bool {
        true
    }
}

/// Representation of a page table entry.
enum Entry {
    /// This entry points to the next page table level.
    Branch(Address),
    /// Translation was successful and the physical address was found.
    Leaf { ppn: Address, entry: u64 },
}

impl Entry {
    /// Parse a raw PTE into a better representation.
    pub fn parse(x: u64) -> Option<Entry> {
        let bit_v = x & BIT_V != 0;
        let bit_r = x & BIT_R != 0;
        let bit_w = x & BIT_W != 0;
        let bit_x = x & BIT_X != 0;

        // check if this entry is valid
        if !bit_v || (!bit_r && bit_w) {
            return None;
        }

        let ppn = Address::from(x) >> 10 << PAGE_SHIFT;

        // check if this PTE is a leaf
        if bit_r || bit_x {
            return Some(Entry::Leaf { ppn, entry: x });
        }

        // this PTE is a branch to the next level
        Some(Entry::Branch(ppn))
    }
}

/// Represents the different paging modes.
enum PagingMode {
    Sv32,
}

impl PagingMode {
    /// Return the amount of page table levels this paging mode has.
    const fn levels(&self) -> usize {
        match self {
            PagingMode::Sv32 => 2,
        }
    }

    /// Get the Virtual Page Number with the given index from the given virtual address
    fn vpn(&self, addr: Address, idx: usize) -> u32 {
        match (self, idx) {
            (PagingMode::Sv32, 0) => (u32::from(addr) >> PAGE_SHIFT) & 0x3FF,
            (PagingMode::Sv32, 1) => (u32::from(addr) >> PAGE_SHIFT >> 10) & 0x3FF,
            (PagingMode::Sv32, _) => unreachable!(),
        }
    }

    /// Read a PTE from the given address.
    fn read_entry(&self, cpu: &Cpu, table: Address, vpn: u32) -> Result<Option<Entry>> {
        Ok(match self {
            PagingMode::Sv32 => Entry::parse(cpu.bus.read::<u32>(table + vpn * 4)? as u64),
        })
    }

    /// Get the pageoffset of a virtual address, at the given level.
    fn offset_of(&self, addr: Address, lvl: usize) -> u32 {
        match (self, lvl) {
            (PagingMode::Sv32, 0) => u32::from(addr & 0x3FFu32),
            (PagingMode::Sv32, 1) => u32::from(addr & 0x3FFFFFu32),
            (PagingMode::Sv32, _) => unreachable!(),
        }
    }
}

fn decode_satp(
    satp: Address,
    prv: PrivilegeMode,
    arch: &Architecture,
) -> Option<(Address, PagingMode)> {
    match prv {
        PrivilegeMode::Machine => None,
        PrivilegeMode::Supervisor | PrivilegeMode::User if arch.xlen == 32 => {
            // read the MODE bit from the satp, and return Sv32 if it's 1
            let mode = u64::from(satp & (1u32 << 31));
            (mode != 0).then(|| ((satp & 0x003FFFFFu32) << PAGE_SHIFT, PagingMode::Sv32))
        }
        _ => todo!(),
    }
}

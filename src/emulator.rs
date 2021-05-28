//! This module implements abstractions for loading, executing and debugging RISC-V code
//! using the spear emulator.

use crate::{cpu::Cpu, memory::Memory, Address, Architecture};
use object::{Object, ObjectSymbol};

/// The `Emulator` is responsible for loading, initializing and running a [`Cpu`].
pub struct Emulator {
    cpu: Cpu,
    tohost_addr: Option<Address>,
}

impl Emulator {
    /// Create an emulator that will execute the given object file.
    pub fn from_object(object: object::File<'_>) -> object::Result<Self> {
        let mut arch = Architecture::rv32i();

        // set the entry point
        arch.base().set_pc((object.entry() as u32).into());

        // load the file into memory
        let mut mem = Memory::new();
        mem.load_object(object)?;

        Ok(Emulator {
            cpu: Cpu::new(arch, mem),
            tohost_addr: None,
        })
    }

    /// Create an emulator that will execute the given object file, which also supports the HTIF
    /// (including `.tohost` and `.fromhost`) behavior.
    pub fn from_object_with_htif(object: object::File<'_>) -> object::Result<Self> {
        let tohost_addr = object
            .symbols()
            .find(|s| s.name().map_or(false, |n| n == "tohost"))
            .map(|sym| Address::from(sym.address()));

        let mut emu = Self::from_object(object)?;
        emu.tohost_addr = tohost_addr;

        Ok(emu)
    }

    /// Run this emulator until a `Fatal` trap gets hit or something is written to `tohost`.
    pub fn run(&mut self) {
        // we set `tohost` to zero at the beginning, to see every write that occurrs.
        if let Some(addr) = self.tohost_addr {
            let _ = self.cpu.write::<u32>(addr, 0u32);
        }

        loop {
            if let Some(addr) = self.tohost_addr {
                if self.cpu.read::<u32>(addr).map_or(false, |x| x != 0) {
                    return;
                }
            }

            match self.cpu.step() {
                Ok(_) => {}
                Err(trap) => trap.take_trap(&mut self.cpu),
            }
        }
    }

    /// Get a mutable reference to the underlying CPU.
    pub fn cpu(&mut self) -> &mut Cpu {
        &mut self.cpu
    }
}

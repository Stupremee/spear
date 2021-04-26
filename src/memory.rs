//! Physical memory management.
//!
//! The main type of this module is the [`Memory`] struct, which contains
//! a list of [`MemoryDevices`]s that are used to read, and write raw memory.

use super::Address;
use bytemuck::{bytes_of, bytes_of_mut, Pod};
use object::{File, Object, ObjectSection};
use std::collections::BTreeMap;
use std::mem::align_of;

/// Anything that can be used to access memory, including RAM and MMIO devices.
///
/// Any device must specify the size it covers using the `size()` method, but it can not control
/// the base address, since that will be done by the user.
pub trait MemoryDevice {
    /// The number of bytes this memory device covers, starting from the base address.
    fn size(&self) -> u64;

    /// Fill `buf` with bytes at the given address.
    ///
    /// Note that the address is a relativ offset to the base address of this device.
    ///
    /// # Returns
    ///
    /// `true` if the load was successful and the **whole** buffer is filled. Not filling the buffer,
    /// but returning `true`, will be the same behaviour as filling the buffer with zeros.
    fn load(&self, addr: Address, buf: &mut [u8]) -> bool;

    /// Write `buf`s contents to the given address.
    ///
    /// Note that the address is a relativ offset to the base address of this device.
    ///
    /// # Returns
    ///
    /// `true` if the write was successful and the **whole** buffer was written.
    /// Not writing the whole buffer, might lead to logic bugs.
    fn write(&mut self, addr: Address, buf: &[u8]) -> bool;
}

/// The main struct of this module, which acts as a memory bus combining multiple
/// [`MemoryDevice`]s into one spot.
#[derive(Default)]
pub struct Memory {
    devices: BTreeMap<Address, Box<dyn MemoryDevice>>,
}

impl Memory {
    /// Create a new memory bus without any devices.
    pub fn new() -> Self {
        Self::default()
    }

    /// Load an object file that was previously parsed by the [`object`] crate.
    ///
    /// This method will create multiple RAM devices into this memory bus.
    pub fn load_object(&mut self, obj: File<'_>) -> object::Result<()> {
        assert!(obj.is_little_endian(), "Big Endian not supported");

        // go through each section that is not at address zero and has no zero size
        for section in obj
            .sections()
            .filter(|sec| sec.size() != 0 && sec.address() != 64)
        {
            let dev = RamDevice::from_vec(section.data()?.to_vec());
            self.add_device(section.address().into(), dev);
        }

        Ok(())
    }

    /// Add a new device to this memory bus, that starts at the `base` address.
    pub fn add_device(&mut self, base: Address, dev: impl MemoryDevice + 'static) {
        // TODO: check overlap of addresses here
        self.devices.insert(base, Box::new(dev));
    }

    /// Read a `T` from the given address.
    ///
    /// # Returns
    ///
    /// `None` if the read failed.
    pub fn read<T: Pod>(&self, addr: Address) -> Option<T> {
        // check alignment of the address
        if u64::from(addr) & (align_of::<T>() as u64 - 1) != 0 {
            return None;
        }

        // find the device that has the smallest, positive distance
        // from the requested address
        let (&offset, device) = self.devices.iter().find(|(&k, v)| {
            let base = u64::from(k);
            let end = base + v.size();
            let addr = u64::from(addr);

            base <= addr && addr < end
        })?;

        // create a zeroed `T` to read into
        let mut item = T::zeroed();
        device
            .load(addr - offset, bytes_of_mut(&mut item))
            .then(|| item)
    }

    /// Write a `T` to the given address.
    ///
    /// # Returns
    ///
    /// `None` if the read failed, which may be caused by unaligned address,
    /// no physical memory for `addr` and others.
    pub fn write<T: Pod>(&mut self, addr: Address, item: T) -> Option<()> {
        // check alignment of the address
        if u64::from(addr) & (align_of::<T>() as u64 - 1) != 0 {
            return None;
        }

        // find the first device that contains the given address
        let (&offset, device) = self.devices.iter_mut().find(|(&k, v)| {
            let base = u64::from(k);
            let end = base + v.size();
            let addr = u64::from(addr);

            base <= addr && addr < end
        })?;

        // write the item into the device
        device.write(addr - offset, bytes_of(&item)).then(|| ())
    }
}

/// A [`MemoryDevice`] which acts as a RAM module containing a fixed buffer of memory.
pub struct RamDevice {
    ram: Box<[u8]>,
}

impl RamDevice {
    /// Create a new RAM device that is able to hold `size` bytes of memory.
    pub fn new(size: usize) -> Self {
        Self {
            ram: vec![0u8; size].into_boxed_slice(),
        }
    }

    /// Create a RAM device that is initialized using the given vec.
    pub fn from_vec(vec: Vec<u8>) -> Self {
        Self {
            ram: vec.into_boxed_slice(),
        }
    }
}

impl MemoryDevice for RamDevice {
    fn size(&self) -> u64 {
        self.ram.len() as u64
    }

    fn load(&self, addr: Address, buf: &mut [u8]) -> bool {
        let addr = u64::from(addr) as usize;
        if let Some(from) = self.ram.get(addr..addr + buf.len()) {
            buf.copy_from_slice(from);
            true
        } else {
            false
        }
    }

    fn write(&mut self, addr: Address, buf: &[u8]) -> bool {
        let addr = u64::from(addr) as usize;
        if let Some(to) = self.ram.get_mut(addr..addr + buf.len()) {
            to.copy_from_slice(buf);
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_write_ram() {
        let mut mem = Memory::new();
        mem.add_device(0xABCD_0000u32.into(), RamDevice::new(256));

        assert_eq!(mem.read::<u64>(0x8000_0000u32.into()), None);
        assert_eq!(mem.read::<u64>(0xABCD_0000u32.into()), Some(0u64));

        assert_eq!(mem.write::<u64>(0xABCD_0000u32.into(), 0x1234), Some(()));
        assert_eq!(mem.read::<u64>(0xABCD_0000u32.into()), Some(0x1234));
    }
}

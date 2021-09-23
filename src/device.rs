//! Implementation of a generic device. The device can be anything from a simple memory device,
//! to the PLIC or UART device.

mod ram;
pub use ram::RamDevice;

use crate::{
    trap::{Exception, Result},
    Address,
};
use bytemuck::Pod;
use object::{File, Object, ObjectSegment};
use std::collections::HashMap;
use std::mem::align_of;

/// The default memory size that each device bus will allocate by default.
pub const DEFAULT_MEMORY_SIZE: usize = 2 << 20;

/// The address where DRAM will start.
pub const DRAM_BASE: u64 = 0x8000_0000;

/// Any device that is able to read/write memory from/to.
///
/// Any device must specify the size it covers using the `size()` method, but it can not control
/// the base address, since that will be done by the user.
pub trait Device {
    /// The number of bytes this memory device covers, starting from the base address.
    fn size(&self) -> u64;

    /// Fill `buf` with bytes at the given address.
    ///
    /// Note that the address is a relativ offset to the base address of this device.
    ///
    /// # Returns
    ///
    /// `Ok(())` if the load was successful and the **whole** buffer is filled. Not filling the buffer,
    /// but returning `Ok(())`, will be the same behaviour as filling the buffer with zeros.
    fn load(&self, off: u64, buf: &mut [u8]) -> Result<()>;

    /// Write `buf`s contents to the given address.
    ///
    /// Note that the address is a relativ offset to the base address of this device.
    ///
    /// # Returns
    ///
    /// `Ok(())` if the write was successful and the **whole** buffer was written.
    /// Not writing the whole buffer, might lead to logic bugs.
    fn write(&mut self, off: u64, buf: &[u8]) -> Result<()>;
}

/// The emulation of a data bus that contains a bunch of devices at specific addresses.
///
/// Used for reading and writing memory.
pub struct DeviceBus {
    devices: HashMap<Address, Box<dyn Device>>,
}

impl Default for DeviceBus {
    fn default() -> Self {
        Self::new()
    }
}

impl DeviceBus {
    /// Create a new memory bus with a RAM device with [`DEFAULT_MEMORY_SIZE`] bytes.
    pub fn new() -> Self {
        let mut bus = DeviceBus {
            devices: HashMap::new(),
        };
        bus.add_device(DRAM_BASE.into(), RamDevice::new(DEFAULT_MEMORY_SIZE));
        bus
    }

    /// Load an object file that was previously parsed by the [`object`] crate.
    pub fn load_object(&mut self, obj: File<'_>) -> object::Result<()> {
        // FIXME: Check for RISC-V architecture
        assert!(obj.is_little_endian(), "Big Endian not supported");

        // go through each section that is not at address zero and has no zero size
        for seg in obj.segments() {
            // first, get the predefined data from the file
            let mut data = seg.data()?.to_vec();

            // then extend the segment to it's real size with zeroes
            data.resize(seg.size() as usize, 0);

            // write the data into the RAM device
            let addr = seg.address().into();
            let (&offset, dev) = self
                .device_for_mut(addr)
                .expect("failed to find device to write ELF segment into");

            dev.write(u64::from(addr) - u64::from(offset), &data)
                .expect("failed to write ELF segment to device");
        }

        Ok(())
    }

    /// Add a new device to this memory bus, that starts at the `base` address.
    pub fn add_device(&mut self, base: Address, dev: impl Device + 'static) {
        // FIXME: check overlap of addresses here
        self.devices.insert(base, Box::new(dev));
    }

    /// Read a `T` from the given address.
    ///
    /// # Returns
    ///
    /// `None` if the read failed.
    pub fn read<T: MemoryPod>(&self, addr: Address) -> Result<T> {
        // check alignment of the address
        if u64::from(addr) & (align_of::<T>() as u64 - 1) != 0 {
            return Err(Exception::LoadAddressMisaligned(addr));
        }

        // find the device that has the smallest, positive distance
        // from the requested address
        let (&offset, device) = self.device_for(addr).ok_or(Exception::LoadAccessFault)?;

        // create a zeroed `T` to read into
        let mut item = T::zeroed();
        device.load(
            u64::from(addr) - u64::from(offset),
            bytemuck::bytes_of_mut(&mut item),
        )?;
        Ok(item.process_read())
    }

    /// Write a `T` to the given address.
    ///
    /// # Returns
    ///
    /// `None` if the read failed, which may be caused by unaligned address,
    /// no physical memory for `addr` and others.
    pub fn write<T: MemoryPod>(&mut self, addr: Address, item: T) -> Result<()> {
        // check alignment of the address
        if u64::from(addr) & (align_of::<T>() as u64 - 1) != 0 {
            return Err(Exception::StoreAddressMisaligned(addr));
        }

        // find the first device that contains the given address
        let (&offset, device) = self
            .device_for_mut(addr)
            .ok_or(Exception::StoreAccessFault)?;

        // write the item into the device
        let item = item.process_write();
        device.write(
            u64::from(addr) - u64::from(offset),
            bytemuck::bytes_of(&item),
        )?;
        Ok(())
    }

    #[allow(clippy::borrowed_box)]
    fn device_for(&self, addr: Address) -> Option<(&Address, &Box<dyn Device>)> {
        self.devices.iter().find(|(&k, v)| {
            let base = u64::from(k);
            let end = base + v.size();
            let addr = u64::from(addr);

            base <= addr && addr < end
        })
    }

    fn device_for_mut(&mut self, addr: Address) -> Option<(&Address, &mut Box<dyn Device>)> {
        self.devices.iter_mut().find(|(&k, v)| {
            let base = u64::from(k);
            let end = base + v.size();
            let addr = u64::from(addr);

            base <= addr && addr < end
        })
    }
}

/// Trait for reading and writing arbitrary values from a [`DeviceBus`].
pub trait MemoryPod: Pod {
    /// After reading a type, it may need further processing, e.g. swapping bytes for the correct
    /// endianess.
    fn process_read(self) -> Self;

    /// Before writing this type to memory, it may need further processing.
    fn process_write(self) -> Self;
}

macro_rules! impl_int {
    ($($int:ty),*$(,)?) => {
        $(
        impl MemoryPod for $int {
            fn process_read(self) -> Self {
                self.to_le()
            }

            fn process_write(self) -> Self {
                <$int>::from_le(self)
            }
        }
        )*
    };
}

impl_int!(usize, u8, u16, u32, u64, isize, i8, i16, i32, i64);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_write_ram() {
        let mut mem = DeviceBus::new();

        assert_eq!(
            mem.read::<u64>(0x6000_0000u32.into()),
            Err(Exception::LoadAccessFault)
        );
        assert_eq!(mem.read::<u64>(0x8000_0000u32.into()), Ok(0u64));

        assert_eq!(mem.write::<u64>(0x8000_0000u32.into(), 0x1234), Ok(()));
        assert_eq!(mem.read::<u64>(0x8000_0000u32.into()), Ok(0x1234));
    }
}

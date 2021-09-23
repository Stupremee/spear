use super::{Device, Exception, Result};

/// A [`Device`] which acts as a RAM module containing a fixed buffer of memory.
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

impl Device for RamDevice {
    fn size(&self) -> u64 {
        self.ram.len() as u64
    }

    fn load(&self, off: u64, buf: &mut [u8]) -> Result<()> {
        let addr = off as usize;
        if let Some(from) = self.ram.get(addr..addr + buf.len()) {
            buf.copy_from_slice(from);
            Ok(())
        } else {
            Err(Exception::LoadAccessFault)
        }
    }

    fn write(&mut self, off: u64, buf: &[u8]) -> Result<()> {
        let addr = off as usize;
        if let Some(to) = self.ram.get_mut(addr..addr + buf.len()) {
            to.copy_from_slice(buf);
            Ok(())
        } else {
            Err(Exception::StoreAccessFault)
        }
    }
}

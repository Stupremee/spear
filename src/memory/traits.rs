use bytemuck::Pod;

/// Trait for reading and writing arbitrary values from [`Memory`](super::Memory).
pub trait MemoryData: Pod {
    /// After reading a type, it may need further processing, e.g. swapping bytes for the correct
    /// endianess.
    fn process_read(self) -> Self;

    /// Before writing this type to memory, it may need further processing.
    fn process_write(self) -> Self;
}

macro_rules! impl_int {
    ($($int:ty),*$(,)?) => {
        $(
        impl MemoryData for $int {
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

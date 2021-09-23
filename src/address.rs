/// Type-Safe representation of a pointer-wide value.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Address(u64);

impl Address {
    /// Get the NULL address.
    pub const fn zero() -> Self {
        Self(0)
    }
}

impl From<u64> for Address {
    fn from(x: u64) -> Self {
        Self(x)
    }
}

impl From<Address> for u64 {
    fn from(x: Address) -> Self {
        x.0
    }
}

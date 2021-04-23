use std::ops;

/// Type-Safe representation of a pointer-wide value.
///
/// This type also provides abstractions for converting between 32 and 64 (and soon 128)
/// bit addresses.
#[repr(transparent)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Address(u64);

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

impl ops::Add for Address {
    type Output = Address;

    fn add(self, x: Address) -> Self {
        Address(self.0 + x.0)
    }
}

impl ops::Sub for Address {
    type Output = Address;

    fn sub(self, x: Address) -> Self {
        Address(self.0 - x.0)
    }
}

impl ops::Mul for Address {
    type Output = Address;

    fn mul(self, x: Address) -> Self {
        Address(self.0 * x.0)
    }
}

impl ops::Div for Address {
    type Output = Address;

    fn div(self, x: Address) -> Self {
        Address(self.0 / x.0)
    }
}

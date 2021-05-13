use std::ops;

// TODO: Use an address representation, that is not 16 bytes large.

/// Different representations of an address.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum AddressKind {
    /// 32-bit address
    U32(u32),
    /// 64-bit address
    U64(u64),
}

/// Type-Safe representation of a pointer-wide value.
///
/// This type also provides abstractions for converting between 32 and 64 (and soon 128)
/// bit addresses.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Address(AddressKind);

impl Address {
    /// Convert this value into a signed value.
    #[inline]
    pub fn signed(self) -> SignedAddress {
        match self.0 {
            AddressKind::U32(x) => (x as i32).into(),
            AddressKind::U64(x) => (x as i64).into(),
        }
    }

    /// Get the inner representation of this address.
    #[inline]
    pub fn kind(self) -> AddressKind {
        self.0
    }
}

impl From<u64> for Address {
    fn from(x: u64) -> Self {
        Self(AddressKind::U64(x))
    }
}

impl From<u32> for Address {
    fn from(x: u32) -> Self {
        Self(AddressKind::U32(x))
    }
}

impl From<Address> for u64 {
    fn from(x: Address) -> Self {
        match x.0 {
            AddressKind::U32(x) => x as u64,
            AddressKind::U64(x) => x,
        }
    }
}

/// Different representations of an address.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SignedAddressKind {
    /// 32-bit address
    I32(i32),
    /// 64-bit address
    I64(i64),
}

/// Type-Safe representation of a pointer-wide signed value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct SignedAddress(SignedAddressKind);

impl SignedAddress {
    /// Convert this value into a unsigned value.
    #[inline]
    pub fn unsigned(self) -> Address {
        match self.0 {
            SignedAddressKind::I32(x) => (x as u32).into(),
            SignedAddressKind::I64(x) => (x as u64).into(),
        }
    }

    /// Get the inner representation of this address.
    #[inline]
    pub fn kind(self) -> SignedAddressKind {
        self.0
    }
}

impl From<i64> for SignedAddress {
    fn from(x: i64) -> Self {
        Self(SignedAddressKind::I64(x))
    }
}

impl From<i32> for SignedAddress {
    fn from(x: i32) -> Self {
        Self(SignedAddressKind::I32(x))
    }
}

impl ops::Shl<u32> for Address {
    type Output = Address;

    fn shl(self, x: u32) -> Self::Output {
        match self.0 {
            AddressKind::U64(a) => (a << x).into(),
            AddressKind::U32(a) => (a << x).into(),
        }
    }
}

impl ops::Shr<u32> for Address {
    type Output = Address;

    fn shr(self, x: u32) -> Self::Output {
        match self.0 {
            AddressKind::U64(a) => (a >> x).into(),
            AddressKind::U32(a) => (a >> x).into(),
        }
    }
}

impl ops::Shr<u32> for SignedAddress {
    type Output = SignedAddress;

    fn shr(self, x: u32) -> Self::Output {
        match self.0 {
            SignedAddressKind::I64(a) => (a >> x).into(),
            SignedAddressKind::I32(a) => (a >> x).into(),
        }
    }
}

macro_rules! impl_op {
    ($trait:ident, $method:ident, $op:ident) => {
        impl ops::$trait for Address {
            type Output = Address;

            fn $method(self, x: Address) -> Self {
                match (self.0, x.0) {
                    (AddressKind::U64(a), AddressKind::U64(b)) => a.$op(b).into(),
                    (AddressKind::U32(a), AddressKind::U32(b)) => a.$op(b).into(),
                    (AddressKind::U64(a), AddressKind::U32(b)) => a.$op(b as u64).into(),
                    (AddressKind::U32(a), AddressKind::U64(b)) => a.$op(b as u32).into(),
                }
            }
        }

        impl ops::$trait<u64> for Address {
            type Output = Address;

            fn $method(self, x: u64) -> Self {
                match self.0 {
                    AddressKind::U64(a) => a.$op(x).into(),
                    AddressKind::U32(a) => a.$op(x as u32).into(),
                }
            }
        }

        impl ops::$trait<u32> for Address {
            type Output = Address;

            fn $method(self, x: u32) -> Self {
                match self.0 {
                    AddressKind::U64(a) => a.$op(x as u64).into(),
                    AddressKind::U32(a) => a.$op(x).into(),
                }
            }
        }
    };
}

impl_op!(Add, add, wrapping_add);
impl_op!(Sub, sub, wrapping_sub);
impl_op!(Mul, mul, wrapping_mul);
impl_op!(Div, div, wrapping_div);
impl_op!(BitAnd, bitand, bitand);
impl_op!(BitOr, bitor, bitor);
impl_op!(BitXor, bitxor, bitxor);

macro_rules! impl_sign_op {
    ($trait:ident, $method:ident, $op:ident) => {
        impl ops::$trait for SignedAddress {
            type Output = SignedAddress;

            fn $method(self, x: SignedAddress) -> Self {
                match (self.0, x.0) {
                    (SignedAddressKind::I64(a), SignedAddressKind::I64(b)) => a.$op(b).into(),
                    (SignedAddressKind::I32(a), SignedAddressKind::I32(b)) => a.$op(b).into(),
                    (SignedAddressKind::I64(a), SignedAddressKind::I32(b)) => {
                        a.$op(b as i64).into()
                    }
                    (SignedAddressKind::I32(a), SignedAddressKind::I64(b)) => {
                        a.$op(b as i32).into()
                    }
                }
            }
        }

        impl ops::$trait<i64> for SignedAddress {
            type Output = SignedAddress;

            fn $method(self, x: i64) -> Self {
                match self.0 {
                    SignedAddressKind::I64(a) => a.$op(x).into(),
                    SignedAddressKind::I32(a) => a.$op(x as i32).into(),
                }
            }
        }

        impl ops::$trait<i32> for SignedAddress {
            type Output = SignedAddress;

            fn $method(self, x: i32) -> Self {
                match self.0 {
                    SignedAddressKind::I64(a) => a.$op(x as i64).into(),
                    SignedAddressKind::I32(a) => a.$op(x).into(),
                }
            }
        }
    };
}

impl_sign_op!(Add, add, wrapping_add);
impl_sign_op!(Sub, sub, wrapping_sub);
impl_sign_op!(Mul, mul, wrapping_mul);
impl_sign_op!(Div, div, wrapping_div);
impl_sign_op!(BitAnd, bitand, bitand);
impl_sign_op!(BitOr, bitor, bitor);
impl_sign_op!(BitXor, bitxor, bitxor);

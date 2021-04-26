use std::ops;

// TODO: Use an address representation, that is not 16 bytes large.

/// Private representation of an address.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Inner {
    U32(u32),
    U64(u64),
}

/// Type-Safe representation of a pointer-wide value.
///
/// This type also provides abstractions for converting between 32 and 64 (and soon 128)
/// bit addresses.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Address(Inner);

impl Address {
    /// Convert this value into a signed value.
    #[inline]
    pub fn signed(self) -> SignedAddress {
        match self.0 {
            Inner::U32(x) => (x as i32).into(),
            Inner::U64(x) => (x as i64).into(),
        }
    }
}

impl From<u64> for Address {
    fn from(x: u64) -> Self {
        Self(Inner::U64(x))
    }
}

impl From<u32> for Address {
    fn from(x: u32) -> Self {
        Self(Inner::U32(x))
    }
}

impl From<Address> for u64 {
    fn from(x: Address) -> Self {
        match x.0 {
            Inner::U32(x) => x as u64,
            Inner::U64(x) => x,
        }
    }
}

/// Private representation of a signed address.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum SignedInner {
    I32(i32),
    I64(i64),
}

/// Type-Safe representation of a pointer-wide signed value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct SignedAddress(SignedInner);

impl From<i64> for SignedAddress {
    fn from(x: i64) -> Self {
        Self(SignedInner::I64(x))
    }
}

impl From<i32> for SignedAddress {
    fn from(x: i32) -> Self {
        Self(SignedInner::I32(x))
    }
}

macro_rules! impl_op {
    ($trait:ident, $method:ident, $op:ident) => {
        impl ops::$trait for Address {
            type Output = Address;

            fn $method(self, x: Address) -> Self {
                match (self.0, x.0) {
                    (Inner::U64(a), Inner::U64(b)) => a.$op(b).into(),
                    (Inner::U32(a), Inner::U32(b)) => a.$op(b).into(),
                    (Inner::U64(a), Inner::U32(b)) => a.$op(b as u64).into(),
                    (Inner::U32(a), Inner::U64(b)) => a.$op(b as u32).into(),
                }
            }
        }

        impl ops::$trait<u64> for Address {
            type Output = Address;

            fn $method(self, x: u64) -> Self {
                match self.0 {
                    Inner::U64(a) => a.$op(x).into(),
                    Inner::U32(a) => a.$op(x as u32).into(),
                }
            }
        }

        impl ops::$trait<u32> for Address {
            type Output = Address;

            fn $method(self, x: u32) -> Self {
                match self.0 {
                    Inner::U64(a) => a.$op(x as u64).into(),
                    Inner::U32(a) => a.$op(x).into(),
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
                    (SignedInner::I64(a), SignedInner::I64(b)) => a.$op(b).into(),
                    (SignedInner::I32(a), SignedInner::I32(b)) => a.$op(b).into(),
                    (SignedInner::I64(a), SignedInner::I32(b)) => a.$op(b as i64).into(),
                    (SignedInner::I32(a), SignedInner::I64(b)) => a.$op(b as i32).into(),
                }
            }
        }

        impl ops::$trait<i64> for SignedAddress {
            type Output = SignedAddress;

            fn $method(self, x: i64) -> Self {
                match self.0 {
                    SignedInner::I64(a) => a.$op(x).into(),
                    SignedInner::I32(a) => a.$op(x as i32).into(),
                }
            }
        }

        impl ops::$trait<i32> for SignedAddress {
            type Output = SignedAddress;

            fn $method(self, x: i32) -> Self {
                match self.0 {
                    SignedInner::I64(a) => a.$op(x as i64).into(),
                    SignedInner::I32(a) => a.$op(x).into(),
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

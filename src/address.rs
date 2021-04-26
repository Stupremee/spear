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

macro_rules! impl_op {
    ($trait:ident, $method:ident, $op:ident) => {
        impl ops::$trait for Address {
            type Output = Address;

            fn $method(self, x: Address) -> Self {
                match (self.0, x.0) {
                    (Inner::U64(a), Inner::U64(b)) => a.$op(b).into(),
                    (Inner::U32(a), Inner::U32(b)) => a.$op(b).into(),
                    _ => panic!("operating on different address lengths is not supported yet"),
                }
            }
        }
    };
}

impl_op!(Add, add, wrapping_add);
impl_op!(Sub, sub, wrapping_sub);
impl_op!(Mul, mul, wrapping_mul);
impl_op!(Div, div, wrapping_div);

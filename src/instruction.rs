//! This module contains everything related to raw instructions, including
//! the central `Instruction` type and functions to decode an instruction.

#[macro_use]
mod macros;
mod fmt;

pub mod parse;
pub use parse::decode;

/// Enum for representing the different instruction formats.
#[derive(Debug)]
pub enum InstructionType {
    /// The R-Type.
    R(RType),
    /// The I-Type.
    I(IType),
    /// The S-Type.
    S(SType),
    /// The B-Type.
    B(BType),
    /// The U-Type.
    U(UType),
    /// The J-Type.
    J(JType),
}

#[rustfmt::skip]
mod from_impls {
    use super::*;

    impl From<RType> for InstructionType { fn from(x: RType) -> Self { Self::R(x) } }
    impl From<IType> for InstructionType { fn from(x: IType) -> Self { Self::I(x) } }
    impl From<SType> for InstructionType { fn from(x: SType) -> Self { Self::S(x) } }
    impl From<BType> for InstructionType { fn from(x: BType) -> Self { Self::B(x) } }
    impl From<UType> for InstructionType { fn from(x: UType) -> Self { Self::U(x) } }
    impl From<JType> for InstructionType { fn from(x: JType) -> Self { Self::J(x) } }
}

/// Type safe access for a X register.
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Register(u8);

impl Register {
    /// Create  new [`Register`] from a raw register index, which validated before
    /// returning the new type.
    ///
    /// # Panics
    ///
    /// If the raw index is not a valid register index.
    pub fn new(raw: u8) -> Self {
        match raw {
            0..=31 => Self(raw),
            _ => panic!("invalid register index: {}", raw),
        }
    }

    /// Check if this register is `x0`.
    #[inline]
    pub fn is_zero(self) -> bool {
        self.0 == 0
    }
}

impl From<u8> for Register {
    fn from(x: u8) -> Self {
        Self::new(x)
    }
}

/// The R instruction format.
#[derive(Debug, Clone)]
pub struct RType {
    /// The destination register index.
    pub rd: Register,
    /// The first source register index.
    pub rs1: Register,
    /// The seconf source register index.
    pub rs2: Register,
}

/// The I instruction format.
#[derive(Debug, Clone)]
pub struct IType {
    /// The immediate value as the raw byte value, that is not yet sign-extended.
    pub val: u32,
    /// The destination register index.
    pub rd: Register,
    /// The source register index.
    pub rs: Register,
}

impl IType {
    /// Sign extend the raw immediate value of this I-type.
    #[inline]
    pub fn sign_imm(&self) -> i32 {
        let val = self.val as i32;
        (val << 20) >> 20
    }

    /// Get the shift amount of this I-type.
    #[inline]
    pub fn shamt(&self) -> u8 {
        (self.val & 0x3F) as u8
    }
}

/// The S instruction format.
#[derive(Debug, Clone)]
pub struct SType {
    /// The immediate value as the raw byte value, that is not yet sign-extended.
    pub val: u32,
    /// The source 1 register index.
    pub rs1: Register,
    /// The source 2 register index.
    pub rs2: Register,
}

impl SType {
    /// Sign extend the raw immediate value of this S-type.
    #[inline]
    pub fn sign_imm(&self) -> i32 {
        let val = self.val as i32;
        (val << 20) >> 20
    }
}

/// The B instruction format.
#[derive(Debug, Clone)]
pub struct BType {
    /// The immediate value as the raw byte value, that is not yet sign-extended.
    pub val: u32,
    /// The source 1 register index.
    pub rs1: Register,
    /// The source 2 register index.
    pub rs2: Register,
}

impl BType {
    /// Sign extend the raw immediate value of this B-type.
    #[inline]
    pub fn sign_imm(&self) -> i32 {
        let val = self.val as i32;
        (val << 19) >> 19
    }
}

/// The U instruction format.
#[derive(Debug, Clone)]
pub struct UType {
    /// The immediate value as the raw byte value, that is not yet sign-extended.
    pub val: u32,
    /// The destination register index.
    pub rd: Register,
}

impl UType {
    /// Get the immediate value of this U-type.
    #[inline]
    pub fn imm(&self) -> u32 {
        self.val
    }
}

/// The J instruction format.
#[derive(Debug, Clone)]
pub struct JType {
    /// The immediate value as the raw byte value, that is not yet sign-extended.
    pub val: u32,
    /// The destination register index.
    pub rd: Register,
}

impl JType {
    /// Sign extend the raw immediate value of this J-type.
    #[inline]
    pub fn sign_imm(&self) -> i32 {
        let val = self.val as i32;
        (val << 11) >> 11
    }
}

instructions! {
    base(RV32I) [
        LUI(UType),
        AUIPC(UType),

        JAL(JType),
        JALR(IType),

        BEQ(BType),
        BNE(BType),
        BLT(BType),
        BGE(BType),
        BLTU(BType),
        BGEU(BType),

        LB(IType),
        LH(IType),
        LW(IType),
        LBU(IType),
        LHU(IType),

        SB(SType),
        SH(SType),
        SW(SType),

        ADDI(IType),
        SLTI(IType),
        SLTIU(IType),
        XORI(IType),
        ORI(IType),
        ANDI(IType),
        SLLI(IType),
        SRLI(IType),
        SRAI(IType),

        ADD(RType),
        SUB(RType),
        SLL(RType),
        SLT(RType),
        SLTU(RType),
        XOR(RType),
        SRL(RType),
        SRA(RType),
        OR(RType),
        AND(RType),

        FENCE(IType),
        FENCEI(IType),
        ECALL(IType),
        EBREAK(IType),
    ]
}

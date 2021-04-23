//! Implementation of the RV32I base extension.

use derive_more::Display;
use std::fmt;

mod parse;
pub use parse::parse;

/// The RV32I base extension.
pub struct Extension {}

/// Enum for representing the different instruction formats.
#[derive(Debug)]
pub enum InstructionType {
    /// The R-Type.
    R,
    /// The I-Type.
    I,
    /// The S-Type.
    S,
    /// The B-Type.
    B,
    /// The U-Type.
    U,
    /// The J-Type.
    J,
}

/// The R instruction format.
#[derive(Debug, Display)]
#[display(fmt = "x{}, x{}, x{}", rd, rs1, rs2)]
pub struct RType {
    /// The destination register index.
    pub rd: u8,
    /// The first source register index.
    pub rs1: u8,
    /// The seconf source register index.
    pub rs2: u8,
}

/// The I instruction format.
#[derive(Debug)]
pub struct IType {
    /// The immediate value as the raw byte value, that is not yet sign-extended.
    pub val: u32,
    /// The destination register index.
    pub rd: u8,
    /// The source register index.
    pub rs: u8,
}

impl IType {
    /// Sign extend the raw immediate value of this I-type.
    #[inline]
    pub fn sign_val(&self) -> i32 {
        let val = self.val as i32;
        (val << 20) >> 20
    }
}

impl fmt::Display for IType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let imm = self.sign_val();
        write!(f, "x{}, x{}, {}", self.rd, self.rs, imm)
    }
}

/// The S instruction format.
#[derive(Debug)]
pub struct SType {
    /// The immediate value as the raw byte value, that is not yet sign-extended.
    pub val: u32,
    /// The source 1 register index.
    pub rs1: u8,
    /// The source 2 register index.
    pub rs2: u8,
}

impl SType {
    /// Sign extend the raw immediate value of this S-type.
    #[inline]
    pub fn sign_val(&self) -> i32 {
        let val = self.val as i32;
        (val << 20) >> 20
    }
}

impl fmt::Display for SType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let imm = self.sign_val();
        write!(f, "x{}, {}(x{})", self.rs2, imm, self.rs1)
    }
}

/// The B instruction format.
#[derive(Debug)]
pub struct BType {
    /// The immediate value as the raw byte value, that is not yet sign-extended.
    pub val: u32,
    /// The source 1 register index.
    pub rs1: u8,
    /// The source 2 register index.
    pub rs2: u8,
}

impl BType {
    /// Sign extend the raw immediate value of this B-type.
    #[inline]
    pub fn sign_val(&self) -> i32 {
        let val = self.val as i32;
        (val << 19) >> 19
    }
}

impl fmt::Display for BType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let imm = self.sign_val();
        write!(f, "{}, x{}, x{}", imm, self.rs1, self.rs2,)
    }
}

/// The U instruction format.
#[derive(Debug)]
pub struct UType {
    /// The immediate value as the raw byte value, that is not yet sign-extended.
    pub val: u32,
    /// The destination register index.
    pub rd: u8,
}

impl UType {
    /// Sign extend the raw immediate value of this U-type.
    #[inline]
    pub fn sign_val(&self) -> i32 {
        self.val as i32
    }
}

impl fmt::Display for UType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let imm = self.sign_val();
        write!(f, "x{}, {}", self.rd, imm,)
    }
}

/// The J instruction format.
#[derive(Debug)]
pub struct JType {
    /// The immediate value as the raw byte value, that is not yet sign-extended.
    pub val: u32,
    /// The destination register index.
    pub rd: u8,
}

impl JType {
    /// Sign extend the raw immediate value of this J-type.
    #[inline]
    pub fn sign_val(&self) -> i32 {
        let val = self.val as i32;
        (val << 11) >> 11
    }
}

impl fmt::Display for JType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let imm = self.sign_val();
        write!(f, "x{}, {}", self.rd, imm,)
    }
}

/// The instruction type of the RV32I base extension.
#[derive(Debug, Display)]
#[allow(clippy::upper_case_acronyms)]
#[allow(missing_docs)]
pub enum Instruction {
    #[display(fmt = "lui {}", "_0")]
    LUI(UType),
    #[display(fmt = "auipc {}", "_0")]
    AUIPC(UType),

    #[display(fmt = "jal {}", "_0")]
    JAL(JType),
    #[display(fmt = "jalr {}", "_0")]
    JALR(IType),

    #[display(fmt = "beq {}", "_0")]
    BEQ(BType),
    #[display(fmt = "bne {}", "_0")]
    BNE(BType),
    #[display(fmt = "blt {}", "_0")]
    BLT(BType),
    #[display(fmt = "bge {}", "_0")]
    BGE(BType),
    #[display(fmt = "bltu {}", "_0")]
    BLTU(BType),
    #[display(fmt = "bgeu {}", "_0")]
    BGEU(BType),

    #[display(fmt = "lb {}", "_0")]
    LB(IType),
    #[display(fmt = "lh {}", "_0")]
    LH(IType),
    #[display(fmt = "lw {}", "_0")]
    LW(IType),
    #[display(fmt = "lbu {}", "_0")]
    LBU(IType),
    #[display(fmt = "lhu {}", "_0")]
    LHU(IType),

    #[display(fmt = "sb {}", "_0")]
    SB(SType),
    #[display(fmt = "sh {}", "_0")]
    SH(SType),
    #[display(fmt = "sw {}", "_0")]
    SW(SType),

    #[display(fmt = "addi {}", "_0")]
    ADDI(IType),
    #[display(fmt = "slti {}", "_0")]
    SLTI(IType),
    #[display(fmt = "sltiu {}", "_0")]
    SLTIU(IType),
    #[display(fmt = "xori {}", "_0")]
    XORI(IType),
    #[display(fmt = "ori {}", "_0")]
    ORI(IType),
    #[display(fmt = "andi {}", "_0")]
    ANDI(IType),
    #[display(fmt = "slli {}", "_0")]
    SLLI(IType),
    #[display(fmt = "srli {}", "_0")]
    SRLI(IType),
    #[display(fmt = "srai {}", "_0")]
    SRAI(IType),

    #[display(fmt = "add {}", "_0")]
    ADD(RType),
    #[display(fmt = "sub {}", "_0")]
    SUB(RType),
    #[display(fmt = "sll {}", "_0")]
    SLL(RType),
    #[display(fmt = "slt {}", "_0")]
    SLT(RType),
    #[display(fmt = "sltu {}", "_0")]
    SLTU(RType),
    #[display(fmt = "xor {}", "_0")]
    XOR(RType),
    #[display(fmt = "srl {}", "_0")]
    SRL(RType),
    #[display(fmt = "sra {}", "_0")]
    SRA(RType),
    #[display(fmt = "or {}", "_0")]
    OR(RType),
    #[display(fmt = "and {}", "_0")]
    AND(RType),

    #[display(fmt = "fence {}", "_0")]
    FENCE(IType),
    #[display(fmt = "ecall {}", "_0")]
    ECALL(IType),
    #[display(fmt = "ebreak {}", "_0")]
    EBREAK(IType),
}

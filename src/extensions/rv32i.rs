//! Implementation of the RV32I base extension.

mod parse;
pub use parse::parse;

/// The RV32I base extension.
pub struct Extension {}

/// Enum for representing the different instruction formats.
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
pub struct RType {
    /// The destination register index.
    pub rd: u8,
    /// The first source register index.
    pub rs1: u8,
    /// The seconf source register index.
    pub rs2: u8,
}

/// The I instruction format.
pub struct IType {
    /// The immediate value as the raw byte value, that is not yet sign-extended.
    pub val: u32,
    /// The destination register index.
    pub rd: u8,
    /// The source register index.
    pub rs: u8,
}

/// The S instruction format.
pub struct SType {
    /// The immediate value as the raw byte value, that is not yet sign-extended.
    pub val: u32,
    /// The source 1 register index.
    pub rs1: u8,
    /// The source 2 register index.
    pub rs2: u8,
}

/// The B instruction format.
pub struct BType {
    /// The immediate value as the raw byte value, that is not yet sign-extended.
    pub val: u32,
    /// The source 1 register index.
    pub rs1: u8,
    /// The source 2 register index.
    pub rs2: u8,
}

/// The U instruction format.
pub struct UType {
    /// The immediate value as the raw byte value, that is not yet sign-extended.
    pub val: u32,
    /// The destination register index.
    pub rd: u8,
}

/// The J instruction format.
pub struct JType {
    /// The immediate value as the raw byte value, that is not yet sign-extended.
    pub val: u32,
    /// The destination register index.
    pub rd: u8,
}

/// The instruction type of the RV32I base extension.
#[allow(clippy::upper_case_acronyms)]
#[allow(missing_docs)]
pub enum Instruction {
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
    ECALL(IType),
    EBREAK(IType),
}

//! Implementation of the RV32I base extension.

use crate::Address;
use derive_more::{Display, From, Into};
use std::fmt;

mod exec;
pub use exec::exec;

mod parse;
pub use parse::parse;

/// The RV32I base extension.
#[derive(Debug)]
pub struct Extension {
    registers: [Address; 32],
}

impl Extension {
    /// Create a new RV32I extension, that uses 32-bit wide registers.
    pub fn new_32bit() -> Self {
        Self {
            registers: [0u32.into(); 32],
        }
    }

    /// Read the given register out of this extension.
    pub fn read_register(&self, reg: Register) -> Address {
        if reg.0 == 0 {
            self.registers[0]
        } else {
            *self
                .registers
                .get(reg.0 as usize - 1)
                .expect("tried to access invalid register")
        }
    }

    /// Write the given value into `reg`.
    pub fn write_register(&mut self, reg: Register, x: Address) {
        if reg.0 != 0 {
            *self
                .registers
                .get_mut(reg.0 as usize - 1)
                .expect("tried to access invalid register") = x;
        }
    }
}

impl crate::Extension for Extension {
    type Inst = Instruction;

    fn parse_instruction(&self, x: u32) -> Option<Self::Inst> {
        parse(x)
    }
}

/// Type safe access for a X register.
/// This type does not guarantee anything abuot the validity about the value inside.
#[repr(transparent)]
#[derive(Debug, From, Into, Clone, Copy)]
pub struct Register(u8);

impl fmt::Display for Register {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            0 => write!(f, "zero"),
            1 => write!(f, "ra"),
            2 => write!(f, "sp"),
            3 => write!(f, "gp"),
            4 => write!(f, "tp"),
            5 => write!(f, "t0"),
            6 => write!(f, "t1"),
            7 => write!(f, "t2"),
            8 => write!(f, "s0"),
            9 => write!(f, "s1"),
            10 => write!(f, "a0"),
            11 => write!(f, "a1"),
            12 => write!(f, "a2"),
            13 => write!(f, "a3"),
            14 => write!(f, "a4"),
            15 => write!(f, "a5"),
            16 => write!(f, "a6"),
            17 => write!(f, "a7"),
            18 => write!(f, "s2"),
            19 => write!(f, "s3"),
            20 => write!(f, "s4"),
            21 => write!(f, "s5"),
            22 => write!(f, "s6"),
            23 => write!(f, "s7"),
            24 => write!(f, "s8"),
            25 => write!(f, "s9"),
            26 => write!(f, "s10"),
            27 => write!(f, "s11"),
            28 => write!(f, "t3"),
            29 => write!(f, "t4"),
            30 => write!(f, "t5"),
            31 => write!(f, "t6"),
            _ => write!(f, "x{}", self.0),
        }
    }
}

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
#[derive(Debug)]
pub struct RType {
    /// The destination register index.
    pub rd: Register,
    /// The first source register index.
    pub rs1: Register,
    /// The seconf source register index.
    pub rs2: Register,
}

impl fmt::Display for RType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}, {}, {}", self.rd, self.rs1, self.rs2)
    }
}

/// The I instruction format.
#[derive(Debug)]
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
    pub fn sign_val(&self) -> i32 {
        let val = self.val as i32;
        (val << 20) >> 20
    }
}

impl fmt::Display for IType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let imm = self.sign_val();
        write!(f, "{}, {}, {}", self.rd, self.rs, imm)
    }
}

/// The S instruction format.
#[derive(Debug)]
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
    pub fn sign_val(&self) -> i32 {
        let val = self.val as i32;
        (val << 20) >> 20
    }
}

impl fmt::Display for SType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let imm = self.sign_val();
        write!(f, "{}, {}({})", self.rs2, imm, self.rs1)
    }
}

/// The B instruction format.
#[derive(Debug)]
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
    pub fn sign_val(&self) -> i32 {
        let val = self.val as i32;
        (val << 19) >> 19
    }
}

impl fmt::Display for BType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let imm = self.sign_val();
        write!(f, "{}, {}, {}", self.rs1, self.rs2, imm)
    }
}

/// The U instruction format.
#[derive(Debug)]
pub struct UType {
    /// The immediate value as the raw byte value, that is not yet sign-extended.
    pub val: u32,
    /// The destination register index.
    pub rd: Register,
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
        write!(f, "{}, {}", self.rd, imm >> 12)
    }
}

/// The J instruction format.
#[derive(Debug)]
pub struct JType {
    /// The immediate value as the raw byte value, that is not yet sign-extended.
    pub val: u32,
    /// The destination register index.
    pub rd: Register,
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
        write!(f, "{}, {}", self.rd, imm,)
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

    #[display(fmt = "fence")]
    FENCE(IType),
    #[display(fmt = "ecall")]
    ECALL(IType),
    #[display(fmt = "ebreak")]
    EBREAK(IType),
}

impl crate::Instruction for Instruction {
    type Ext = Extension;

    fn exec(self, ext: &mut Self::Ext) {
        exec(ext, self)
    }
}

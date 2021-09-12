//! `fmt::Display` implementations for various instruction related types.

use super::*;
use core::fmt;

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
            _ => unreachable!(),
        }
    }
}

impl fmt::Display for RType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}, {}, {}", self.rd, self.rs1, self.rs2)
    }
}

impl fmt::Display for IType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let imm = self.sign_imm();
        write!(f, "{}, {}, {}", self.rd, self.rs, imm)
    }
}

impl fmt::Display for SType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let imm = self.sign_imm();
        write!(f, "{}, {}({})", self.rs2, imm, self.rs1)
    }
}

impl fmt::Display for BType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let imm = self.sign_imm();
        write!(f, "{}, {}, {}", self.rs1, self.rs2, imm)
    }
}

impl fmt::Display for UType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let imm = self.imm();
        write!(f, "{}, {}", self.rd, imm >> 12)
    }
}

impl fmt::Display for JType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let imm = self.sign_imm();
        write!(f, "{}, {}", self.rd, imm,)
    }
}

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Instruction::LUI(ty) => write!(f, "lui {}", ty)?,
            Instruction::AUIPC(ty) => write!(f, "auipc {}", ty)?,
            Instruction::JAL(ty) => write!(f, "jal {}", ty)?,
            Instruction::JALR(ty) => write!(f, "jalr {}", ty)?,
            Instruction::BEQ(ty) => write!(f, "beq {}", ty)?,
            Instruction::BNE(ty) => write!(f, "bne {}", ty)?,
            Instruction::BLT(ty) => write!(f, "blt {}", ty)?,
            Instruction::BGE(ty) => write!(f, "bge {}", ty)?,
            Instruction::BLTU(ty) => write!(f, "bltu {}", ty)?,
            Instruction::BGEU(ty) => write!(f, "bgeu {}", ty)?,
            Instruction::LB(ty) => write!(f, "lb {}", ty)?,
            Instruction::LH(ty) => write!(f, "lh {}", ty)?,
            Instruction::LW(ty) => write!(f, "lw {}", ty)?,
            Instruction::LBU(ty) => write!(f, "lbu {}", ty)?,
            Instruction::LHU(ty) => write!(f, "lhu {}", ty)?,
            Instruction::SB(ty) => write!(f, "sb {}", ty)?,
            Instruction::SH(ty) => write!(f, "sh {}", ty)?,
            Instruction::SW(ty) => write!(f, "sw {}", ty)?,
            Instruction::ADDI(ty) => write!(f, "addi {}", ty)?,
            Instruction::SLTI(ty) => write!(f, "slti {}", ty)?,
            Instruction::SLTIU(ty) => write!(f, "sltiu {}", ty)?,
            Instruction::XORI(ty) => write!(f, "xori {}", ty)?,
            Instruction::ORI(ty) => write!(f, "ori {}", ty)?,
            Instruction::ANDI(ty) => write!(f, "andi {}", ty)?,
            Instruction::SLLI(ty) => write!(f, "slli {}", ty)?,
            Instruction::SRLI(ty) => write!(f, "srli {}", ty)?,
            Instruction::SRAI(ty) => write!(f, "srai {}", ty)?,
            Instruction::ADD(ty) => write!(f, "add {}", ty)?,
            Instruction::SUB(ty) => write!(f, "sub {}", ty)?,
            Instruction::SLL(ty) => write!(f, "sll {}", ty)?,
            Instruction::SLT(ty) => write!(f, "slt {}", ty)?,
            Instruction::SLTU(ty) => write!(f, "sltu {}", ty)?,
            Instruction::XOR(ty) => write!(f, "xor {}", ty)?,
            Instruction::SRL(ty) => write!(f, "srl {}", ty)?,
            Instruction::SRA(ty) => write!(f, "sra {}", ty)?,
            Instruction::OR(ty) => write!(f, "or {}", ty)?,
            Instruction::AND(ty) => write!(f, "and {}", ty)?,
            Instruction::FENCE(ty) => write!(f, "fence {}", ty)?,
            Instruction::FENCEI(ty) => write!(f, "fencei {}", ty)?,
            Instruction::ECALL(_) => write!(f, "ecall")?,
            Instruction::EBREAK(_) => write!(f, "ebreak")?,
        }

        Ok(())
    }
}

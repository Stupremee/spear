//! Instruction decoding.

use super::{BType, IType, Instruction, InstructionType, JType, RType, SType, UType};

impl InstructionType {
    /// Find the associated instruction type for a specific opcode.
    pub fn from_opcode(opcode: u8) -> Option<Self> {
        match opcode {
            // R-variant
            0b011_0011 => Some(Self::R),
            // I-variant
            0b000_0011 | 0b000_1111 | 0b001_0011 | 0b110_0111 | 0b111_0011 => Some(Self::I),
            // S-variant
            0b010_0011 => Some(Self::S),
            // B-variant
            0b110_0011 => Some(Self::B),
            // U-variant
            0b001_0111 | 0b011_0111 => Some(Self::U),
            // J-variant
            0b110_1111 => Some(Self::J),
            _ => None,
        }
    }
}

impl RType {
    /// Parse a R-Type instruction from the raw bytes.
    ///
    /// # Returns
    /// A tuple containing the `funct3` and `funct7` and the parsed [`RType`].
    pub fn parse(inst: u32) -> (u8, u8, Self) {
        let rd = (inst >> 7) & 0x1F;
        let rs1 = (inst >> 15) & 0x1F;
        let rs2 = (inst >> 20) & 0x1F;
        let funct3 = (inst >> 12) & 0x7;
        let funct7 = (inst >> 25) & 0x7F;

        (
            funct3 as u8,
            funct7 as u8,
            RType {
                rd: (rd as u8).into(),
                rs1: (rs1 as u8).into(),
                rs2: (rs2 as u8).into(),
            },
        )
    }
}

impl IType {
    /// Parse a I-Type instruction from the raw bytes.
    ///
    /// # Returns
    /// A tuple containing the `funct3` and the parsed [`IType`].
    pub fn parse(inst: u32) -> (u8, Self) {
        let rd = (inst >> 7) & 0x1F;
        let rs = (inst >> 15) & 0x1F;
        let funct3 = (inst >> 12) & 0x7;
        let imm = (inst >> 20) & 0xFFF;

        (
            funct3 as u8,
            IType {
                val: imm,
                rd: (rd as u8).into(),
                rs: (rs as u8).into(),
            },
        )
    }
}

impl SType {
    /// Parse a S-Type instruction from the raw bytes.
    ///
    /// # Returns
    /// A tuple containing the `funct3` and the parsed [`SType`].
    pub fn parse(inst: u32) -> (u8, Self) {
        let rs1 = (inst >> 15) & 0x1F;
        let rs2 = (inst >> 20) & 0x1F;
        let funct3 = (inst >> 12) & 0x7;

        let imm_low = (inst >> 7) & 0x1F;
        let imm_high = (inst >> 25) << 5;

        (
            funct3 as u8,
            SType {
                val: imm_high | imm_low,
                rs1: (rs1 as u8).into(),
                rs2: (rs2 as u8).into(),
            },
        )
    }
}

impl BType {
    /// Parse a B-Type instruction from the raw bytes.
    ///
    /// # Returns
    /// A tuple containing the `funct3` and the parsed [`BType`].
    pub fn parse(inst: u32) -> (u8, Self) {
        let rs1 = (inst >> 15) & 0x1F;
        let rs2 = (inst >> 20) & 0x1F;
        let funct3 = (inst >> 12) & 0x7;

        let imm12105 = (inst >> 25) & 0x7F;
        let imm4111 = (inst >> 7) & 0x1F;
        let imm12 = (imm12105 & 0x40) >> 6;
        let imm105 = imm12105 & 0x3F;
        let imm41 = (imm4111 & 0x1E) >> 1;
        let imm11 = imm4111 & 0x1;

        let imm = (imm12 << 12) | (imm11 << 11) | (imm105 << 5) | (imm41 << 1);

        (
            funct3 as u8,
            BType {
                val: imm,
                rs1: (rs1 as u8).into(),
                rs2: (rs2 as u8).into(),
            },
        )
    }
}

impl UType {
    /// Parse a U-Type instruction from the raw bytes.
    ///
    /// # Returns
    /// The parsed [`UType`].
    pub fn parse(inst: u32) -> Self {
        let rd = (inst >> 7) & 0x1F;
        let imm = (inst >> 12) << 12;

        UType {
            val: imm,
            rd: (rd as u8).into(),
        }
    }
}

impl JType {
    /// Parse a J-Type instruction from the raw bytes.
    ///
    /// # Returns
    /// The parsed [`JType`].
    pub fn parse(inst: u32) -> Self {
        let rd = (inst >> 7) & 0x1F;

        let imm = (inst & 0xFFFF_F000) >> 12;
        let imm20 = (imm >> 19) & 0x1;
        let imm101 = (imm >> 9) & 0x3FF;
        let imm11 = (imm >> 8) & 0x1;
        let imm1912 = imm & 0xFF;

        let imm = (imm20 << 20) | (imm1912 << 12) | (imm11 << 11) | (imm101 << 1);

        JType {
            val: imm,
            rd: (rd as u8).into(),
        }
    }
}

fn get_r_type(ty: RType, funct3: u8, funct7: u8) -> Option<Instruction> {
    let inst = match (funct3, funct7) {
        (0b000, 0b0000000) => Instruction::ADD(ty),
        (0b000, 0b0100000) => Instruction::SUB(ty),
        (0b001, 0b0000000) => Instruction::SLL(ty),
        (0b010, 0b0000000) => Instruction::SLT(ty),
        (0b011, 0b0000000) => Instruction::SLTU(ty),
        (0b100, 0b0000000) => Instruction::XOR(ty),
        (0b101, 0b0000000) => Instruction::SRL(ty),
        (0b101, 0b0100000) => Instruction::SRA(ty),
        (0b110, 0b0100000) => Instruction::OR(ty),
        (0b111, 0b0100000) => Instruction::AND(ty),
        _ => return None,
    };
    Some(inst)
}

fn get_i_type(mut ty: IType, opcode: u8, funct3: u8) -> Option<Instruction> {
    let inst = match (opcode, funct3) {
        (0b000_0011, 0b000) => Instruction::LB(ty),
        (0b000_0011, 0b001) => Instruction::LH(ty),
        (0b000_0011, 0b010) => Instruction::LW(ty),
        (0b000_0011, 0b100) => Instruction::LBU(ty),
        (0b000_0011, 0b101) => Instruction::LHU(ty),

        (0b000_1111, 0b000) => Instruction::FENCE(ty),

        (0b001_0011, 0b000) => Instruction::ADDI(ty),
        (0b001_0011, 0b010) => Instruction::SLTI(ty),
        (0b001_0011, 0b011) => Instruction::SLTIU(ty),
        (0b001_0011, 0b100) => Instruction::XORI(ty),
        (0b001_0011, 0b110) => Instruction::ORI(ty),
        (0b001_0011, 0b111) => Instruction::ANDI(ty),
        (0b001_0011, 0b001) => Instruction::SLLI(ty),
        (0b001_0011, 0b101) => match ty.val & (1 << 10) == 0 {
            true => Instruction::SRLI(ty),
            false => {
                ty.val &= !(1 << 10);
                Instruction::SRAI(ty)
            }
        },

        (0b110_0111, 0b000) => Instruction::JALR(ty),

        (0b111_0011, 0b000) if ty.val == 0 => Instruction::ECALL(ty),
        (0b111_0011, 0b000) if ty.val == 1 => Instruction::EBREAK(ty),
        _ => return None,
    };
    Some(inst)
}

fn get_s_type(ty: SType, funct3: u8) -> Option<Instruction> {
    let inst = match funct3 {
        0b000 => Instruction::SB(ty),
        0b001 => Instruction::SH(ty),
        0b010 => Instruction::SW(ty),
        _ => return None,
    };
    Some(inst)
}

fn get_b_type(ty: BType, funct3: u8) -> Option<Instruction> {
    let inst = match funct3 {
        0b000 => Instruction::BEQ(ty),
        0b001 => Instruction::BNE(ty),
        0b100 => Instruction::BLT(ty),
        0b101 => Instruction::BGE(ty),
        0b110 => Instruction::BLTU(ty),
        0b111 => Instruction::BGEU(ty),
        _ => return None,
    };
    Some(inst)
}

fn get_u_type(ty: UType, opcode: u8) -> Option<Instruction> {
    let inst = match opcode {
        0b011_0111 => Instruction::LUI(ty),
        0b001_0111 => Instruction::AUIPC(ty),
        _ => return None,
    };
    Some(inst)
}

fn get_j_type(ty: JType, opcode: u8) -> Option<Instruction> {
    let inst = match opcode {
        0b110_1111 => Instruction::JAL(ty),
        _ => return None,
    };
    Some(inst)
}

/// Top level function for parsing a RV32I instruction.
pub fn parse(inst: u32) -> Option<Instruction> {
    // get the opcode from the first 6 bits
    let opcode = (inst & 0x7F) as u8;
    match InstructionType::from_opcode(opcode)? {
        InstructionType::R => {
            let (funct3, funct7, ty) = RType::parse(inst);
            get_r_type(ty, funct3, funct7)
        }
        // I-variant
        InstructionType::I => {
            let (funct3, ty) = IType::parse(inst);
            get_i_type(ty, opcode, funct3)
        }
        // S-variant
        InstructionType::S => {
            let (funct3, ty) = SType::parse(inst);
            get_s_type(ty, funct3)
        }
        // B-variant
        InstructionType::B => {
            let (funct3, ty) = BType::parse(inst);
            get_b_type(ty, funct3)
        }
        // U-variant
        InstructionType::U => {
            let ty = UType::parse(inst);
            get_u_type(ty, opcode)
        }
        // J-variant
        InstructionType::J => {
            let ty = JType::parse(inst);
            get_j_type(ty, opcode)
        }
    }
}

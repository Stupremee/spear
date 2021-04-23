//! Instruction decoding.

use super::{BType, IType, Instruction, InstructionType, JType, RType, SType, UType};

impl InstructionType {
    /// Find the associated instruction type for a specific opcode.
    pub fn from_opcode(opcode: u8) -> Option<Self> {
        match opcode {
            // R-variant
            0b011_0011 | 0b011_1011 => Some(Self::R),
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
                rd: rd as u8,
                rs1: rs1 as u8,
                rs2: rs2 as u8,
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
                rd: rd as u8,
                rs: rs as u8,
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
                rs1: rs1 as u8,
                rs2: rs2 as u8,
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
                rs1: rs1 as u8,
                rs2: rs2 as u8,
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
        let imm = inst & 0xFFFF_F000;

        UType {
            val: imm,
            rd: rd as u8,
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
            rd: rd as u8,
        }
    }
}

fn get_r_type(_ty: RType, _funct3: u8, _funct7: u8) -> Option<Instruction> {
    None
}

/// Top level function for parsing a RV32I instruction.
pub fn parse(inst: u32) -> Option<Instruction> {
    // get the opcode from the first 6 bits
    let opcode = inst & 0x3FF;
    match InstructionType::from_opcode(opcode as u8)? {
        InstructionType::R => {
            let _ty = RType::parse(inst);
        }
        // I-variant
        InstructionType::I => {
            let _ty = IType::parse(inst);
        }
        // S-variant
        InstructionType::S => {
            let _ty = SType::parse(inst);
        }
        // B-variant
        InstructionType::B => {
            let _ty = BType::parse(inst);
        }
        // U-variant
        InstructionType::U => {
            let _ty = UType::parse(inst);
        }
        // J-variant
        InstructionType::J => {
            let _ty = JType::parse(inst);
        }
    }

    None
}

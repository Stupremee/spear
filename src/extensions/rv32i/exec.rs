//! Exeuction engine for RV32I instructions.

use super::{Extension, Instruction};
use crate::Address;

/// Execute a RV32I instruction on the given cpu.
pub fn exec(ext: &mut Extension, inst: Instruction) {
    match inst {
        Instruction::LUI(op) => ext.write_register(op.rd, Address::from(op.imm())),
        Instruction::AUIPC(op) => {
            let pc = ext.get_pc() + op.imm();
            ext.set_pc(pc);
        }

        Instruction::JAL(op) => {
            let pc = ext.get_pc();
            let target = pc + op.sign_imm() as u32;

            // jump target must be aligned to 4 byte boundary
            if target & 3u32 != 0u32.into() {
                panic!("unaligned jump");
            }

            ext.set_pc(target);

            // add 4 to point to the next instruction
            ext.write_register(op.rd, pc + 4u32);
        }
        Instruction::JALR(op) => {
            let pc = ext.get_pc();

            let target = ext.read_register(op.rs) + op.sign_imm() as u32;
            let target = target & !1u32;

            // jump target must be aligned to 4 byte boundary
            if target & 3u32 != 0u32.into() {
                panic!("unaligned jump");
            }

            ext.set_pc(target);

            // add 4 to point to the next instruction
            ext.write_register(op.rd, pc + 4u32);
        }

        Instruction::BEQ(op) => branch(ext, op, |a, b| a == b),
        Instruction::BNE(op) => branch(ext, op, |a, b| a != b),
        Instruction::BLT(op) => branch(ext, op, |a, b| a.signed() < b.signed()),
        Instruction::BGE(op) => branch(ext, op, |a, b| a.signed() >= b.signed()),
        Instruction::BLTU(op) => branch(ext, op, |a, b| a < b),
        Instruction::BGEU(op) => branch(ext, op, |a, b| a >= b),

        Instruction::ADDI(op) => {
            let src = ext.read_register(op.rs);
            let val = src + op.sign_imm() as u32;
            ext.write_register(op.rd, val);
        }
        _ => todo!(),
    }
}

fn branch<F: FnOnce(Address, Address) -> bool>(ext: &mut Extension, op: super::BType, cond: F) {
    if cond(ext.read_register(op.rs1), ext.read_register(op.rs2)) {
        let target = ext.get_pc() + op.sign_imm() as u32;

        // jump target must be aligned to 4 byte boundary
        if target & 3u32 != 0u32.into() {
            panic!("unaligned jump");
        }

        ext.set_pc(target);
    }
}

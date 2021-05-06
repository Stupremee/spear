//! Exeuction engine for RV32I instructions.

use super::{Extension, Instruction, Register};
use crate::{cpu, Address};

/// Execute a RV32I instruction on the given cpu.
pub fn exec(inst: Instruction, mut cpu: cpu::CpuOrExtension<'_, Extension>) {
    let ext = cpu.ext();

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

        Instruction::LB(op) => {
            let addr = ext.read_register(op.rs) + op.sign_imm() as u32;
            let read = cpu.cpu().read::<u8>(addr).expect("trap: failed to read memory");
            cpu.ext().write_register(op.rd, Address::from(read as u32));
        }

        Instruction::ADDI(op) => imm_inst(ext, op.rs, op.rd, |x| x + op.sign_imm() as u32),
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

fn imm_inst<F: FnOnce(Address) -> Address>(ext: &mut Extension, rs: Register, rd: Register, op: F) {
    let src = ext.read_register(rs);
    ext.write_register(rd, op(src));
}

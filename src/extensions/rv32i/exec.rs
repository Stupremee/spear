//! Exeuction engine for RV32I instructions.

use bytemuck::Pod;

use super::{Extension, Instruction, Register};
use crate::{cpu, Address, Continuation};

/// Execute a RV32I instruction on the given cpu.
pub fn exec(inst: Instruction, mut cpu: cpu::CpuOrExtension<'_, Extension>) -> Continuation {
    let ext = cpu.ext();

    match dbg!(inst) {
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

            return Continuation::Jump;
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

            return Continuation::Jump;
        }

        Instruction::BEQ(op) => return branch(ext, op, |a, b| a == b),
        Instruction::BNE(op) => return branch(ext, op, |a, b| a != b),
        Instruction::BLT(op) => return branch(ext, op, |a, b| a.signed() < b.signed()),
        Instruction::BGE(op) => return branch(ext, op, |a, b| a.signed() >= b.signed()),
        Instruction::BLTU(op) => return branch(ext, op, |a, b| a < b),
        Instruction::BGEU(op) => return branch(ext, op, |a, b| a >= b),

        Instruction::LB(op) => load_inst::<u8>(op, cpu),
        Instruction::LW(op) => load_inst::<u32>(op, cpu),
        Instruction::LH(op) => load_inst::<u16>(op, cpu),

        Instruction::ADDI(op) => imm_inst(ext, op.rs, op.rd, |x| x + op.sign_imm() as u32),
        _ => todo!(),
    }

    Continuation::Next
}

fn branch<F: FnOnce(Address, Address) -> bool>(
    ext: &mut Extension,
    op: super::BType,
    cond: F,
) -> Continuation {
    if cond(ext.read_register(op.rs1), ext.read_register(op.rs2)) {
        let target = ext.get_pc() + op.sign_imm() as u32;

        // jump target must be aligned to 4 byte boundary
        if target & 3u32 != 0u32.into() {
            panic!("unaligned jump");
        }

        ext.set_pc(target);
        Continuation::Jump
    } else {
        Continuation::Next
    }
}

fn imm_inst<F: FnOnce(Address) -> Address>(ext: &mut Extension, rs: Register, rd: Register, op: F) {
    let src = ext.read_register(rs);
    ext.write_register(rd, op(src));
}

fn load_inst<T: Pod>(op: super::IType, mut cpu: cpu::CpuOrExtension<'_, Extension>)
where
    u32: From<T>,
{
    let addr = cpu.ext().read_register(op.rs) + op.sign_imm() as u32;
    let read = cpu
        .cpu()
        .read::<T>(addr)
        .expect("trap: failed to read memory");
    cpu.ext()
        .write_register(op.rd, Address::from(u32::from(read)));
}

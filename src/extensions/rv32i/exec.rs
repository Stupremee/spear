//! Exeuction engine for RV32I instructions.

use bytemuck::Pod;

use super::{Extension, Instruction, Register};
use crate::{cpu, Address, Continuation};

/// Execute a RV32I instruction on the given cpu.
pub fn exec(inst: Instruction, mut cpu: cpu::CpuOrExtension<'_, Extension>) -> Continuation {
    let ext = cpu.ext();

    println!("executing {}", inst);
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

        Instruction::LB(op) | Instruction::LBU(op) => {
            load_inst::<u8, _>(op, cpu, |x| Address::from(x as u32))
        }
        Instruction::LH(op) => load_inst::<u16, _>(op, cpu, |x| {
            let x = x as u32 as i32;
            let x = (x << 16) >> 16;
            Address::from(x as u32)
        }),
        Instruction::LHU(op) => load_inst::<u16, _>(op, cpu, |x| Address::from(x as u32)),
        Instruction::LW(op) => load_inst::<u32, _>(op, cpu, Address::from),

        Instruction::SB(op) => store_inst(op, cpu, |x| x as u8),
        Instruction::SH(op) => store_inst(op, cpu, |x| x as u16),
        Instruction::SW(op) => store_inst(op, cpu, |x| x as u32),

        Instruction::ADDI(op) => imm_inst(ext, op.rs, op.rd, |x| x + op.sign_imm() as u32),
        Instruction::SLTI(op) => imm_inst(ext, op.rs, op.rd, |x| {
            if (u64::from(x) as i64) < op.sign_imm() as i64 {
                1u32.into()
            } else {
                0u32.into()
            }
        }),
        Instruction::SLTIU(op) => imm_inst(ext, op.rs, op.rd, |x| {
            if u64::from(x) < op.sign_imm() as u32 as u64 {
                1u32.into()
            } else {
                0u32.into()
            }
        }),
        Instruction::XORI(op) => imm_inst(ext, op.rs, op.rd, |x| x ^ op.sign_imm() as u32),
        Instruction::ORI(op) => imm_inst(ext, op.rs, op.rd, |x| x | op.sign_imm() as u32),
        Instruction::ANDI(op) => imm_inst(ext, op.rs, op.rd, |x| x & op.sign_imm() as u32),
        Instruction::SLLI(op) => imm_inst(ext, op.rs, op.rd, |x| x << op.shamt() as u32),
        Instruction::SRLI(op) => imm_inst(ext, op.rs, op.rd, |x| x >> op.shamt() as u32),
        Instruction::SRAI(op) => imm_inst(ext, op.rs, op.rd, |x| {
            let x = x.signed() >> op.shamt() as u32;
            x.unsigned()
        }),

        Instruction::ADD(op) => reg_inst(ext, op, |a, b| a + b),
        Instruction::SUB(op) => reg_inst(ext, op, |a, b| a - b),
        Instruction::SLL(op) => reg_inst(ext, op, |a, b| a << u64::from(b & 0x1Fu32) as u32),
        Instruction::SLT(op) => reg_inst(ext, op, |a, b| {
            if a.signed() < b.signed() {
                1u32.into()
            } else {
                0u32.into()
            }
        }),
        Instruction::SLTU(op) => reg_inst(
            ext,
            op,
            |a, b| {
                if a < b {
                    1u32.into()
                } else {
                    0u32.into()
                }
            },
        ),
        Instruction::XOR(op) => reg_inst(ext, op, |a, b| a ^ b),
        Instruction::SRL(op) => reg_inst(ext, op, |a, b| a >> u64::from(b & 0x1Fu32) as u32),
        Instruction::SRA(op) => reg_inst(ext, op, |a, b| {
            let x = a.signed() >> u64::from(b & 0x1Fu32) as u32;
            x.unsigned()
        }),
        Instruction::OR(op) => reg_inst(ext, op, |a, b| a | b),
        Instruction::AND(op) => reg_inst(ext, op, |a, b| a & b),
        // we are not real hardware, so we dont need fences
        Instruction::FENCE(_) => {}
        inst => todo!("{}", inst),
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

fn reg_inst<F: FnOnce(Address, Address) -> Address>(ext: &mut Extension, op: super::RType, f: F) {
    let rs1 = ext.read_register(op.rs1);
    let rs2 = ext.read_register(op.rs2);
    ext.write_register(op.rd, f(rs1, rs2));
}

fn imm_inst<F: FnOnce(Address) -> Address>(ext: &mut Extension, rs: Register, rd: Register, op: F) {
    let src = ext.read_register(rs);
    ext.write_register(rd, op(src));
}

fn load_inst<T: Pod, F: FnOnce(T) -> Address>(
    op: super::IType,
    mut cpu: cpu::CpuOrExtension<'_, Extension>,
    conv: F,
) {
    let addr = cpu.ext().read_register(op.rs) + op.sign_imm() as u32;
    let read = cpu
        .cpu()
        .read::<T>(addr)
        .expect("trap: failed to read memory");
    cpu.ext().write_register(op.rd, conv(read));
}

fn store_inst<T: Pod, F: FnOnce(u64) -> T>(
    op: super::SType,
    mut cpu: cpu::CpuOrExtension<'_, Extension>,
    conv: F,
) {
    let addr = cpu.ext().read_register(op.rs1) + op.sign_imm() as u32;
    let value = u64::from(cpu.ext().read_register(op.rs2));
    let value = conv(value);

    cpu.cpu()
        .write(addr, value)
        .expect("failed to write memory");
}

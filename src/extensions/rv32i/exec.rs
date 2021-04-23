//! Exeuction engine for RV32I instructions.

use super::{Extension, Instruction};
use crate::Address;

/// Execute a RV32I instruction on the given cpu.
pub fn exec(ext: &mut Extension, inst: Instruction) {
    match inst {
        Instruction::ADDI(ty) => {
            let src = ext.read_register(ty.rs);

            let val = Address::from(ty.sign_val() as u64);
            let val = src + val;

            ext.write_register(ty.rd, val);
        }
        _ => todo!(),
    }
}

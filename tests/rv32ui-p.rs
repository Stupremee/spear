//! Test that runs the RV32UI suite provided by riscv-tests

use spear::{cpu::PrivilegeMode, emulator::Emulator, extensions::rv32i::Register, Address};
use std::path::PathBuf;

macro_rules! register_tests {
    ($($name:ident),*$(,)?) => {
        $(#[test]
        fn $name() -> Result<(), Box<dyn std::error::Error>> {
            let mut root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
            root.push("tests/binaries/rv32ui-p");
            root.push(stringify!($name).replacen("_", "-", 2));

            let data = std::fs::read(root)?;
            let file = object::File::parse(data.as_slice())?;
            let mut emu = Emulator::from_object_with_htif(file)?;
            emu.run();

            // the result of the test is stored in the `a0` register
            assert_eq!(
                Address::from(0u32),
                emu.cpu().arch().base().read_register(Register::from(10))
            );
            assert_eq!(PrivilegeMode::Machine, emu.cpu().mode());

            Ok(())
        })*
    };
}

register_tests![
    rv32ui_p_add,
    rv32ui_p_addi,
    rv32ui_p_and,
    rv32ui_p_andi,
    rv32ui_p_auipc,
    rv32ui_p_beq,
    rv32ui_p_bge,
    rv32ui_p_bgeu,
    rv32ui_p_blt,
    rv32ui_p_bltu,
    rv32ui_p_bne,
    rv32ui_p_fence_i,
    rv32ui_p_jal,
    rv32ui_p_jalr,
    rv32ui_p_lb,
    rv32ui_p_lbu,
    rv32ui_p_lh,
    rv32ui_p_lhu,
    rv32ui_p_lui,
    rv32ui_p_lw,
    rv32ui_p_or,
    rv32ui_p_ori,
    rv32ui_p_sb,
    rv32ui_p_sh,
    rv32ui_p_simple,
    rv32ui_p_sll,
    rv32ui_p_slli,
    rv32ui_p_slt,
    rv32ui_p_slti,
    rv32ui_p_sltiu,
    rv32ui_p_sltu,
    rv32ui_p_sra,
    rv32ui_p_srai,
    rv32ui_p_srl,
    rv32ui_p_srli,
    rv32ui_p_sub,
    rv32ui_p_sw,
    rv32ui_p_xor,
    rv32ui_p_xori,
];

//! Test that runs the RV32MI suite provided by riscv-tests

use spear::{cpu::PrivilegeMode, emulator::Emulator, extensions::rv32i::Register, Address};
use std::path::PathBuf;

macro_rules! register_tests {
    ($($name:ident),*$(,)?) => {
        $(#[test]
        fn $name() -> Result<(), Box<dyn std::error::Error>> {
            let mut root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
            root.push("tests/binaries/rv32mi-p");
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
    // who needs breakpoints anyway
    // rv32mi_p_breakpoint,
    rv32mi_p_csr,
    rv32mi_p_illegal,
    rv32mi_p_ma_addr,
    rv32mi_p_ma_fetch,
    rv32mi_p_mcsr,
    rv32mi_p_sbreak,
    rv32mi_p_scall,
    rv32mi_p_shamt,
];

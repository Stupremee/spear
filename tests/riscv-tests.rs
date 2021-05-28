macro_rules! register_test_suite {
    ($suite:ident => $($name:ident),*$(,)?) => {
        pub mod $suite {
        use spear::{cpu::PrivilegeMode, emulator::Emulator, extensions::rv32i::Register, Address};
        use std::path::PathBuf;

        $(#[test]
        fn $name() -> Result<(), Box<dyn std::error::Error>> {
            let mut root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
            root.push("tests/binaries");
            root.push(stringify!($suite).replacen("_", "-", 2));
            root.push(concat!(stringify!($suite), "_", stringify!($name)).replacen("_", "-", 2));

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
        }
    };
}

register_test_suite![ rv32mi_p =>
    // who needs breakpoints anyway
    // breakpoint,

    // FIXME: Compares cycle count which is not implemented yet
    // csr,

    // `EBREAK` is not yet implemented
    // sbreak,

    illegal,
    scall,
    shamt,
    ma_addr,
    ma_fetch,
    mcsr,
];

register_test_suite![ rv32si_p =>
    // `EBREAK` is not yet implemented
    // sbreak,
    // FIXME: Requires virtual memory
    // dirty,

    csr,
    ma_fetch,
    scall,
    wfi,
];

register_test_suite![ rv32ui_p =>
    add,
    addi,
    and,
    andi,
    auipc,
    beq,
    bge,
    bgeu,
    blt,
    bltu,
    bne,
    fence_i,
    jal,
    jalr,
    lb,
    lbu,
    lh,
    lhu,
    lui,
    lw,
    or,
    ori,
    sb,
    sh,
    simple,
    sll,
    slli,
    slt,
    slti,
    sltiu,
    sltu,
    sra,
    srai,
    srl,
    srli,
    sub,
    sw,
    xor,
    xori,
];

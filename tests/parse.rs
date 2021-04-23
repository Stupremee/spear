//! Tests for decoding instructions.

macro_rules! test_instructions {
    ($($name:ident {
        $($raw:literal: $str:literal),*$(,)?
    }),*$(,)?) => {
        $(#[test]
        fn $name() {$(
            let inst = spear::extensions::rv32i::parse($raw).unwrap();
            assert_eq!(inst.to_string(), $str);
        )*})*
    };
}

test_instructions! {
    test_add_inst {
        0xFF010113: "addi sp, sp, -16",
        0x01010413: "addi s0, sp, 16",
        0x01010113: "addi sp, sp, 16",
    }
}

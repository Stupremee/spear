//! Tests for decoding instructions.

macro_rules! test_instructions {
    ($($name:ident {
        $($raw:literal: $str:literal),*$(,)?
    })*) => {
        $(#[test]
        fn $name() {$(
            let inst = spear::extensions::rv32i::parse($raw).unwrap();
            assert_eq!(inst.to_string(), $str);
        )*})*
    };
}

test_instructions! {
    test_lui_inst {
        0x000007B7: "lui a5, 0",
        0x00F0F5B7: "lui a1, 3855",
        0x05555637: "lui a2, 21845",
    }
    test_auipc_inst {
        0x0000F097: "auipc ra, 15",
        0x00019597: "auipc a1, 25",
        0x00017517: "auipc a0, 23",
    }
    test_jal_inst {
        0x73625F6F: "jal t5, 153398",
        0x69706D6F: "jal s10, 28310",
        0x00656E6F: "jal t3, 352262",
    }
    test_jalr_inst {
        0xA78080E7: "jalr ra, ra, -1416",
        0xE4A080E7: "jalr ra, ra, -438",
        0x464080E7: "jalr ra, ra, 1124",
    }
    test_beq_inst {
        0x09170A63: "beq a4, a7, 148",
        0x01178A63: "beq a5, a7, 20",
        0xFF0784E3: "beq a5, a6, -24",
    }
    test_bne_inst {
        0x08F71063: "bne a4, a5, 128",
        0x00F69E63: "bne a3, a5, 28",
    }
    test_blt_inst {
        0xFD07CFE3: "blt a5, a6, -34",
        0x00F84363: "blt a6, a5, 6",
    }
    test_bge_inst {
        0x1747D763: "bge a5, s4, 366",
        0x08A6DD63: "bge a3, a0, 154",
    }
    test_bltu_inst {
        0x02F56B63: "bltu a0, a5, 54",
        0xFAD76FE3: "bltu a4, a3, -66",
    }
    test_bgeu_inst {
        0x02F67A63: "bgeu a2, a5, 52",
        0x00F6FD63: "bgeu a3, a5, 26",
    }
    test_lb_inst {
        0x03048503: "lb a0, s1, 48",
    }
    test_addi_inst {
        0x03848493: "addi s1, s1, 56",
    }
    test_slli_inst {
        0x00499593: "slli a1, s3, 4",
    }
    test_srai_inst {
        0x4386D793: "srai a5, a3, 56",
    }
    test_add_inst {
        0x008506B3: "add a3, a0, s0",
    }
    test_fence_inst {
        0x0230000F: "fence",
    }
    test_ecall_inst {
        0x00000073: "ecall",
    }
    test_ebreak_inst {
        0x00100073: "ebreak",
    }
}

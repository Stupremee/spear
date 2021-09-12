macro_rules! instructions {
    (
    base($base:ident) [
        $($base_inst:ident ($base_inst_ty:ident) ),*$(,)?
    ]
    ) => {
        /// The instruction type containing every possible
        /// instruction, from every extension.
        #[derive(Debug)]
        #[allow(clippy::upper_case_acronyms)]
        #[allow(missing_docs)]
        pub enum Instruction {
            $($base_inst ($base_inst_ty),)*
        }

        impl Instruction {
            /// Return the instruction type of this instruction.
            pub fn inst_type(&self) -> $crate::instruction::InstructionType {
                match self {
                    $(Instruction::$base_inst(ty) => $crate::instruction::InstructionType::from(ty.clone()),)*
                }
            }
        }
    };
}

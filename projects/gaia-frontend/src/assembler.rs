use crate::exports::nyar::gaia_assembly::assembler::{Diagnostic, DisassembleResult, Guest, InstructionMetadata, TargetArch};

pub struct AssemblerImpl;

impl Guest for AssemblerImpl {
    fn assemble(source: String, target: TargetArch, optimize: bool, debug: bool) -> Result<(), ()> {
        todo!()
    }

    fn get_supported_targets() -> Vec<TargetArch> {
        todo!()
    }

    fn validate_syntax(source: String) -> Vec<Diagnostic> {
        todo!()
    }

    fn get_instruction_set(target: TargetArch) -> Vec<InstructionMetadata> {
        todo!()
    }

    fn disassemble(bytecode: Vec<u8>, target: TargetArch) -> DisassembleResult {
        todo!()
    }
}

use crate::exports::nyar::gaia_assembly::metadata::{Guest, PlatformInfo, ProgramMetadata, SymbolInfo, TargetArch};

pub struct MetadataImpl;

impl Guest for MetadataImpl {
    fn get_program_metadata(bytecode: Vec<u8>, target: TargetArch) -> ProgramMetadata {
        todo!()
    }

    fn get_symbol_info(bytecode: Vec<u8>, target: TargetArch) -> Vec<SymbolInfo> {
        todo!()
    }

    fn get_platform_info(target: TargetArch) -> PlatformInfo {
        todo!()
    }
}
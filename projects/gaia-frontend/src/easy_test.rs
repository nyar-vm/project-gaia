use crate::exports::nyar::gaia_assembly::easy_test::{Guest, TargetArch};

pub struct EasyTestImpl;

impl Guest for EasyTestImpl {
    fn generate_exit_code(code: u32, target: TargetArch) -> Vec<u8> {
        todo!()
    }

    fn generate_console_log(message: String, target: TargetArch) -> Vec<u8> {
        todo!()
    }
}
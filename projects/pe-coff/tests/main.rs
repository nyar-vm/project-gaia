use gaia_types::{helpers::Architecture, GaiaError};

#[cfg(target_os = "windows")]
mod windows;

#[test]
fn ready() {
    println!("it works!")
}

pub fn easy_exit_code(arch: Architecture, code: u32) -> Result<Vec<u8>, GaiaError> {
    todo!("Implement easy_exit_code for {:?} with code {}", arch, code)
}

pub fn easy_console_log(arch: Architecture, text: String) -> Result<Vec<u8>, GaiaError> {
    todo!("Implement easy_console_log for {:?} with text: {}", arch, text)
}

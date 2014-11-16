//! Exit Code PE 文件生成测试

use crate::test_tools::test_path;
use gaia_types::{helpers::Architecture, GaiaError};
use pe_assembler::{helpers::PeAssemblerBuilder, types::SubsystemType};
use std::process::Command;
use x86_64_assembler::{builder::ProgramBuilder, instruction::Register};

#[test]
fn test_exit_codes() -> Result<(), GaiaError> {
    let exit_codes = [0, 1, 42, 255, 114514, 1919810];
    let architectures = [Architecture::X86, Architecture::X86_64];

    for &code in &exit_codes {
        for arch in architectures.iter() {
            let pe_data = easy_exit_code(arch, code)?;
            run_pe_with_exit_code(&pe_data, code, arch)?;
        }
    }
    Ok(())
}

/// Helper function to run a PE file and get its exit code
fn run_pe_with_exit_code(pe_data: &[u8], expected_exit_code: i32, arch: &Architecture) -> Result<(), GaiaError> {
    let test_dir = test_path("generated");
    let exe_path = match arch {
        Architecture::X86 => test_dir.join(format!("exit_{}_x86.exe", expected_exit_code)),
        Architecture::X86_64 => test_dir.join(format!("exit_{}_x64.exe", expected_exit_code)),
        _ => panic!("不支持架构 {}", arch),
    };

    std::fs::write(&exe_path, pe_data)?;
    let output = Command::new(&exe_path).output()?;
    let exit_code = output.status.code().unwrap_or(-1);
    assert_eq!(exit_code, expected_exit_code, "Exit code mismatch for {:?}", exe_path);
    Ok(())
}

pub fn easy_exit_code(arch: &Architecture, exit_code: i32) -> Result<Vec<u8>, GaiaError> {
    match arch {
        Architecture::X86 => generate_exit_code_x86(exit_code),
        Architecture::X86_64 => generate_exit_code_x64(exit_code),
        _ => Err(GaiaError::unsupported_architecture(arch.clone())),
    }
}

fn generate_exit_code_x86(exit_code: i32) -> Result<Vec<u8>, GaiaError> {
    // 使用 ProgramBuilder 生成 x86：push exit_code; call [IAT]
    let mut program = ProgramBuilder::new(Architecture::X86);
    program.push_imm(exit_code as i64)?;
    program.call_indirect(0)?; // IAT 占位符，构建器后续修补
    let code = program.compile_instructions()?;

    PeAssemblerBuilder::new()
        .architecture(Architecture::X86)
        .subsystem(SubsystemType::Console)
        .entry_point(0x1000)
        .image_base(0x400000)
        .import_function("kernel32.dll", "ExitProcess")
        .code(code)
        .generate()
}

fn generate_exit_code_x64(exit_code: i32) -> Result<Vec<u8>, GaiaError> {
    let mut program = ProgramBuilder::new(Architecture::X86_64);
    // Windows x64：为被调函数预留 32 字节 shadow space，并保持对齐
    // 使用 sub rsp, 28h（40 字节：32 字节 shadow space + 8 字节对齐补偿）
    program.sub_reg_imm(Register::RSP, 0x28)?;
    // Windows x64：第一个参数放入 RCX（使用 64 位寄存器名称）
    program.mov_reg_imm(Register::RCX, exit_code as i64)?;
    program.call_indirect(0)?; // IAT 占位符，构建器后续修补
    let code = program.compile_instructions()?;

    PeAssemblerBuilder::new()
        .architecture(Architecture::X86_64)
        .subsystem(SubsystemType::Console)
        .entry_point(0x1000)
        .image_base(0x140000000)
        .import_function("kernel32.dll", "ExitProcess")
        .code(code)
        .generate()
}

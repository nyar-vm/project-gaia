//! 在 Windows 上生成并执行一个实际可运行的 Hello World（无硬编码地址）

use crate::test_tools::test_path;
use gaia_types::{helpers::Architecture, GaiaError};
use pe_assembler::{helpers::PeAssemblerBuilder, types::SubsystemType};
use std::{
    fs::{create_dir_all, write},
    process::Command,
};
use x86_64_assembler::{
    builder::ProgramBuilder,
    instruction::{Instruction, Operand, Register},
};

#[test]
fn test_hello_world() -> Result<(), GaiaError> {
    // 通过 easy_console_log 生成并保存 echo_hello_world_x86.exe 与 echo_hello_world_x64.exe
    let generated_dir = test_path("generated");
    create_dir_all(&generated_dir).ok();
    let msg = "Hello world!\n";

    // X86
    let x86_bytes = easy_console_log(Architecture::X86, msg).expect("Failed to generate x86 hello world PE bytes");
    let x86_path = generated_dir.join("echo_hello_world_x86.exe");
    write(&x86_path, &x86_bytes).expect("Failed to write x86 exe");
    let x86_out = Command::new(&x86_path).output().expect("Failed to execute generated x86 PE file");
    assert!(x86_out.status.success(), "x86 process did not exit with code 0: {:?}", x86_out.status);
    let x86_stdout = String::from_utf8_lossy(&x86_out.stdout);
    assert!(x86_stdout.contains("Hello world"), "x86 stdout does not contain 'hello world': {}", x86_stdout);

    // X64
    let x64_bytes = easy_console_log(Architecture::X86_64, msg).expect("Failed to generate x64 hello world PE bytes");
    let x64_path = generated_dir.join("echo_hello_world_x64.exe");
    write(&x64_path, &x64_bytes).expect("Failed to write x64 exe");
    let x64_out = Command::new(&x64_path).output().expect("Failed to execute generated x64 PE file");
    assert!(x64_out.status.success(), "x64 process did not exit with code 0: {:?}", x64_out.status);
    let x64_stdout = String::from_utf8_lossy(&x64_out.stdout);
    assert!(x64_stdout.contains("Hello world"), "x64 stdout does not contain 'hello world': {}", x64_stdout);
    Ok(())
}

pub fn easy_console_log(arch: Architecture, message: &str) -> Result<Vec<u8>, GaiaError> {
    match arch {
        Architecture::X86 => generate_console_log_x86(message),
        Architecture::X86_64 => generate_console_log_x64(message),
        _ => Err(GaiaError::unsupported_architecture(arch)),
    }
}

fn generate_console_log_x86(message: &str) -> Result<Vec<u8>, GaiaError> {
    let mut program = ProgramBuilder::new(Architecture::X86);

    // GetStdHandle(STD_OUTPUT_HANDLE = -11)
    program.push_imm(-11)?.call_indirect(0)?;

    // WriteFile(handle=EAX, lpBuffer=&msg, nBytes=len, lpNumberOfBytesWritten=&written, lpOverlapped=NULL)
    // 为避免依赖固定 image_base（ASLR），动态计算 &written：先修补并获取 msg 地址，再偏移 (len + 1)
    let msg_len = message.len() as i64;
    program
        // 为了让修补器将 msg 指针正确替换到 .data 开头：先 push len，再 push label
        .push_imm(msg_len) // 记录长度，供修补器识别后续的 msg 占位
        ?
        .push_label("msg".to_string()) // msg 的占位，将被修补为 .data 起始地址
        ?
        .pop_reg(Register::EBX) // EBX = &msg
        ?
        .pop_reg(Register::ECX) // 弹出用于修补的 msg_len，占位不再作为参数
        ?
        .mov_reg_reg(Register::EDI, Register::EBX) // EDI = &msg（保留一份用于 lpBuffer）
        ?
        .add_reg_imm(Register::EBX, (msg_len + 1) as i64) // EBX = &written（位于 NUL 之后的 4 字节）
        ?
        // x86 调用约定：参数从右到左压栈 (lpOverlapped, lpNumberOfBytesWritten, nNumberOfBytesToWrite, lpBuffer, hFile)
        .push_imm(0) // lpOverlapped (第5个参数)
        ?
        .push_reg(Register::EBX) // lpNumberOfBytesWritten = &written (第4个参数)
        ?
        .push_imm(msg_len) // nNumberOfBytesToWrite = len (第3个参数)
        ?
        .push_reg(Register::EDI) // lpBuffer = &msg (第2个参数)
        ?
        .push_reg(Register::EAX) // hFile (第1个参数)
        ?
        .call_indirect(1) // WriteFile
        ?;

    // ExitProcess(0)
    program.push_imm(0)?.call_indirect(2)?.ret()?;

    let code = program.compile_instructions()?;

    // 数据段：以 0 结尾的字符串 + 4 字节用于 written 计数
    let mut data = Vec::new();
    data.extend_from_slice(message.as_bytes());
    data.push(0);
    data.extend_from_slice(&[0, 0, 0, 0]);

    PeAssemblerBuilder::new()
        .architecture(Architecture::X86)
        .subsystem(SubsystemType::Console)
        .entry_point(0x1000)
        .image_base(0x400000)
        .import_functions("kernel32.dll", &["GetStdHandle", "WriteFile", "ExitProcess"])
        .code(code)
        .data(data)
        .generate()
}

fn generate_console_log_x64(message: &str) -> Result<Vec<u8>, GaiaError> {
    let mut program = ProgramBuilder::new(Architecture::X86_64);

    // 设置栈空间：shadow space (32 bytes) + 对齐补偿 (8 bytes) = 40 bytes
    // Windows x64 要求 CALL 前保持 16 字节对齐，CALL 会压栈 8 字节返回地址。
    // 因此调用点通常使用 sub rsp, 40 以满足对齐要求。
    program.sub_reg_imm(Register::RSP, 40)?;

    // GetStdHandle(STD_OUTPUT_HANDLE = -11)
    program.mov_reg_imm(Register::RCX, -11)?;
    // 调用 GetStdHandle（已预留 shadow space 与对齐）
    program.call_indirect(0)?;

    // mov rcx, rax  (hFile = 返回句柄)
    program.mov_reg_reg(Register::RCX, Register::RAX)?;

    // lea rdx, [rip+disp32]  (lpBuffer = msg)
    program.add_instruction(Instruction::Lea { dst: Register::RDX, displacement: 0, rip_relative: true });

    // mov r8d, len  (nNumberOfBytesToWrite) - DWORD 类型，使用 32 位寄存器
    let len = message.len() as i64;
    program.mov_reg_imm(Register::R8D, len)?;

    // lea r9, [rip+disp32]  (lpNumberOfBytesWritten)
    program.add_instruction(Instruction::Lea { dst: Register::R9, displacement: 0, rip_relative: true });

    // 为 WriteFile 的第5个参数 lpOverlapped 设置为 NULL（位于 [rsp+32]）
    program.mov_reg_imm(Register::RAX, 0)?;
    program.add_instruction(Instruction::Mov {
        dst: Operand::Mem { base: Some(Register::RSP), index: None, scale: 1, displacement: 32 },
        src: Operand::Reg(Register::RAX),
    });

    // 调用 WriteFile（shadow space 已存在，且已设置第5参数）
    program.call_indirect(1)?;

    // ExitProcess(0)
    program.mov_reg_imm(Register::RCX, 0)?;
    program.call_indirect(2)?;

    // 恢复栈（如果 ExitProcess 返回）
    program.add_reg_imm(Register::RSP, 40)?;

    program.ret()?;

    let code = program.compile_instructions()?;

    // 数据段：字符串 + NUL + 对齐到4字节 + 写入计数（4字节）
    let mut data = Vec::new();
    data.extend_from_slice(message.as_bytes());
    data.push(0);
    // 对齐到 4 字节，确保 LPDWORD 指向 4 字节对齐的区域
    let pad = (4 - (data.len() % 4)) % 4;
    for _ in 0..pad {
        data.push(0);
    }
    data.extend_from_slice(&[0u8; 4]);

    PeAssemblerBuilder::new()
        .architecture(Architecture::X86_64)
        .subsystem(SubsystemType::Console)
        .entry_point(0x1000)
        .image_base(0x140000000)
        .import_functions("kernel32.dll", &["GetStdHandle", "WriteFile", "ExitProcess"])
        .code(code)
        .data(data)
        .generate()
}

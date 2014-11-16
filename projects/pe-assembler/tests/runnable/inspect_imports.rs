//! 打印 hello_world_x86.exe 的导入表细节，用于对比 Windows 正常加载的 PE

use crate::test_tools::test_path;
use gaia_types::helpers::Architecture;
use pe_assembler::{helpers::PeAssemblerBuilder, types::SubsystemType};
use std::fs::{create_dir_all, write};
use x86_64_assembler::{builder::ProgramBuilder, instruction::Register};

#[test]
fn inspect_hello_world_imports() {
    // 生成与 test_execute_hello_world_runnable 完全相同的 PE
    let mut program = ProgramBuilder::new(Architecture::X86);
    program.push_imm(-11).unwrap().call_indirect(0).unwrap();
    const MSG: &str = "hello world\n";
    let data_base: u32 = 0x402000;
    let written_addr: u32 = data_base + (MSG.len() as u32) + 1;
    program
        .push_imm(0)
        .unwrap()
        .push_imm(written_addr as i64)
        .unwrap()
        .push_imm(MSG.len() as i64)
        .unwrap()
        .push_label("msg".to_string())
        .unwrap()
        .push_reg(Register::EAX)
        .unwrap()
        .call_indirect(1)
        .unwrap()
        .push_imm(0)
        .unwrap()
        .call_indirect(2)
        .unwrap()
        .ret()
        .unwrap();
    let code = program.compile_instructions().unwrap();
    let mut data = Vec::new();
    data.extend_from_slice(MSG.as_bytes());
    data.push(0);
    data.extend_from_slice(&[0, 0, 0, 0]);
    let pe_bytes = PeAssemblerBuilder::new()
        .architecture(Architecture::X86)
        .subsystem(SubsystemType::Console)
        .entry_point(0x1000)
        .image_base(0x400000)
        .import_functions("kernel32.dll", &["GetStdHandle", "WriteFile", "ExitProcess"])
        .code(code)
        .data(data)
        .generate()
        .unwrap();

    let generated_dir = test_path("generated");
    create_dir_all(&generated_dir).ok();
    let exe_path = generated_dir.join("hello_world_x86.exe");
    write(&exe_path, &pe_bytes).unwrap();

    // 手动解析 PE 头找导入表 RVA
    let pe_offset = u32::from_le_bytes([pe_bytes[60], pe_bytes[61], pe_bytes[62], pe_bytes[63]]) as usize;
    let opt_offset = pe_offset + 24;
    let data_dir_offset = opt_offset + 96; // PE32: 96 起为 DataDirectory
    let import_rva = u32::from_le_bytes([
        pe_bytes[data_dir_offset + 8],
        pe_bytes[data_dir_offset + 9],
        pe_bytes[data_dir_offset + 10],
        pe_bytes[data_dir_offset + 11],
    ]);
    let import_size = u32::from_le_bytes([
        pe_bytes[data_dir_offset + 12],
        pe_bytes[data_dir_offset + 13],
        pe_bytes[data_dir_offset + 14],
        pe_bytes[data_dir_offset + 15],
    ]);
    println!("[!] Import Directory RVA: 0x{:08X}, Size: 0x{:08X}", import_rva, import_size);

    // 通过 Section Header 找 RVA 对应的文件偏移
    let sect_table_offset = opt_offset + 224; // PE32 OptionalHeader 大小 224
    let mut file_offset = 0usize;
    for i in 0..16 {
        let sect_rva = u32::from_le_bytes([
            pe_bytes[sect_table_offset + i * 40 + 12],
            pe_bytes[sect_table_offset + i * 40 + 13],
            pe_bytes[sect_table_offset + i * 40 + 14],
            pe_bytes[sect_table_offset + i * 40 + 15],
        ]);
        let sect_raw = u32::from_le_bytes([
            pe_bytes[sect_table_offset + i * 40 + 20],
            pe_bytes[sect_table_offset + i * 40 + 21],
            pe_bytes[sect_table_offset + i * 40 + 22],
            pe_bytes[sect_table_offset + i * 40 + 23],
        ]);
        if import_rva >= sect_rva && import_rva < sect_rva + 0x1000 {
            file_offset = (import_rva - sect_rva + sect_raw) as usize;
            break;
        }
    }
    println!("[!] Import Directory FileOffset: 0x{:08X}", file_offset);

    // 打印前 3 个 IMAGE_IMPORT_DESCRIPTOR（每个 20 字节）
    for i in 0..2 {
        let off = file_offset + i * 20;
        let orig_first_thunk = u32::from_le_bytes([pe_bytes[off], pe_bytes[off + 1], pe_bytes[off + 2], pe_bytes[off + 3]]);
        let time_date_stamp = u32::from_le_bytes([pe_bytes[off + 4], pe_bytes[off + 5], pe_bytes[off + 6], pe_bytes[off + 7]]);
        let forwarder_chain =
            u32::from_le_bytes([pe_bytes[off + 8], pe_bytes[off + 9], pe_bytes[off + 10], pe_bytes[off + 11]]);
        let name_rva = u32::from_le_bytes([pe_bytes[off + 12], pe_bytes[off + 13], pe_bytes[off + 14], pe_bytes[off + 15]]);
        let first_thunk = u32::from_le_bytes([pe_bytes[off + 16], pe_bytes[off + 17], pe_bytes[off + 18], pe_bytes[off + 19]]);
        println!("[IMPORT_DESC {}] OrigFirstThunk=0x{:08X} TimeDateStamp=0x{:08X} ForwarderChain=0x{:08X} NameRVA=0x{:08X} FirstThunk=0x{:08X}",
                 i, orig_first_thunk, time_date_stamp, forwarder_chain, name_rva, first_thunk);
        if orig_first_thunk == 0 && first_thunk == 0 {
            break;
        }

        // 打印 Name
        let name_file_off = rva_to_file_off(&pe_bytes, name_rva).unwrap_or(0);
        let name_len = pe_bytes[name_file_off..].iter().position(|&b| b == 0).unwrap_or(0);
        let name = std::str::from_utf8(&pe_bytes[name_file_off..name_file_off + name_len]).unwrap_or("<invalid>");
        println!("  -> DLL: {}", name);

        // 打印 INT（OrigFirstThunk）
        if orig_first_thunk != 0 {
            print_thunks("INT", &pe_bytes, orig_first_thunk);
        }
        // 打印 IAT（FirstThunk）
        if first_thunk != 0 {
            print_thunks("IAT", &pe_bytes, first_thunk);
        }
    }
}

fn rva_to_file_off(pe: &[u8], rva: u32) -> Option<usize> {
    let pe_offset = u32::from_le_bytes([pe[60], pe[61], pe[62], pe[63]]) as usize;
    let opt_offset = pe_offset + 24;
    let sect_table_offset = opt_offset + 224;
    for i in 0..16 {
        let sect_rva = u32::from_le_bytes([
            pe[sect_table_offset + i * 40 + 12],
            pe[sect_table_offset + i * 40 + 13],
            pe[sect_table_offset + i * 40 + 14],
            pe[sect_table_offset + i * 40 + 15],
        ]);
        let sect_raw = u32::from_le_bytes([
            pe[sect_table_offset + i * 40 + 20],
            pe[sect_table_offset + i * 40 + 21],
            pe[sect_table_offset + i * 40 + 22],
            pe[sect_table_offset + i * 40 + 23],
        ]);
        let sect_size = u32::from_le_bytes([
            pe[sect_table_offset + i * 40 + 8],
            pe[sect_table_offset + i * 40 + 9],
            pe[sect_table_offset + i * 40 + 10],
            pe[sect_table_offset + i * 40 + 11],
        ]);
        if rva >= sect_rva && rva < sect_rva + sect_size {
            return Some((rva - sect_rva + sect_raw) as usize);
        }
    }
    None
}

fn print_thunks(label: &str, pe: &[u8], rva: u32) {
    let off = rva_to_file_off(pe, rva).unwrap_or(0);
    println!("  -> {} RVA 0x{:08X} (file 0x{:08X})", label, rva, off);
    for i in 0..4 {
        let dw = u32::from_le_bytes([pe[off + i * 4], pe[off + i * 4 + 1], pe[off + i * 4 + 2], pe[off + i * 4 + 3]]);
        if dw == 0 {
            break;
        }
        if dw & 0x80000000 != 0 {
            println!("    [{}] Ordinal: 0x{:04X}", i, dw & 0xFFFF);
        }
        else {
            let name_off = rva_to_file_off(pe, dw).unwrap_or(0);
            let hint = u16::from_le_bytes([pe[name_off], pe[name_off + 1]]);
            let name_len = pe[name_off + 2..].iter().position(|&b| b == 0).unwrap_or(0);
            let name = std::str::from_utf8(&pe[name_off + 2..name_off + 2 + name_len]).unwrap_or("<invalid>");
            println!("    [{}] Hint=0x{:04X} Name: {}", i, hint, name);
        }
    }
}

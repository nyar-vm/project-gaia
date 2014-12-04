//! x64-only 导入表兼容模式测试：IAT 非零且指向有效 IMAGE_IMPORT_BY_NAME

use gaia_types::{helpers::Architecture, GaiaError};
use pe_assembler::{
    formats::exe::reader::ExeReader,
    helpers::{PeBuilder, PeReader},
    types::{PeSection, SubsystemType},
};
use std::io::Cursor;
use x86_64_assembler::{builder::ProgramBuilder, instruction::Register};

#[test]
fn test_x64_idata_compat_iat_points_to_hint_name() -> Result<(), GaiaError> {
    // 1) 生成最小 x64 EXE：ExitProcess(exit_code=0)，IAT 间接调用
    let exit_code = 0i32;
    let mut program = ProgramBuilder::new(Architecture::X86_64);
    program.sub_reg_imm(Register::RSP, 0x28)?; // shadow space + 对齐
    program.mov_reg_imm(Register::RCX, exit_code as i64)?; // RCX 传参
    program.call_indirect(0)?; // IAT 占位，构建器后续修补
    let code = program.compile_instructions()?;

    let pe_data = PeBuilder::new()
        .architecture(Architecture::X86_64)
        .subsystem(SubsystemType::Console)
        .entry_point(0x1000)
        .image_base(0x140000000)
        .import_function("kernel32.dll", "ExitProcess")
        .code(code)
        .generate()?;

    // 2) 解析程序结构，读取 IAT 目录范围
    let mut reader = ExeReader::new(Cursor::new(pe_data.clone()));
    let program = match reader.get_program() {
        Ok(p) => p,
        Err(e) => return Err(e),
    };

    let iat_dir = program.header.optional_header.data_directories.get(12).expect("IAT data directory missing");
    assert!(iat_dir.virtual_address != 0 && iat_dir.size >= 8, "IAT 目录必须有效且至少包含一个条目");

    // 3) 将 IAT 起始 RVA 映射到文件偏移，读取首个 IAT 条目（x64：8 字节）
    let iat_offset = rva_to_file_offset(iat_dir.virtual_address, &program.sections).expect("无法将 IAT RVA 映射到文件偏移");
    let entry_bytes: [u8; 8] = pe_data[iat_offset as usize..iat_offset as usize + 8].try_into().expect("IAT 首项读取失败");
    let iat_entry_qword = u64::from_le_bytes(entry_bytes);

    // 断言：IAT 首项非零，且为指向 IMAGE_IMPORT_BY_NAME（Hint+Name）的 RVA
    assert_ne!(iat_entry_qword, 0, "IAT 首项不应为 0");
    let hint_name_rva = iat_entry_qword as u32;

    // IAT 首项目标应位于 .idata 节范围内
    let idata = program.sections.iter().find(|s| s.name == ".idata").expect("缺少 .idata 节");
    assert!(
        hint_name_rva >= idata.virtual_address && hint_name_rva < idata.virtual_address + idata.virtual_size,
        "IAT 指向的 RVA 应落在 .idata 节内"
    );

    // 4) 读取 IMAGE_IMPORT_BY_NAME：校验 2 字节 Hint 与以 0 结尾的名称字符串
    let hint_name_offset = rva_to_file_offset(hint_name_rva, &program.sections).expect("无法将 Hint/Name RVA 映射到文件偏移");
    let hint_bytes: [u8; 2] =
        pe_data[hint_name_offset as usize..hint_name_offset as usize + 2].try_into().expect("Hint 读取失败");
    let hint = u16::from_le_bytes(hint_bytes);
    assert_eq!(hint, 0, "预期 Hint 为 0");

    let mut pos = hint_name_offset as usize + 2;
    let mut name_bytes = Vec::new();
    while pe_data[pos] != 0 {
        name_bytes.push(pe_data[pos]);
        pos += 1;
    }
    let func_name = String::from_utf8_lossy(&name_bytes).to_string();
    assert_eq!(func_name, "ExitProcess", "IAT 指向的名称应为 ExitProcess");

    Ok(())
}

fn rva_to_file_offset(rva: u32, sections: &[PeSection]) -> Option<u32> {
    for s in sections {
        if rva >= s.virtual_address && rva < s.virtual_address + s.virtual_size {
            let offset_in_section = rva - s.virtual_address;
            return Some(s.pointer_to_raw_data + offset_in_section);
        }
    }
    None
}

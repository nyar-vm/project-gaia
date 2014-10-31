use gaia_types::helpers::Architecture;
use pe_assembler::generator::easy_exit_code;
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("测试 pe-assembler 的 easy_exit_code 函数...");

    // 生成一个返回 42 的 PE 文件
    let pe_bytes = easy_exit_code(Architecture::X86_64, 42)?;

    println!("生成的 PE 文件大小: {} 字节", pe_bytes.len());

    // 保存到文件
    let output_path = "test_pe_output.exe";
    fs::write(output_path, pe_bytes)?;

    println!("已保存到: {}", output_path);

    Ok(())
}

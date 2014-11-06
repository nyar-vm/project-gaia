//! PE 汇编器主程序
//!
//! 用于生成 Hello World PE 可执行文件的示例程序

use pe_assembler::assembler::x64::code_builder::X64CodeBuilder;
use std::{fs::File, io::Write};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("生成 Hello World PE 文件...");

    // 生成 Hello World 程序的机器码
    let code = X64CodeBuilder::hello_world_program();

    // 写入到文件
    let mut file = File::create("hello_world.exe")?;
    file.write_all(&code)?;

    println!("Hello World PE 文件已生成: hello_world.exe");
    println!("文件大小: {} 字节", code.len());

    Ok(())
}

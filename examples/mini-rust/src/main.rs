//! Mini Rust 语言编译器
//!
//! 一个简化的 Rust 语言实现，支持编译到多个目标平台

use clap::{Parser, ValueEnum};
use gaia_assembler::{assembler::TargetPlatform, *};
use gaia_types::{helpers::Url, *};
use std::{fs, path::PathBuf};

mod ast;
mod codegen;
mod lexer;
mod parser;

use crate::codegen::MiniRustParser;

#[derive(Parser)]
#[command(name = "mini-rust")]
#[command(about = "Mini Rust 语言编译器")]
struct Cli {
    /// 输入的 mini-rust 源文件
    #[arg(value_name = "FILE")]
    input: PathBuf,

    /// 目标平台
    #[arg(short, long, value_enum, default_value_t = Target::All)]
    target: Target,

    /// 输出目录
    #[arg(short, long, default_value = "output")]
    output: PathBuf,

    /// 是否显示详细信息
    #[arg(short, long)]
    verbose: bool,
}

#[derive(Clone, ValueEnum)]
enum Target {
    /// 编译到所有平台
    All,
    /// .NET IL
    Il,
    /// Java Virtual Machine
    Jvm,
    /// Portable Executable
    Pe,
    /// WebAssembly
    Wasi,
}

impl From<Target> for Option<TargetPlatform> {
    fn from(target: Target) -> Self {
        match target {
            Target::All => None,
            Target::Il => Some(TargetPlatform::IL),
            Target::Jvm => Some(TargetPlatform::JVM),
            Target::Pe => Some(TargetPlatform::PE),
            Target::Wasi => Some(TargetPlatform::WASI),
        }
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // 读取源文件
    let source = fs::read_to_string(&cli.input).map_err(|e| {
        let url = Url::from_file_path(&cli.input.canonicalize().unwrap_or_else(|_| cli.input.clone()))
            .unwrap_or_else(|_| Url::parse(&format!("file://{}", cli.input.display())).unwrap());
        GaiaError::io_error(e, url)
    })?;

    if cli.verbose {
        println!("正在解析文件: {:?}", cli.input);
    }

    // 解析 mini-rust 源代码
    let program = MiniRustParser::parse(&source)?;

    if cli.verbose {
        println!("解析完成，生成的程序:");
        println!("{:#?}", program);
    }

    // 创建输出目录
    fs::create_dir_all(&cli.output).map_err(|e| GaiaError::io_error(e, Url::from_file_path(&cli.output).unwrap()))?;

    // 编译到目标平台
    match cli.target.into() {
        Some(target_platform) => {
            compile_to_single_platform(&program, target_platform, &cli.output, cli.verbose)?;
        }
        None => {
            compile_to_all_platforms(&program, &cli.output, cli.verbose)?;
        }
    }

    println!("编译完成！");
    Ok(())
}

fn compile_to_single_platform(
    program: &GaiaProgram,
    target: TargetPlatform,
    output_dir: &PathBuf,
    verbose: bool,
) -> Result<()> {
    if verbose {
        println!("正在编译到 {} 平台...", target.name());
    }

    let compiler = GaiaCompiler::new(target);
    let bytecode = compiler.compile(program)?;

    let output_file = output_dir.join(format!("output.{}", target.file_extension()));
    fs::write(&output_file, bytecode).map_err(|e| GaiaError::io_error(e, Url::from_file_path(&output_file).unwrap()))?;

    println!("已生成 {} 文件: {:?}", target.name(), output_file);
    Ok(())
}

fn compile_to_all_platforms(program: &GaiaProgram, output_dir: &PathBuf, verbose: bool) -> Result<()> {
    if verbose {
        println!("正在编译到所有平台...");
    }

    let results = GaiaCompiler::compile_all(program)?;

    for (platform, bytecode) in results {
        let output_file = output_dir.join(format!("output.{}", platform.file_extension()));
        fs::write(&output_file, bytecode).map_err(|e| GaiaError::io_error(e, Url::from_file_path(&output_file).unwrap()))?;

        println!("已生成 {} 文件: {:?}", platform.name(), output_file);
    }

    Ok(())
}

# ELF 后端入门指南

本指南将帮助您快速上手 Gaia 的 ELF 后端，从环境配置到创建第一个 Linux 可执行文件。

## 环境准备

### 系统要求

- **操作系统**: Linux (推荐 Ubuntu 20.04+, CentOS 8+, Arch Linux)
- **架构**: x86-64, ARM64, RISC-V (根据目标平台选择)
- **内存**: 至少 2GB RAM (推荐 4GB+)
- **存储**: 至少 1GB 可用空间

### Rust 工具链

```bash
# 安装 Rust (如果尚未安装)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# 验证安装
rustc --version
cargo --version

# 添加目标架构 (可选)
rustup target add x86_64-unknown-linux-gnu
rustup target add aarch64-unknown-linux-gnu
rustup target add riscv64gc-unknown-linux-gnu
```

### 项目依赖

在您的 `Cargo.toml` 中添加以下依赖：

```toml
[dependencies]
gaia-assembler = { path = "../gaia-assembler" }
gaia-types = { path = "../gaia-types" }
elf = "0.7"
byteorder = "1.4"

[dev-dependencies]
tempfile = "3.0"
```

### 开发工具 (可选但推荐)

```bash
# ELF 分析工具
sudo apt install binutils elfutils

# 调试工具
sudo apt install gdb valgrind strace

# 性能分析工具
sudo apt install perf linux-tools-generic

# 十六进制编辑器
sudo apt install hexdump xxd
```

## 第一个程序：Hello World

### 步骤 1: 创建项目

```bash
cargo new hello-elf --bin
cd hello-elf
```

### 步骤 2: 编写代码

创建 `src/main.rs`：

```rust
use gaia_assembler::elf::{ElfAssembler, WriterConfig, SectionType, SectionFlags};
use gaia_types::arch::x86_64::X86_64;
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 创建 ELF 汇编器
    let mut assembler = ElfAssembler::new_executable();

    // 设置目标架构和入口点
    assembler.set_machine(elf::EM_X86_64);
    assembler.set_class(elf::ELFCLASS64);
    assembler.set_entry_point(0x401000);

    // 添加代码段
    let text_section = assembler.add_section(".text")?;
    text_section.set_type(SectionType::ProgBits);
    text_section.set_flags(SectionFlags::ALLOC | SectionFlags::EXECINSTR);
    text_section.set_address(0x401000);

    // Hello World 系统调用代码
    let hello_code = vec![
        // write(1, message, 13)
        0x48, 0xc7, 0xc0, 0x01, 0x00, 0x00, 0x00,  // mov rax, 1 (sys_write)
        0x48, 0xc7, 0xc7, 0x01, 0x00, 0x00, 0x00,  // mov rdi, 1 (stdout)
        0x48, 0xc7, 0xc6, 0x00, 0x20, 0x40, 0x00,  // mov rsi, 0x402000 (message)
        0x48, 0xc7, 0xc2, 0x0d, 0x00, 0x00, 0x00,  // mov rdx, 13 (length)
        0x0f, 0x05,                                  // syscall

        // exit(0)
        0x48, 0xc7, 0xc0, 0x3c, 0x00, 0x00, 0x00,  // mov rax, 60 (sys_exit)
        0x48, 0xc7, 0xc7, 0x00, 0x00, 0x00, 0x00,  // mov rdi, 0 (exit code)
        0x0f, 0x05,                                  // syscall
    ];

    text_section.add_code(&hello_code)?;

    // 添加数据段
    let data_section = assembler.add_section(".data")?;
    data_section.set_type(SectionType::ProgBits);
    data_section.set_flags(SectionFlags::ALLOC | SectionFlags::WRITE);
    data_section.set_address(0x402000);
    data_section.add_string("Hello, World!\n")?;

    // 生成 ELF 文件
    let config = WriterConfig::default();
    let elf_data = assembler.build(config)?;

    // 保存到文件
    fs::write("hello", elf_data)?;

    // 设置可执行权限
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata("hello")?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions("hello", perms)?;
    }

    println!("ELF 文件已生成: hello");
    Ok(())
}
```

### 步骤 3: 构建和运行

```bash
# 构建生成器
cargo build --release

# 运行生成器
cargo run

# 验证生成的 ELF 文件
file hello
readelf -h hello

# 运行生成的程序
./hello
```

预期输出：

```
Hello, World!
```

## 创建共享库

### 步骤 1: 创建数学库

```rust
use gaia_assembler::elf::{ElfAssembler, WriterConfig, SymbolBinding, SymbolType};

fn create_math_library() -> Result<(), Box<dyn std::error::Error>> {
    let mut assembler = ElfAssembler::new_shared_object();

    // 设置 SONAME
    assembler.set_soname("libmath.so.1")?;

    // 添加代码段
    let text_section = assembler.add_section(".text")?;
    text_section.set_type(SectionType::ProgBits);
    text_section.set_flags(SectionFlags::ALLOC | SectionFlags::EXECINSTR);

    // add 函数: 返回两个参数的和
    let add_function = vec![
        0x48, 0x89, 0xf8,        // mov rax, rdi (第一个参数)
        0x48, 0x01, 0xf0,        // add rax, rsi (第二个参数)
        0xc3,                    // ret
    ];

    let add_offset = text_section.add_code(&add_function)?;

    // multiply 函数: 返回两个参数的乘积
    let mul_function = vec![
        0x48, 0x89, 0xf8,        // mov rax, rdi
        0x48, 0x0f, 0xaf, 0xc6,  // imul rax, rsi
        0xc3,                    // ret
    ];

    let mul_offset = text_section.add_code(&mul_function)?;

    // 添加导出符号
    assembler.add_symbol("add", add_offset, SymbolType::Function, SymbolBinding::Global)?;
    assembler.add_symbol("multiply", mul_offset, SymbolType::Function, SymbolBinding::Global)?;

    // 构建动态符号表
    assembler.build_dynamic_symbol_table()?;

    // 生成共享库
    let so_data = assembler.build(WriterConfig::default())?;
    std::fs::write("libmath.so", so_data)?;

    println!("共享库已生成: libmath.so");
    Ok(())
}
```

### 步骤 2: 使用共享库

```rust
fn create_client_program() -> Result<(), Box<dyn std::error::Error>> {
    let mut assembler = ElfAssembler::new_executable();
    assembler.set_entry_point(0x401000);

    // 添加对共享库的依赖
    assembler.add_needed_library("libmath.so")?;

    // 添加代码段
    let text_section = assembler.add_section(".text")?;

    // 调用共享库函数的代码
    let main_code = vec![
        // 设置参数
        0x48, 0xc7, 0xc7, 0x05, 0x00, 0x00, 0x00,  // mov rdi, 5
        0x48, 0xc7, 0xc6, 0x03, 0x00, 0x00, 0x00,  // mov rsi, 3

        // 调用 add 函数 (通过 PLT)
        0xe8, 0x00, 0x00, 0x00, 0x00,              // call add@plt

        // 退出程序
        0x48, 0x89, 0xc7,                          // mov rdi, rax (结果作为退出码)
        0x48, 0xc7, 0xc0, 0x3c, 0x00, 0x00, 0x00,  // mov rax, 60 (sys_exit)
        0x0f, 0x05,                                // syscall
    ];

    text_section.add_code(&main_code)?;

    // 添加 PLT 和 GOT
    assembler.create_plt_entry("add")?;
    assembler.create_got_entry("add")?;

    let elf_data = assembler.build(WriterConfig::default())?;
    std::fs::write("client", elf_data)?;

    println!("客户端程序已生成: client");
    Ok(())
}
```

## 验证生成的文件

### 使用 readelf 分析

```bash
# 查看 ELF 头信息
readelf -h hello

# 查看程序头表
readelf -l hello

# 查看节区头表
readelf -S hello

# 查看符号表
readelf -s libmath.so

# 查看动态段
readelf -d client
```

### 使用 objdump 反汇编

```bash
# 反汇编代码段
objdump -d hello

# 查看所有节区
objdump -s hello

# 查看重定位信息
objdump -r client
```

### 集成测试

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command;
    use tempfile::TempDir;

    #[test]
    fn test_hello_world_execution() {
        let temp_dir = TempDir::new().unwrap();
        let hello_path = temp_dir.path().join("hello");

        // 生成 ELF 文件
        create_hello_world(&hello_path).unwrap();

        // 执行并验证输出
        let output = Command::new(&hello_path)
            .output()
            .expect("Failed to execute hello");

        assert!(output.status.success());
        assert_eq!(output.stdout, b"Hello, World!\n");
    }

    #[test]
    fn test_elf_structure() {
        let temp_dir = TempDir::new().unwrap();
        let elf_path = temp_dir.path().join("test.elf");

        create_hello_world(&elf_path).unwrap();

        // 验证 ELF 魔数
        let elf_data = std::fs::read(&elf_path).unwrap();
        assert_eq!(&elf_data[0..4], b"\x7fELF");

        // 验证架构
        assert_eq!(elf_data[4], 2); // ELFCLASS64
        assert_eq!(elf_data[5], 1); // ELFDATA2LSB
    }
}
```

## 常见问题排除

### 问题 1: "Permission denied" 错误

```bash
# 确保文件有执行权限
chmod +x hello

# 检查文件权限
ls -la hello
```

### 问题 2: "No such file or directory" (32位程序在64位系统)

```bash
# 安装32位库支持
sudo apt install libc6-dev-i386

# 或者确保生成64位程序
# 在代码中设置: assembler.set_class(elf::ELFCLASS64)
```

### 问题 3: 段错误 (Segmentation fault)

```bash
# 使用 GDB 调试
gdb ./hello
(gdb) run
(gdb) bt

# 使用 strace 跟踪系统调用
strace ./hello

# 检查内存映射
cat /proc/$(pidof hello)/maps
```

### 问题 4: 共享库加载失败

```bash
# 检查库依赖
ldd client

# 设置库路径
export LD_LIBRARY_PATH=.:$LD_LIBRARY_PATH

# 或者使用 RPATH
# 在代码中: assembler.set_rpath("$ORIGIN")
```

## 调试技巧

### 使用 GDB 调试

```bash
# 编译时添加调试信息
# assembler.add_debug_info(true);

# 启动 GDB
gdb ./hello

# 常用 GDB 命令
(gdb) info registers    # 查看寄存器
(gdb) x/10i $pc        # 查看当前指令
(gdb) x/10x $rsp       # 查看栈内容
(gdb) disas main       # 反汇编函数
```

### 性能测试

```bash
# 使用 time 测量执行时间
time ./hello

# 使用 perf 进行性能分析
perf stat ./hello
perf record ./hello
perf report

# 内存使用分析
valgrind --tool=memcheck ./hello
```

## 下一步

现在您已经掌握了 ELF 后端的基础使用，可以继续学习：

1. **[基础概念](./concepts.md)** - 深入理解 ELF 格式和核心概念
2. **[文件结构](./file-structure.md)** - 学习 ELF 文件的详细结构
3. **[用户指南](../../user-guide/index.md)** - 了解 Gaia 框架的通用功能

## 参考资源

- [ELF 格式规范](https://refspecs.linuxfoundation.org/elf/elf.pdf)
- [Linux 系统调用表](https://filippo.io/linux-syscall-table/)
- [x86-64 指令集参考](https://www.intel.com/content/www/us/en/developer/articles/technical/intel-sdm.html)
- [GDB 用户手册](https://sourceware.org/gdb/current/onlinedocs/gdb/)

---

*如果您在使用过程中遇到问题，请查看 [维护指南](../../maintenance/troubleshooting.md) 或在 GitHub 上提交 Issue。*
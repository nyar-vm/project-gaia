# ELF 汇编器 - ELF 文件生成工具

一个用 Rust 编写的 ELF（可执行和链接格式）汇编器，可以生成 Linux 可执行文件。
该工具提供简单易用的 API 来创建基本的 ELF 可执行文件。

## 特性

- **ELF 文件生成**: 创建符合 ELF 标准的可执行文件
- **架构支持**: 支持 x86-64 架构
- **简单 API**: 提供易于使用的生成函数
- **Hello World 程序**: 快速生成 Hello World 可执行文件
- **退出代码程序**: 生成带有自定义退出代码的程序
- **控制台输出**: 生成可以输出文本到控制台的程序
- **测试覆盖**: 完整的测试套件确保生成的 ELF 文件正确性

### 基本示例

```rust
use elf_assembler::generator::{easy_hello_world, easy_exit_code, easy_console_log};
use gaia_types::helpers::Architecture;
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 生成 Hello World ELF 文件
    let hello_elf = easy_hello_world(Architecture::X86_64)?;
    fs::write("hello_world", &hello_elf)?;
    
    // 生成退出代码 ELF 文件
    let exit_elf = easy_exit_code(Architecture::X86_64, 42)?;
    fs::write("exit_program", &exit_elf)?;
    
    // 生成控制台输出 ELF 文件
    let console_elf = easy_console_log(Architecture::X86_64, "Hello from ELF!")?;
    fs::write("console_program", &console_elf)?;
    
    println!("ELF 文件生成完成！");
    Ok(())
}
```

### 使用 ELF 构建器

```rust
use elf_assembler::writer::ElfBuilder;

fn main() {
    let mut builder = ElfBuilder::new();
    
    // 添加代码段
    let code = vec![
        0x48, 0xc7, 0xc0, 0x3c, 0x00, 0x00, 0x00, // mov rax, 60 (sys_exit)
        0x48, 0xc7, 0xc7, 0x00, 0x00, 0x00, 0x00, // mov rdi, 0
        0x0f, 0x05, // syscall
    ];
    builder.add_code_section(code);
    
    // 构建 ELF 文件
    let elf_data = builder.build();
    
    // 保存到文件
    std::fs::write("custom_program", &elf_data).unwrap();
}
```

## API 参考

### 生成器函数

- `easy_hello_world(arch: Architecture) -> Result<Vec<u8>, GaiaError>`
  - 生成一个简单的 Hello World ELF 程序

- `easy_exit_code(arch: Architecture, exit_code: u8) -> Result<Vec<u8>, GaiaError>`
  - 生成一个以指定退出代码退出的 ELF 程序

- `easy_console_log(arch: Architecture, message: &str) -> Result<Vec<u8>, GaiaError>`
  - 生成一个输出指定消息到控制台的 ELF 程序

### ELF 构建器

`ElfBuilder` 类提供了更灵活的 ELF 文件构建方式：

- `new() -> ElfBuilder` - 创建新的构建器
- `add_code_section(code: Vec<u8>) -> &mut Self` - 添加代码段
- `add_data_section(data: Vec<u8>) -> &mut Self` - 添加数据段
- `build() -> Vec<u8>` - 构建最终的 ELF 文件

## 支持的架构

目前支持以下架构：
- **x86-64**: 64 位 x86 架构（Linux）

## 生成的 ELF 文件结构

生成的 ELF 文件包含：
- **ELF 头**: 标准的 64 位 ELF 头
- **程序头表**: 描述内存段的程序头
- **代码段**: 包含可执行机器码
- **数据段**: 包含程序数据（如果需要）

## 测试

运行测试套件：

```bash
cargo test
```

测试包括：
- ELF 文件格式验证
- 架构检测测试
- 生成器函数测试
- 文件保存和加载测试

## 示例程序

查看 `examples/` 目录中的示例程序，了解如何使用此库。

## 许可证

此项目采用 MIT 许可证 - 查看 [LICENSE](../../License.md) 文件了解详情。

## 贡献

欢迎贡献！请随时提交 Pull Request 或创建 Issue。

## 技术细节

### ELF 文件格式

生成的 ELF 文件遵循标准的 ELF 64 位格式：
- 魔数: `\x7fELF`
- 类别: 64 位 (ELFCLASS64)
- 数据编码: 小端序 (ELFDATA2LSB)
- 版本: 当前版本 (EV_CURRENT)
- 机器类型: x86-64 (EM_X86_64)

### 系统调用

生成的程序使用 Linux 系统调用：
- `sys_write` (1): 输出到控制台
- `sys_exit` (60): 程序退出

### 内存布局

- 代码段加载地址: 0x401000
- 数据段加载地址: 0x402000
- 程序入口点: 0x401000

## 故障排除

### 常见问题

1. **权限错误**: 确保生成的文件有执行权限
   ```bash
   chmod +x your_program
   ```

2. **架构不支持**: 目前只支持 x86-64 架构

3. **Linux 专用**: 生成的 ELF 文件只能在 Linux 系统上运行

### 调试

使用以下工具调试生成的 ELF 文件：
- `readelf -h file` - 查看 ELF 头信息
- `objdump -d file` - 反汇编代码
- `hexdump -C file` - 查看十六进制内容
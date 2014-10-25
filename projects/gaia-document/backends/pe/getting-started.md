# PE 后端入门指南

本指南将帮助您快速上手 Gaia 的 PE 后端，从环境配置到创建第一个 Windows 可执行文件。

## 环境准备

### 系统要求

- **操作系统**: Windows 10/11 或 Windows Server 2016+
- **架构**: x86-64 (推荐) 或 x86
- **内存**: 至少 4GB RAM
- **存储**: 至少 1GB 可用空间

### 开发工具

#### 1. Rust 工具链

```bash
# 安装 Rust (如果尚未安装)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 添加 Windows 目标平台
rustup target add x86_64-pc-windows-msvc
rustup target add i686-pc-windows-msvc

# 安装 MSVC 构建工具 (如果使用 Visual Studio)
# 或下载 "Build Tools for Visual Studio"
```

#### 2. 项目依赖

在您的 `Cargo.toml` 中添加：

```toml
[dependencies]
gaia-assembler = { path = "../gaia-assembler" }
gaia-types = { path = "../gaia-types" }

# PE 特定依赖
pe-parser = "0.1"
winapi = { version = "0.3", features = ["winuser", "consoleapi"] }

[dev-dependencies]
tempfile = "3.0"
```

#### 3. 可选开发工具

```bash
# PE 文件分析工具
choco install pe-explorer
choco install dependency-walker

# 调试工具
choco install windbg
choco install x64dbg
```

## 第一步：创建简单的控制台应用

### 项目结构

```
my-pe-project/
├── Cargo.toml
├── src/
│   ├── main.rs
│   └── lib.rs
├── examples/
│   └── hello_world.rs
└── tests/
    └── integration_tests.rs
```

### 基础示例：Hello World

创建 `examples/hello_world.rs`：

```rust
use gaia_assembler::pe::{PeAssembler, WriterConfig, SectionCharacteristics};
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. 创建 PE 汇编器实例
    let mut assembler = PeAssembler::new_console_app();

    // 2. 设置基本属性
    assembler.set_entry_point(0x1000);
    assembler.set_image_base(0x400000);
    assembler.set_subsystem(PeSubsystem::Console);

    // 3. 添加代码段
    let text_section = assembler.add_section(".text");
    text_section.set_characteristics(
        SectionCharacteristics::CODE |
            SectionCharacteristics::EXECUTE |
            SectionCharacteristics::READ
    );

    // 4. 生成 Hello World 的机器码
    let hello_code = generate_hello_world_code();
    text_section.add_code(&hello_code);

    // 5. 添加数据段
    let data_section = assembler.add_section(".data");
    data_section.set_characteristics(
        SectionCharacteristics::INITIALIZED_DATA |
            SectionCharacteristics::READ |
            SectionCharacteristics::WRITE
    );

    // 添加字符串数据
    let message = "Hello, World from Gaia PE!\n\0";
    data_section.add_string(message);

    // 6. 添加导入表
    assembler.add_import("kernel32.dll", &[
        "GetStdHandle",
        "WriteConsoleA",
        "ExitProcess"
    ]);

    // 7. 生成 PE 文件
    let config = WriterConfig {
        file_alignment: 0x200,
        section_alignment: 0x1000,
        ..Default::default()
    };

    let pe_data = assembler.build(config)?;

    // 8. 保存到文件
    fs::write("hello.exe", pe_data)?;

    println!("✅ 成功生成 hello.exe");
    println!("运行: ./hello.exe");

    Ok(())
}

fn generate_hello_world_code() -> Vec<u8> {
    // x86-64 汇编代码对应的机器码
    vec![
        // 函数序言
        0x48, 0x83, 0xEC, 0x28,                     // sub rsp, 40

        // 获取标准输出句柄
        0x48, 0xC7, 0xC1, 0xF5, 0xFF, 0xFF, 0xFF,   // mov rcx, -11 (STD_OUTPUT_HANDLE)
        0xFF, 0x15, 0x00, 0x00, 0x00, 0x00,         // call [GetStdHandle]
        0x48, 0x89, 0xC1,                           // mov rcx, rax (句柄)

        // 准备 WriteConsoleA 参数
        0x48, 0x8D, 0x15, 0x00, 0x00, 0x00, 0x00,   // lea rdx, [message]
        0x41, 0xB8, 0x1C, 0x00, 0x00, 0x00,         // mov r8d, 28 (字符串长度)
        0x4D, 0x31, 0xC9,                           // xor r9, r9 (NULL)
        0x48, 0x83, 0xEC, 0x20,                     // sub rsp, 32 (调用约定)
        0x48, 0xC7, 0x04, 0x24, 0x00, 0x00, 0x00, 0x00, // mov qword [rsp], 0
        0xFF, 0x15, 0x00, 0x00, 0x00, 0x00,         // call [WriteConsoleA]
        0x48, 0x83, 0xC4, 0x20,                     // add rsp, 32

        // 退出程序
        0x48, 0x31, 0xC9,                           // xor rcx, rcx (退出码 0)
        0xFF, 0x15, 0x00, 0x00, 0x00, 0x00,         // call [ExitProcess]

        // 函数尾声 (实际不会执行到)
        0x48, 0x83, 0xC4, 0x28,                     // add rsp, 40
        0xC3,                                       // ret
    ]
}
```

### 构建和运行

```bash
# 编译示例
cargo run --example hello_world

# 运行生成的 PE 文件
./hello.exe
```

## 第二步：创建动态链接库 (DLL)

创建 `examples/simple_dll.rs`：

```rust
use gaia_assembler::pe::{PeAssembler, WriterConfig, PeSubsystem};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. 创建 DLL 汇编器
    let mut assembler = PeAssembler::new_dll();

    // 2. 设置 DLL 属性
    assembler.set_dll_name("SimpleLib.dll");
    assembler.set_subsystem(PeSubsystem::WindowsGui);

    // 3. 添加导出函数
    assembler.add_export("Add", 0x1000);
    assembler.add_export("Multiply", 0x1020);

    // 4. 添加代码段
    let text_section = assembler.add_section(".text");

    // Add 函数: 两个整数相加
    let add_code = vec![
        0x48, 0x89, 0xC8,           // mov rax, rcx
        0x48, 0x01, 0xD0,           // add rax, rdx
        0xC3,                       // ret
    ];
    text_section.add_function("Add", &add_code);

    // Multiply 函数: 两个整数相乘
    let multiply_code = vec![
        0x48, 0x89, 0xC8,           // mov rax, rcx
        0x48, 0x0F, 0xAF, 0xC2,     // imul rax, rdx
        0xC3,                       // ret
    ];
    text_section.add_function("Multiply", &multiply_code);

    // 5. 生成 DLL
    let dll_data = assembler.build_dll(WriterConfig::default())?;

    // 6. 保存文件
    std::fs::write("SimpleLib.dll", dll_data)?;

    println!("✅ 成功生成 SimpleLib.dll");

    Ok(())
}
```

## 第三步：验证生成的文件

### 检查 PE 文件结构

```rust
use std::process::Command;

fn verify_pe_file(filename: &str) -> Result<(), Box<dyn std::error::Error>> {
    // 使用 dumpbin 工具检查 PE 结构
    let output = Command::new("dumpbin")
        .args(&["/headers", filename])
        .output()?;

    println!("PE 文件头信息:");
    println!("{}", String::from_utf8_lossy(&output.stdout));

    // 检查导入表
    let output = Command::new("dumpbin")
        .args(&["/imports", filename])
        .output()?;

    println!("导入表:");
    println!("{}", String::from_utf8_lossy(&output.stdout));

    Ok(())
}
```

### 运行时测试

创建 `tests/integration_tests.rs`：

```rust
#[cfg(test)]
mod tests {
    use std::process::Command;
    use tempfile::tempdir;

    #[test]
    fn test_hello_world_execution() {
        // 生成临时 PE 文件
        let dir = tempdir().unwrap();
        let exe_path = dir.path().join("test_hello.exe");

        // 这里调用您的 PE 生成代码
        // generate_hello_world_pe(&exe_path).unwrap();

        // 执行并检查输出
        let output = Command::new(&exe_path)
            .output()
            .expect("Failed to execute PE file");

        assert!(output.status.success());
        assert!(String::from_utf8_lossy(&output.stdout)
            .contains("Hello, World"));
    }

    #[test]
    fn test_dll_loading() {
        // 测试 DLL 加载和函数调用
        // 这需要额外的 Windows API 调用
    }
}
```

## 故障排除

### 常见问题

#### 1. "不是有效的 Win32 应用程序"

**原因**: PE 头格式错误或机器类型不匹配

**解决方案**:

```rust
// 确保设置正确的机器类型
assembler.set_machine_type(MachineType::Amd64); // 64位
// 或
assembler.set_machine_type(MachineType::I386);  // 32位
```

#### 2. "找不到入口点"

**原因**: 入口点地址设置错误

**解决方案**:

```rust
// 确保入口点地址与代码段中的实际地址匹配
assembler.set_entry_point(0x1000);
// 并且代码段的虚拟地址也是 0x1000
```

#### 3. "访问冲突"

**原因**: 内存权限设置错误

**解决方案**:

```rust
// 确保代码段有执行权限
text_section.set_characteristics(
SectionCharacteristics::CODE |
SectionCharacteristics::EXECUTE |
SectionCharacteristics::READ
);
```

### 调试技巧

1. **使用 PE 分析工具**:
   ```bash
   # 查看 PE 结构
   dumpbin /all your_file.exe
   
   # 检查依赖关系
   depends your_file.exe
   ```

2. **启用调试信息**:
   ```rust
   assembler.enable_debug_info(true);
   assembler.add_debug_section();
   ```

3. **逐步验证**:
    - 先生成最简单的 PE 文件
    - 逐步添加功能
    - 每次修改后都进行测试

## 性能测试

创建 `examples/performance_test.rs`：

```rust
use std::time::Instant;

fn benchmark_pe_generation() {
    let start = Instant::now();

    // 生成 PE 文件的代码
    for i in 0..100 {
        let mut assembler = PeAssembler::new_console_app();
        // ... 添加代码和数据
        let _pe_data = assembler.build(WriterConfig::default()).unwrap();
    }

    let duration = start.elapsed();
    println!("生成 100 个 PE 文件耗时: {:?}", duration);
    println!("平均每个文件: {:?}", duration / 100);
}
```

## 下一步

现在您已经成功创建了第一个 PE 文件，可以继续学习：

1. **[基础概念](./concepts.md)** - 深入了解 PE 格式的核心概念
2. **[文件结构](./file-structure.md)** - 学习 PE 文件的详细结构
3. **[代码生成](./code-generation.md)** - 掌握机器码生成技术
4. **[导入导出](./import-export.md)** - 学习 DLL 导入导出机制

## 示例代码仓库

完整的示例代码可以在以下位置找到：

- [GitHub 仓库](https://github.com/nyar-vm/project-gaia/tree/main/examples/pe)
- [在线演示](https://gaia-demo.example.com/pe)

---

*如果您在学习过程中遇到问题，欢迎在 [GitHub Issues](https://github.com/nyar-vm/project-gaia/issues) 中提问。*
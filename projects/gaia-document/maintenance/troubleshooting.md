# 故障排除

本文档提供 Gaia Assembler 常见问题的诊断和解决方案。

## 常见问题

### 编译错误

#### 1. 语法错误

**问题**: 编译时出现语法错误

```
Error: Unexpected token at line 15, column 8
  |
15| mov eax, [ebp+
  |          ^
  | Expected closing bracket
```

**解决方案**:

- 检查括号、引号是否匹配
- 验证指令语法是否正确
- 使用 IDE 的语法高亮功能

#### 2. 符号未定义

**问题**: 引用了未定义的符号

```
Error: Undefined symbol 'main'
  |
23| call main
  |      ^^^^
  | Symbol 'main' is not defined
```

**解决方案**:

```assembly
; 确保符号已定义
.global main
main:
    ; 函数实现
    ret
```

#### 3. 类型不匹配

**问题**: 操作数类型不匹配

```
Error: Type mismatch in instruction
  |
10| mov eax, "hello"
  |          ^^^^^^^
  | Cannot move string literal to register
```

**解决方案**:

```assembly
; 正确的做法
.data
hello_str: .ascii "hello"

.text
mov eax, offset hello_str
```

### 运行时错误

#### 1. 段错误 (Segmentation Fault)

**问题**: 程序运行时崩溃

**诊断步骤**:

```bash
# 使用调试器
gdb ./program
(gdb) run
(gdb) bt  # 查看调用栈
(gdb) info registers  # 查看寄存器状态
```

**常见原因**:

- 空指针解引用
- 数组越界访问
- 栈溢出

**解决方案**:

```assembly
; 检查指针有效性
test eax, eax
jz null_pointer_error
mov ebx, [eax]  ; 安全访问

null_pointer_error:
    ; 错误处理
    ret
```

#### 2. 栈溢出

**问题**: 栈空间不足

**诊断**:

```bash
# 检查栈使用情况
ulimit -s  # 查看栈大小限制
```

**解决方案**:

```assembly
; 减少局部变量使用
; 避免深度递归
; 使用堆分配大型数据结构
```

### 后端特定问题

#### CLR 后端

**问题**: .NET 运行时错误

```
System.InvalidProgramException: Common Language Runtime detected an invalid program.
```

**解决方案**:

1. 检查 IL 代码有效性
2. 验证元数据完整性
3. 使用 PEVerify 工具验证

```bash
# 验证生成的程序集
peverify program.exe
```

#### JVM 后端

**问题**: 字节码验证失败

```
java.lang.VerifyError: Bad type on operand stack
```

**解决方案**:

1. 检查操作数栈平衡
2. 验证类型转换
3. 确保异常处理正确

```bash
# 使用 javap 检查字节码
javap -c -verbose Program.class
```

## 诊断工具

### 1. 编译器诊断

```bash
# 启用详细输出
gaia-assembler --verbose input.gasm

# 生成调试信息
gaia-assembler --debug-info input.gasm

# 输出中间表示
gaia-assembler --emit-ir input.gasm
```

### 2. 性能分析

```bash
# 编译时性能分析
time gaia-assembler input.gasm

# 内存使用分析
valgrind --tool=massif gaia-assembler input.gasm

# CPU 性能分析
perf record gaia-assembler input.gasm
perf report
```

### 3. 调试工具

```rust
// 启用调试日志
use log::{debug, info, warn, error};

fn compile_with_debug(source: &str) {
    debug!("Starting compilation");
    info!("Processing {} lines", source.lines().count());
    
    // 编译逻辑
    
    warn!("Deprecated feature used");
    error!("Compilation failed");
}
```

## 环境问题

### 1. 依赖问题

**问题**: 缺少必要的依赖

```
error: failed to run custom build command for `gaia-assembler`
```

**解决方案**:

```bash
# 更新 Rust 工具链
rustup update

# 安装必要的系统依赖
# Ubuntu/Debian
sudo apt-get install build-essential

# CentOS/RHEL
sudo yum groupinstall "Development Tools"

# macOS
xcode-select --install
```

### 2. 版本兼容性

**问题**: Rust 版本不兼容

```
error: package `gaia-assembler v0.3.0` cannot be built because it requires rustc 1.70.0 or newer
```

**解决方案**:

```bash
# 检查当前版本
rustc --version

# 更新到最新版本
rustup update stable

# 安装特定版本
rustup install 1.70.0
rustup default 1.70.0
```

### 3. 平台特定问题

#### Windows

**问题**: MSVC 链接器错误

```
error: linking with `link.exe` failed
```

**解决方案**:

```bash
# 安装 Visual Studio Build Tools
# 或使用 GNU 工具链
rustup target add x86_64-pc-windows-gnu
cargo build --target x86_64-pc-windows-gnu
```

#### Linux

**问题**: 动态链接库缺失

```
error while loading shared libraries: libssl.so.1.1
```

**解决方案**:

```bash
# 安装缺失的库
sudo apt-get install libssl-dev

# 或使用静态链接
cargo build --features static-link
```

#### macOS

**问题**: 代码签名问题

```
error: failed to sign executable
```

**解决方案**:

```bash
# 禁用代码签名
export CODESIGN_ALLOCATE=/usr/bin/codesign_allocate
cargo build
```

## 性能问题

### 1. 编译速度慢

**诊断**:

```bash
# 分析编译时间
cargo build --timings

# 检查依赖编译时间
cargo build -v 2>&1 | grep "Compiling"
```

**优化方案**:

```toml
# Cargo.toml
[profile.dev]
opt-level = 1  # 轻微优化
debug = 1      # 减少调试信息

[build]
jobs = 4       # 并行编译
```

### 2. 内存使用过高

**诊断**:

```bash
# 监控内存使用
/usr/bin/time -v gaia-assembler input.gasm

# 使用 Valgrind
valgrind --tool=massif gaia-assembler input.gasm
```

**优化方案**:

```rust
// 使用流式处理
fn process_large_file(path: &Path) -> Result<(), Error> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    
    for line in reader.lines() {
        process_line(&line?)?;
    }
    
    Ok(())
}
```

### 3. 生成代码性能差

**诊断**:

```bash
# 性能分析
perf record ./program
perf report

# 反汇编分析
objdump -d program
```

**优化方案**:

- 启用编译器优化
- 使用更高效的算法
- 减少内存分配

## 错误报告

### 1. 收集信息

创建错误报告时，请包含以下信息：

```bash
# 系统信息
uname -a
rustc --version
cargo --version

# 编译器版本
gaia-assembler --version

# 重现步骤
echo "详细的重现步骤"

# 错误输出
gaia-assembler input.gasm 2>&1 | tee error.log
```

### 2. 最小化测试用例

```assembly
; 创建最小的重现用例
.text
.global _start

_start:
    ; 导致问题的最少代码
    mov eax, 1
    int 0x80
```

### 3. 环境隔离

```bash
# 在干净环境中测试
docker run --rm -it rust:latest bash
cargo install gaia-assembler
# 重现问题
```

## 调试技巧

### 1. 逐步调试

```rust
// 添加调试输出
#[cfg(debug_assertions)]
macro_rules! debug_print {
    ($($arg:tt)*) => {
        eprintln!("[DEBUG] {}: {}", file!(), format!($($arg)*));
    };
}

fn compile_instruction(inst: &Instruction) {
    debug_print!("Compiling instruction: {:?}", inst);
    // 编译逻辑
}
```

### 2. 单元测试

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_problematic_case() {
        let input = "mov eax, ebx";
        let result = compile(input);
        assert!(result.is_ok());
    }
}
```

### 3. 集成测试

```bash
#!/bin/bash
# tests/integration_test.sh

echo "Running integration tests..."

for test_file in tests/*.gasm; do
    echo "Testing $test_file"
    if ! gaia-assembler "$test_file" > /dev/null 2>&1; then
        echo "FAILED: $test_file"
        exit 1
    fi
done

echo "All tests passed!"
```

## 获取帮助

### 1. 文档资源

- [用户指南](../user-guide/index.md)
- [API 参考](../api-reference/index.md)
- [内部实现](../internals/index.md)

### 2. 社区支持

- GitHub Issues: 报告 bug 和功能请求
- 讨论区: 技术讨论和问答
- 邮件列表: 开发者交流

### 3. 专业支持

如需专业技术支持，请联系：

- 邮箱: support@gaia-assembler.org
- 企业支持: enterprise@gaia-assembler.org

## 预防措施

### 1. 最佳实践

- 定期更新依赖
- 使用版本控制
- 编写测试用例
- 代码审查

### 2. 监控和告警

```rust
// 添加健康检查
fn health_check() -> Result<(), Error> {
    // 检查系统资源
    check_memory_usage()?;
    check_disk_space()?;
    check_dependencies()?;
    
    Ok(())
}
```

### 3. 备份和恢复

```bash
# 定期备份重要文件
tar -czf backup-$(date +%Y%m%d).tar.gz src/ tests/ Cargo.toml

# 版本控制
git add .
git commit -m "Working version before changes"
git tag -a v0.3.0-stable -m "Stable version"
```
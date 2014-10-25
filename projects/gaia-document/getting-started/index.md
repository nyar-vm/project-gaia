# 快速开始

欢迎使用 Gaia Assembler！这是一个现代化的多平台汇编器框架，支持多种目标平台的代码生成。

## 安装

### 系统要求

- Rust 1.70 或更高版本
- 支持的操作系统：Windows、Linux、macOS

### 从源码构建

```bash
# 克隆项目
git clone https://github.com/nyar-vm/project-gaia.git
cd project-gaia/projects

# 构建所有项目
cargo build --release

# 或者构建特定的汇编器
cd gaia-assembler
cargo build --release
```

### 预编译二进制文件

```bash
# 下载最新版本
curl -L https://github.com/nyar-vm/project-gaia/releases/latest/download/gaia-assembler-{platform}.tar.gz | tar xz

# 添加到 PATH
export PATH=$PATH:./gaia-assembler/bin
```

## 第一个程序

让我们从一个简单的 "Hello, World!" 程序开始：

### CLR/.NET 示例

创建文件 `hello.msil`：

```msil
.assembly hello {}
.class public HelloWorld {
    .method public static void Main() {
        ldstr "Hello, World!"
        call void [mscorlib]System.Console::WriteLine(string)
        ret
    }
}
```

编译并运行：

```bash
gaia-assembler clr hello.msil -o hello.exe
./hello.exe
```

### JVM/Java 示例

创建文件 `hello.jasm`：

```jasm
.class public HelloWorld
.super java/lang/Object

.method public static main([Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "Hello, World!"
    invokevirtual java/io/PrintStream/println(Ljava/lang/String;)V
    return
.end method
```

编译并运行：

```bash
gaia-assembler jvm hello.jasm -o HelloWorld.class
java HelloWorld
```

## 核心概念

### 汇编器架构

Gaia Assembler 采用模块化设计：

- **前端**: 解析汇编源码，生成中间表示
- **中间层**: 优化和转换
- **后端**: 生成目标平台的机器码或字节码

### 支持的后端

- **CLR**: .NET IL 代码生成
- **PE**: Windows 可执行文件
- **ELF**: Linux 可执行文件
- **JVM**: Java 字节码
- **WASM**: WebAssembly 模块

### 指令集

每个后端都有自己的指令集：

- **MSIL**: Microsoft Intermediate Language
- **JASM**: Java Assembly Language
- **WASM**: WebAssembly Text Format

## 下一步

- [用户指南](/user-guide/) - 深入了解汇编语法和指令集
- [后端文档](/backends/) - 选择合适的目标平台
- [开发者指南](/developer-guide/) - 扩展和定制 Gaia

## 获取帮助

- [GitHub Issues](https://github.com/nyar-vm/project-gaia/issues)
- [讨论区](https://github.com/nyar-vm/project-gaia/discussions)
- [故障排除](/maintenance/troubleshooting) - 常见问题解决方案
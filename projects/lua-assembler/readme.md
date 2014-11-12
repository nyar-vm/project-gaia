# lua-assembler

用于读取/写入 Python `.pyc` 字节码文件的 Rust 实现。

## 功能

- 读取 `.pyc` 文件：解析 16 字节头（PEP 552）并保留主体
- 写入 `.pyc` 文件：按原样写回头与主体，实现无损回写
- 命令行工具：支持 `pyc-assembler <in.pyc> <out.pyc>` 进行回写验证

## 快速开始

### 构建

```bash
cargo build -p pyc-assembler
```

### 生成示例 .pyc（需要已安装 Python）

```bash
python - <<'PY'
import py_compile, pathlib
p = pathlib.Path('tests/pyc_src')
p.mkdir(parents=True, exist_ok=True)
(p/'hello.py').write_text('print("hello from pyc")\n')
py_compile.compile(str(p/'hello.py'), cfile=str(p/'hello.pyc'))
print('OK')
PY
```

### 回写并执行验证

```bash
cargo run -p pyc-assembler -- tests/pyc_src/hello.pyc tests/pyc_src/out.pyc
python tests/pyc_src/out.pyc
# 预期输出：hello from pyc
```

## API 示例

```rust
use std::path::Path;
use pyc_assembler::formats::pyc::{read_pyc_file, write_pyc_file};

let pyc = read_pyc_file(Path::new("tests/pyc_src/hello.pyc")).unwrap();
write_pyc_file(Path::new("tests/pyc_src/out.pyc"), &pyc).unwrap();
```

## 说明

`.pyc` 主体为 Python 的 `marshal` 序列化的 `code object`，本库当前不解析主体，仅做无损读写；这足以用于执行验证与后续扩展。
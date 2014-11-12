# Builder 模块

`builder` 模块负责构建 `PythonProgram` 对象，它将各种组件（如指令、常量等）组装成一个完整的、可表示 Python 字节码的程序结构。

## 结构

- `PythonBuilder`: 用于逐步构建 `PythonProgram` 的结构体。

## 示例

```rust
use pyc_assembler::{
    builder::PythonBuilder,
    program::{PycHeader, PythonObject, PythonProgram},
};

let builder = PythonBuilder::new();
let header = PycHeader::default();
let program = builder.build(header);

assert_eq!(program.code_object.source_name, "<string>".to_string());
```
#![doc = include_str!("readme.md")]

use crate::program::{PycHeader, PythonObject};

mod to_program;
mod to_pyc;

#[derive(Clone, Debug)]
/// PycView 结构体表示一个 .pyc 文件的抽象视图，包含其头部信息、代码对象字节码以及解析后的各种组件。
pub struct PycView {
    pub(crate) header: PycHeader,
    pub(crate) code_object_bytes: Vec<u8>,
    pub(crate) constants: Vec<PythonObject>,
    #[allow(dead_code)]
    pub(crate) names: Vec<String>,
    pub(crate) varnames: Vec<String>,
    pub(crate) freevars: Vec<String>,
    #[allow(dead_code)]
    pub(crate) cellvars: Vec<String>,
    pub(crate) filename: String,
    #[allow(dead_code)]
    pub(crate) name: String,
    pub(crate) argcount: u32,
    #[allow(dead_code)]
    pub(crate) posonlyargcount: u32,
    #[allow(dead_code)]
    pub(crate) kwonlyargcount: u32,
    pub(crate) nlocals: u32,
    pub(crate) stacksize: u32,
    pub(crate) flags: u32,
    pub(crate) firstlineno: u32,
    pub(crate) lnotab: Vec<u8>,
}

impl Default for PycView {
    fn default() -> Self {
        Self {
            header: PycHeader::default(),
            code_object_bytes: Vec::new(),
            constants: Vec::new(),
            names: Vec::new(),
            varnames: Vec::new(),
            freevars: Vec::new(),
            cellvars: Vec::new(),
            filename: String::new(),
            name: String::new(),
            argcount: 0,
            posonlyargcount: 0,
            kwonlyargcount: 0,
            nlocals: 0,
            stacksize: 0,
            flags: 0,
            firstlineno: 0,
            lnotab: Vec::new(),
        }
    }
}

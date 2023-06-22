use std::{io, mem, slice};
use std::io::Write;
use dynasm::dynasm;
use dynasmrt::{DynasmApi, DynasmLabelApi};


fn main() {
    // 创建动态汇编代码块
    let mut ops = dynasmrt::x64::Assembler::new().unwrap();

    // 生成动态汇编代码
    dynasm!(ops
        ; push rdi
        ; push rsi
        ; mov rdi, rbx
        ; mov rsi, rcx
        ; mov rax, QWORD run_wasi as _
        ; call rax
        ; pop rsi
        ; pop rdi
    );

    // 将动态汇编代码块转换为函数指针
    let code = ops.finalize().unwrap();
    let func: extern "sysv64" fn(*const u8, u64) = unsafe { std::mem::transmute(code.ptr(code.start)) };

    // 调用函数
    let buffer = b"Hello World";
    let length = buffer.len() as u64;
    func(buffer.as_ptr(), length);
}
use std::{io, mem, slice};
use std::io::Write;
use dynasm::dynasm;
use dynasmrt::{DynasmApi, DynasmLabelApi};

fn main() {
    let mut ops = dynasmrt::x64::Assembler::new().unwrap();
    let string = "Hello World!";
    ops.global_label("wasi");
    ops.extend(string.as_bytes()
    );

    let hello = ops.offset();
    dynasm!(ops
        ; .arch x64
        ; lea rcx, [->wasi]
        ; xor edx, edx
        ; mov dl, BYTE string.len() as _
        ; mov rax, QWORD run_wasi as _
        ; sub rsp, BYTE 0x28
        ; call rax
        ; add rsp, BYTE 0x28
        ; ret
    );

    let buf = ops.finalize().unwrap();

    let hello_fn: extern "win64" fn() -> bool = unsafe { mem::transmute(buf.ptr(hello)) };
    // 140695210460816
    println!("PTR: {}", run_wasi as u64);

    assert!(hello_fn());
}

pub extern "win64" fn run_wasi(buffer: *const u8, length: u64) -> bool {
    io::stdout()
        .write_all(unsafe { slice::from_raw_parts(buffer, length as usize) })
        .is_ok()
}
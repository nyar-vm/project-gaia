//! X64 代码构建器
//!
//! 提供生成 x64 汇编指令的高级接口

use super::context::{RelocationType, X64Context};

/// X64 代码构建器
///
/// 现在使用 context 来管理代码生成状态
#[derive(Debug)]
pub struct X64CodeBuilder {
    context: X64Context,
}

impl X64CodeBuilder {
    /// 创建新的代码构建器
    pub fn new() -> Self {
        Self { context: X64Context::new() }
    }

    /// 获取上下文的可变引用
    pub fn context_mut(&mut self) -> &mut X64Context {
        &mut self.context
    }

    /// 获取上下文的不可变引用
    pub fn context(&self) -> &X64Context {
        &self.context
    }

    /// 获取生成的代码
    pub fn code(&self) -> &[u8] {
        &self.context.code
    }

    /// 完成代码生成并返回上下文
    pub fn finish(self) -> X64Context {
        self.context
    }

    /// 生成函数序言
    pub fn function_prologue(&mut self) -> &mut Self {
        let bytes = vec![
            0x55, // push rbp
            0x48, 0x89, 0xE5, // mov rbp, rsp
            0x48, 0x83, 0xEC, 0x20, // sub rsp, 32 (shadow space)
        ];
        self.context.emit_bytes(&bytes);
        self
    }

    /// 生成函数尾声
    pub fn function_epilogue(&mut self) -> &mut Self {
        let bytes = vec![
            0x48, 0x89, 0xEC, // mov rsp, rbp
            0x5D, // pop rbp
            0xC3, // ret
        ];
        self.context.emit_bytes(&bytes);
        self
    }

    /// 生成退出程序代码
    pub fn exit_program(&mut self, exit_code: i32) -> &mut Self {
        // mov ecx, exit_code (第一个参数)
        self.context.emit_bytes(&[0xB9]);
        self.context.emit_bytes(&exit_code.to_le_bytes());

        // call ExitProcess - 添加函数调用信息
        self.context.add_function_call("ExitProcess", true);
        self.context.add_relocation(RelocationType::RipRel32, "ExitProcess");
        self.context.emit_bytes(&[0xFF, 0x15, 0x00, 0x00, 0x00, 0x00]); // call [ExitProcess]

        self
    }

    /// 实现 context 管理功能
    pub fn load_immediate(&mut self, value: i64) {
        // mov rax, value
        if value >= i32::MIN as i64 && value <= i32::MAX as i64 {
            self.context.emit_bytes(&[0xB8]);
            self.context.emit_bytes(&(value as i32).to_le_bytes());
        }
        else {
            self.context.emit_bytes(&[0x48, 0xB8]);
            self.context.emit_bytes(&value.to_le_bytes());
        }
    }

    pub fn load_string_address(&mut self, string_id: &str) {
        // 添加字符串常量
        self.context.add_string_constant(string_id);

        // lea rax, [rip + string_offset]
        self.context.emit_bytes(&[0x48, 0x8D, 0x05]);
        let _offset = self.context.reference_label(&format!("str_{}", string_id));
        self.context.emit_bytes(&[0x00, 0x00, 0x00, 0x00]); // 占位符
    }

    pub fn store_local(&mut self, offset: i32) {
        let stack_offset = self.context.allocate_stack(8);
        // mov [rbp + offset], rax
        if stack_offset >= -128 && stack_offset <= 127 {
            self.context.emit_bytes(&[0x48, 0x89, 0x45]);
            self.context.emit_bytes(&[stack_offset as u8]);
        }
        else {
            self.context.emit_bytes(&[0x48, 0x89, 0x85]);
            self.context.emit_bytes(&stack_offset.to_le_bytes());
        }
    }

    pub fn load_local(&mut self, offset: i32) {
        // mov rax, [rbp + offset]
        if offset >= -128 && offset <= 127 {
            self.context.emit_bytes(&[0x48, 0x8B, 0x45]);
            self.context.emit_bytes(&[offset as u8]);
        }
        else {
            self.context.emit_bytes(&[0x48, 0x8B, 0x85]);
            self.context.emit_bytes(&offset.to_le_bytes());
        }
    }

    pub fn add_operation(&mut self) {
        // pop rbx; pop rax; add rax, rbx; push rax
        self.context.emit_bytes(&[
            0x5B, // pop rbx
            0x58, // pop rax
            0x48, 0x01, 0xD8, // add rax, rbx
            0x50, // push rax
        ]);
    }

    pub fn sub_operation(&mut self) {
        // pop rbx; pop rax; sub rax, rbx; push rax
        self.context.emit_bytes(&[
            0x5B, // pop rbx
            0x58, // pop rax
            0x48, 0x29, 0xD8, // sub rax, rbx
            0x50, // push rax
        ]);
    }

    pub fn mul_operation(&mut self) {
        // pop rbx; pop rax; imul rax, rbx; push rax
        self.context.emit_bytes(&[
            0x5B, // pop rbx
            0x58, // pop rax
            0x48, 0x0F, 0xAF, 0xC3, // imul rax, rbx
            0x50, // push rax
        ]);
    }

    pub fn call_printf(&mut self) {
        // 调用 printf 函数
        self.context.add_function_call("printf", true);

        // call [printf] - RIP相对寻址
        self.context.emit_bytes(&[0xFF, 0x15, 0x00, 0x00, 0x00, 0x00]);
    }

    pub fn pop_stack(&mut self) {
        // pop rax
        self.context.emit_bytes(&[0x58]);
    }

    pub fn conditional_jump_false(&mut self, label: &str) {
        // test rax, rax; jz label
        self.context.emit_bytes(&[0x48, 0x85, 0xC0]); // test rax, rax
        self.context.emit_bytes(&[0x0F, 0x84]); // jz
        let _offset = self.context.reference_label(label);
        self.context.emit_bytes(&[0x00, 0x00, 0x00, 0x00]); // 占位符
    }

    pub fn unconditional_jump(&mut self, label: &str) {
        // jmp label
        self.context.emit_bytes(&[0xE9]); // jmp
        let _offset = self.context.reference_label(label);
        self.context.emit_bytes(&[0x00, 0x00, 0x00, 0x00]); // 占位符
    }

    /// 生成 Hello World 程序
    pub fn hello_world_program() -> Vec<u8> {
        let mut builder = X64CodeBuilder::new();

        // 函数序言
        builder.function_prologue();

        // 加载字符串地址到 RCX (第一个参数)
        builder.load_string_address("Hello, World!\n");

        // 调用 printf
        builder.call_printf();

        // 退出程序
        builder.exit_program(0);

        builder.finish().code
    }

    /// 生成消息框程序
    pub fn message_box_program() -> Vec<u8> {
        let mut builder = X64CodeBuilder::new();

        // 函数序言
        builder.function_prologue();

        // MessageBoxA 参数设置
        // mov r9d, 0 (uType = MB_OK)
        builder.context_mut().emit_bytes(&[0x41, 0xB9, 0x00, 0x00, 0x00, 0x00]);

        // mov r8, title_addr (lpCaption)
        builder.context_mut().emit_bytes(&[0x49, 0xB8]);
        builder.context_mut().emit_bytes(&0x2100u64.to_le_bytes()); // 假设标题在 0x2100

        // mov rdx, message_addr (lpText)
        builder.context_mut().emit_bytes(&[0x48, 0xBA]);
        builder.context_mut().emit_bytes(&0x2000u64.to_le_bytes()); // 假设消息在 0x2000

        // mov rcx, 0 (hWnd = NULL)
        builder.context_mut().emit_bytes(&[0x48, 0xB9, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);

        // call MessageBoxA
        builder.context_mut().emit_bytes(&[0xFF, 0x15, 0x00, 0x00, 0x00, 0x00]);

        // 退出程序
        builder.exit_program(0);

        builder.finish().code
    }
}

use crate::program::{JvmAccessFlags, JvmExceptionHandler, JvmInstruction, JvmMethod, JvmProgram, JvmVersion};

/// JVM 程序构建器
pub struct JvmProgramBuilder {
    program: JvmProgram,
}

impl JvmProgramBuilder {
    pub fn new(name: impl Into<String>) -> Self {
        Self { program: JvmProgram::new(name.into()) }
    }

    pub fn with_version(mut self, major: u16, minor: u16) -> Self {
        self.program.version = JvmVersion { major, minor };
        self
    }

    pub fn with_access_flags(mut self, flags: JvmAccessFlags) -> Self {
        self.program.access_flags = flags;
        self
    }

    pub fn with_public(mut self) -> Self {
        self.program.access_flags.is_public = true;
        self
    }

    pub fn with_super_class(mut self, super_class: impl Into<String>) -> Self {
        self.program.super_class = Some(super_class.into());
        self
    }

    pub fn add_interface(mut self, interface: impl Into<String>) -> Self {
        self.program.interfaces.push(interface.into());
        self
    }

    pub fn with_source_file(mut self, source_file: impl Into<String>) -> Self {
        self.program.source_file = Some(source_file.into());
        self
    }

    pub fn add_method<F>(mut self, name: impl Into<String>, descriptor: impl Into<String>, f: F) -> Self
    where
        F: FnOnce(JvmMethodBuilder) -> JvmMethodBuilder,
    {
        let builder = JvmMethodBuilder::new(name.into(), descriptor.into());
        let method = f(builder).build();
        self.program.add_method(method);
        self
    }

    pub fn build(self) -> JvmProgram {
        self.program
    }
}

/// JVM 方法构建器
pub struct JvmMethodBuilder {
    method: JvmMethod,
}

impl JvmMethodBuilder {
    pub fn new(name: String, descriptor: String) -> Self {
        Self { method: JvmMethod::new(name, descriptor) }
    }

    pub fn with_public(mut self) -> Self {
        self.method.access_flags.is_public = true;
        self
    }

    pub fn with_private(mut self) -> Self {
        self.method.access_flags.is_private = true;
        self
    }

    pub fn with_protected(mut self) -> Self {
        self.method.access_flags.is_protected = true;
        self
    }

    pub fn with_static(mut self) -> Self {
        self.method.access_flags.is_static = true;
        self
    }

    pub fn with_final(mut self) -> Self {
        self.method.access_flags.is_final = true;
        self
    }

    pub fn with_synchronized(mut self) -> Self {
        self.method.access_flags.is_synchronized = true;
        self
    }

    pub fn with_max_stack(mut self, max_stack: u16) -> Self {
        self.method.max_stack = max_stack;
        self
    }

    pub fn with_max_locals(mut self, max_locals: u16) -> Self {
        self.method.max_locals = max_locals;
        self
    }

    pub fn add_instruction(mut self, instruction: JvmInstruction) -> Self {
        self.method.add_instruction(instruction);
        self
    }

    pub fn add_exception_handler(mut self, handler: JvmExceptionHandler) -> Self {
        self.method.add_exception_handler(handler);
        self
    }

    pub fn add_exception(mut self, exception: impl Into<String>) -> Self {
        self.method.add_exception(exception.into());
        self
    }

    // 常用指令的便捷方法
    pub fn nop(self) -> Self {
        self.add_instruction(JvmInstruction::Nop)
    }
    pub fn aconst_null(self) -> Self {
        self.add_instruction(JvmInstruction::AconstNull)
    }
    pub fn iconst_m1(self) -> Self {
        self.add_instruction(JvmInstruction::IconstM1)
    }
    pub fn iconst_0(self) -> Self {
        self.add_instruction(JvmInstruction::Iconst0)
    }
    pub fn iconst_1(self) -> Self {
        self.add_instruction(JvmInstruction::Iconst1)
    }
    pub fn iconst_2(self) -> Self {
        self.add_instruction(JvmInstruction::Iconst2)
    }
    pub fn iconst_3(self) -> Self {
        self.add_instruction(JvmInstruction::Iconst3)
    }
    pub fn iconst_4(self) -> Self {
        self.add_instruction(JvmInstruction::Iconst4)
    }
    pub fn iconst_5(self) -> Self {
        self.add_instruction(JvmInstruction::Iconst5)
    }
    pub fn lconst_0(self) -> Self {
        self.add_instruction(JvmInstruction::Lconst0)
    }
    pub fn lconst_1(self) -> Self {
        self.add_instruction(JvmInstruction::Lconst1)
    }
    pub fn fconst_0(self) -> Self {
        self.add_instruction(JvmInstruction::Fconst0)
    }
    pub fn fconst_1(self) -> Self {
        self.add_instruction(JvmInstruction::Fconst1)
    }
    pub fn fconst_2(self) -> Self {
        self.add_instruction(JvmInstruction::Fconst2)
    }
    pub fn dconst_0(self) -> Self {
        self.add_instruction(JvmInstruction::Dconst0)
    }
    pub fn dconst_1(self) -> Self {
        self.add_instruction(JvmInstruction::Dconst1)
    }

    pub fn bipush(self, value: i8) -> Self {
        self.add_instruction(JvmInstruction::Bipush { value })
    }
    pub fn sipush(self, value: i16) -> Self {
        self.add_instruction(JvmInstruction::Sipush { value })
    }
    pub fn ldc(self, symbol: impl Into<String>) -> Self {
        self.add_instruction(JvmInstruction::Ldc { symbol: symbol.into() })
    }

    pub fn iload(self, index: u16) -> Self {
        self.add_instruction(JvmInstruction::Iload { index })
    }
    pub fn lload(self, index: u16) -> Self {
        self.add_instruction(JvmInstruction::Lload { index })
    }
    pub fn fload(self, index: u16) -> Self {
        self.add_instruction(JvmInstruction::Fload { index })
    }
    pub fn dload(self, index: u16) -> Self {
        self.add_instruction(JvmInstruction::Dload { index })
    }
    pub fn aload(self, index: u16) -> Self {
        self.add_instruction(JvmInstruction::Aload { index })
    }
    pub fn iload_0(self) -> Self {
        self.add_instruction(JvmInstruction::Iload0)
    }
    pub fn iload_1(self) -> Self {
        self.add_instruction(JvmInstruction::Iload1)
    }
    pub fn iload_2(self) -> Self {
        self.add_instruction(JvmInstruction::Iload2)
    }
    pub fn iload_3(self) -> Self {
        self.add_instruction(JvmInstruction::Iload3)
    }
    pub fn lload_0(self) -> Self {
        self.add_instruction(JvmInstruction::Lload0)
    }
    pub fn lload_1(self) -> Self {
        self.add_instruction(JvmInstruction::Lload1)
    }
    pub fn lload_2(self) -> Self {
        self.add_instruction(JvmInstruction::Lload2)
    }
    pub fn lload_3(self) -> Self {
        self.add_instruction(JvmInstruction::Lload3)
    }
    pub fn fload_0(self) -> Self {
        self.add_instruction(JvmInstruction::Fload0)
    }
    pub fn fload_1(self) -> Self {
        self.add_instruction(JvmInstruction::Fload1)
    }
    pub fn fload_2(self) -> Self {
        self.add_instruction(JvmInstruction::Fload2)
    }
    pub fn fload_3(self) -> Self {
        self.add_instruction(JvmInstruction::Fload3)
    }
    pub fn dload_0(self) -> Self {
        self.add_instruction(JvmInstruction::Dload0)
    }
    pub fn dload_1(self) -> Self {
        self.add_instruction(JvmInstruction::Dload1)
    }
    pub fn dload_2(self) -> Self {
        self.add_instruction(JvmInstruction::Dload2)
    }
    pub fn dload_3(self) -> Self {
        self.add_instruction(JvmInstruction::Dload3)
    }
    pub fn aload_0(self) -> Self {
        self.add_instruction(JvmInstruction::Aload0)
    }
    pub fn aload_1(self) -> Self {
        self.add_instruction(JvmInstruction::Aload1)
    }
    pub fn aload_2(self) -> Self {
        self.add_instruction(JvmInstruction::Aload2)
    }
    pub fn aload_3(self) -> Self {
        self.add_instruction(JvmInstruction::Aload3)
    }

    pub fn istore(self, index: u16) -> Self {
        self.add_instruction(JvmInstruction::Istore { index })
    }
    pub fn lstore(self, index: u16) -> Self {
        self.add_instruction(JvmInstruction::Lstore { index })
    }
    pub fn fstore(self, index: u16) -> Self {
        self.add_instruction(JvmInstruction::Fstore { index })
    }
    pub fn dstore(self, index: u16) -> Self {
        self.add_instruction(JvmInstruction::Dstore { index })
    }
    pub fn astore(self, index: u16) -> Self {
        self.add_instruction(JvmInstruction::Astore { index })
    }
    pub fn istore_0(self) -> Self {
        self.add_instruction(JvmInstruction::Istore0)
    }
    pub fn istore_1(self) -> Self {
        self.add_instruction(JvmInstruction::Istore1)
    }
    pub fn istore_2(self) -> Self {
        self.add_instruction(JvmInstruction::Istore2)
    }
    pub fn istore_3(self) -> Self {
        self.add_instruction(JvmInstruction::Istore3)
    }
    pub fn lstore_0(self) -> Self {
        self.add_instruction(JvmInstruction::Lstore0)
    }
    pub fn lstore_1(self) -> Self {
        self.add_instruction(JvmInstruction::Lstore1)
    }
    pub fn lstore_2(self) -> Self {
        self.add_instruction(JvmInstruction::Lstore2)
    }
    pub fn lstore_3(self) -> Self {
        self.add_instruction(JvmInstruction::Lstore3)
    }
    pub fn fstore_0(self) -> Self {
        self.add_instruction(JvmInstruction::Fstore0)
    }
    pub fn fstore_1(self) -> Self {
        self.add_instruction(JvmInstruction::Fstore1)
    }
    pub fn fstore_2(self) -> Self {
        self.add_instruction(JvmInstruction::Fstore2)
    }
    pub fn fstore_3(self) -> Self {
        self.add_instruction(JvmInstruction::Fstore3)
    }
    pub fn dstore_0(self) -> Self {
        self.add_instruction(JvmInstruction::Dstore0)
    }
    pub fn dstore_1(self) -> Self {
        self.add_instruction(JvmInstruction::Dstore1)
    }
    pub fn dstore_2(self) -> Self {
        self.add_instruction(JvmInstruction::Dstore2)
    }
    pub fn dstore_3(self) -> Self {
        self.add_instruction(JvmInstruction::Dstore3)
    }
    pub fn astore_0(self) -> Self {
        self.add_instruction(JvmInstruction::Astore0)
    }
    pub fn astore_1(self) -> Self {
        self.add_instruction(JvmInstruction::Astore1)
    }
    pub fn astore_2(self) -> Self {
        self.add_instruction(JvmInstruction::Astore2)
    }
    pub fn astore_3(self) -> Self {
        self.add_instruction(JvmInstruction::Astore3)
    }

    pub fn iadd(self) -> Self {
        self.add_instruction(JvmInstruction::Iadd)
    }
    pub fn ladd(self) -> Self {
        self.add_instruction(JvmInstruction::Ladd)
    }
    pub fn fadd(self) -> Self {
        self.add_instruction(JvmInstruction::Fadd)
    }
    pub fn dadd(self) -> Self {
        self.add_instruction(JvmInstruction::Dadd)
    }
    pub fn isub(self) -> Self {
        self.add_instruction(JvmInstruction::Isub)
    }
    pub fn lsub(self) -> Self {
        self.add_instruction(JvmInstruction::Lsub)
    }
    pub fn fsub(self) -> Self {
        self.add_instruction(JvmInstruction::Fsub)
    }
    pub fn dsub(self) -> Self {
        self.add_instruction(JvmInstruction::Dsub)
    }
    pub fn imul(self) -> Self {
        self.add_instruction(JvmInstruction::Imul)
    }
    pub fn lmul(self) -> Self {
        self.add_instruction(JvmInstruction::Lmul)
    }
    pub fn fmul(self) -> Self {
        self.add_instruction(JvmInstruction::Fmul)
    }
    pub fn dmul(self) -> Self {
        self.add_instruction(JvmInstruction::Dmul)
    }
    pub fn idiv(self) -> Self {
        self.add_instruction(JvmInstruction::Idiv)
    }
    pub fn ldiv(self) -> Self {
        self.add_instruction(JvmInstruction::Ldiv)
    }
    pub fn fdiv(self) -> Self {
        self.add_instruction(JvmInstruction::Fdiv)
    }
    pub fn ddiv(self) -> Self {
        self.add_instruction(JvmInstruction::Ddiv)
    }
    pub fn irem(self) -> Self {
        self.add_instruction(JvmInstruction::Irem)
    }
    pub fn lrem(self) -> Self {
        self.add_instruction(JvmInstruction::Lrem)
    }
    pub fn frem(self) -> Self {
        self.add_instruction(JvmInstruction::Frem)
    }
    pub fn drem(self) -> Self {
        self.add_instruction(JvmInstruction::Drem)
    }
    pub fn ineg(self) -> Self {
        self.add_instruction(JvmInstruction::Ineg)
    }
    pub fn lneg(self) -> Self {
        self.add_instruction(JvmInstruction::Lneg)
    }
    pub fn fneg(self) -> Self {
        self.add_instruction(JvmInstruction::Fneg)
    }
    pub fn dneg(self) -> Self {
        self.add_instruction(JvmInstruction::Dneg)
    }

    pub fn ishl(self) -> Self {
        self.add_instruction(JvmInstruction::Ishl)
    }
    pub fn lshl(self) -> Self {
        self.add_instruction(JvmInstruction::Lshl)
    }
    pub fn ishr(self) -> Self {
        self.add_instruction(JvmInstruction::Ishr)
    }
    pub fn lshr(self) -> Self {
        self.add_instruction(JvmInstruction::Lshr)
    }
    pub fn iushr(self) -> Self {
        self.add_instruction(JvmInstruction::Iushr)
    }
    pub fn lushr(self) -> Self {
        self.add_instruction(JvmInstruction::Lushr)
    }
    pub fn iand(self) -> Self {
        self.add_instruction(JvmInstruction::Iand)
    }
    pub fn land(self) -> Self {
        self.add_instruction(JvmInstruction::Land)
    }
    pub fn ior(self) -> Self {
        self.add_instruction(JvmInstruction::Ior)
    }
    pub fn lor(self) -> Self {
        self.add_instruction(JvmInstruction::Lor)
    }
    pub fn ixor(self) -> Self {
        self.add_instruction(JvmInstruction::Ixor)
    }
    pub fn lxor(self) -> Self {
        self.add_instruction(JvmInstruction::Lxor)
    }

    pub fn lcmp(self) -> Self {
        self.add_instruction(JvmInstruction::Lcmp)
    }
    pub fn fcmpl(self) -> Self {
        self.add_instruction(JvmInstruction::Fcmpl)
    }
    pub fn fcmpg(self) -> Self {
        self.add_instruction(JvmInstruction::Fcmpg)
    }
    pub fn dcmpl(self) -> Self {
        self.add_instruction(JvmInstruction::Dcmpl)
    }
    pub fn dcmpg(self) -> Self {
        self.add_instruction(JvmInstruction::Dcmpg)
    }

    pub fn iinc(self, index: u16, increment: i16) -> Self {
        self.add_instruction(JvmInstruction::Iinc { index, increment })
    }

    pub fn getstatic(self, class: impl Into<String>, name: impl Into<String>, desc: impl Into<String>) -> Self {
        self.add_instruction(JvmInstruction::Getstatic {
            class_name: class.into(),
            field_name: name.into(),
            descriptor: desc.into(),
        })
    }

    pub fn putstatic(self, class: impl Into<String>, name: impl Into<String>, desc: impl Into<String>) -> Self {
        self.add_instruction(JvmInstruction::Putstatic {
            class_name: class.into(),
            field_name: name.into(),
            descriptor: desc.into(),
        })
    }

    pub fn getfield(self, class: impl Into<String>, name: impl Into<String>, desc: impl Into<String>) -> Self {
        self.add_instruction(JvmInstruction::Getfield {
            class_name: class.into(),
            field_name: name.into(),
            descriptor: desc.into(),
        })
    }

    pub fn putfield(self, class: impl Into<String>, name: impl Into<String>, desc: impl Into<String>) -> Self {
        self.add_instruction(JvmInstruction::Putfield {
            class_name: class.into(),
            field_name: name.into(),
            descriptor: desc.into(),
        })
    }

    pub fn invokevirtual(self, class: impl Into<String>, name: impl Into<String>, desc: impl Into<String>) -> Self {
        self.add_instruction(JvmInstruction::Invokevirtual {
            class_name: class.into(),
            method_name: name.into(),
            descriptor: desc.into(),
        })
    }

    pub fn invokespecial(self, class: impl Into<String>, name: impl Into<String>, desc: impl Into<String>) -> Self {
        self.add_instruction(JvmInstruction::Invokespecial {
            class_name: class.into(),
            method_name: name.into(),
            descriptor: desc.into(),
        })
    }

    pub fn invokestatic(self, class: impl Into<String>, name: impl Into<String>, desc: impl Into<String>) -> Self {
        self.add_instruction(JvmInstruction::Invokestatic {
            class_name: class.into(),
            method_name: name.into(),
            descriptor: desc.into(),
        })
    }

    pub fn invokeinterface(self, class: impl Into<String>, name: impl Into<String>, desc: impl Into<String>) -> Self {
        self.add_instruction(JvmInstruction::Invokeinterface {
            class_name: class.into(),
            method_name: name.into(),
            descriptor: desc.into(),
        })
    }

    pub fn new_instance(self, class: impl Into<String>) -> Self {
        self.add_instruction(JvmInstruction::New { class_name: class.into() })
    }

    pub fn dup(self) -> Self {
        self.add_instruction(JvmInstruction::Dup)
    }
    pub fn dup_x1(self) -> Self {
        self.add_instruction(JvmInstruction::DupX1)
    }
    pub fn dup_x2(self) -> Self {
        self.add_instruction(JvmInstruction::DupX2)
    }
    pub fn dup2(self) -> Self {
        self.add_instruction(JvmInstruction::Dup2)
    }
    pub fn dup2_x1(self) -> Self {
        self.add_instruction(JvmInstruction::Dup2X1)
    }
    pub fn dup2_x2(self) -> Self {
        self.add_instruction(JvmInstruction::Dup2X2)
    }
    pub fn swap(self) -> Self {
        self.add_instruction(JvmInstruction::Swap)
    }
    pub fn pop(self) -> Self {
        self.add_instruction(JvmInstruction::Pop)
    }
    pub fn pop2(self) -> Self {
        self.add_instruction(JvmInstruction::Pop2)
    }

    pub fn return_void(self) -> Self {
        self.add_instruction(JvmInstruction::Return)
    }
    pub fn ireturn(self) -> Self {
        self.add_instruction(JvmInstruction::Ireturn)
    }
    pub fn lreturn(self) -> Self {
        self.add_instruction(JvmInstruction::Lreturn)
    }
    pub fn freturn(self) -> Self {
        self.add_instruction(JvmInstruction::Freturn)
    }
    pub fn dreturn(self) -> Self {
        self.add_instruction(JvmInstruction::Dreturn)
    }
    pub fn areturn(self) -> Self {
        self.add_instruction(JvmInstruction::Areturn)
    }

    pub fn label(self, name: impl Into<String>) -> Self {
        self.add_instruction(JvmInstruction::Label { name: name.into() })
    }

    pub fn goto(self, target: impl Into<String>) -> Self {
        self.add_instruction(JvmInstruction::Goto { target: target.into() })
    }

    pub fn ifeq(self, target: impl Into<String>) -> Self {
        self.add_instruction(JvmInstruction::Ifeq { target: target.into() })
    }

    pub fn ifne(self, target: impl Into<String>) -> Self {
        self.add_instruction(JvmInstruction::Ifne { target: target.into() })
    }

    pub fn build(self) -> JvmMethod {
        self.method
    }
}

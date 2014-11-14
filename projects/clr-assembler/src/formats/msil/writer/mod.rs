use gaia_types::{writer::TextWriter, Result};
use std::fmt::Write;

/// MSIL 代码写入器
///
/// 这个结构体提供了生成 MSIL (Microsoft Intermediate Language) 代码的功能，
/// 复用了 gaia-types 的 TextWriter 来处理格式化和缩进。
#[derive(Debug)]
pub struct MsilWriter<W> {
    /// 底层的文本写入器
    writer: TextWriter<W>,
}

impl<W: Write> MsilWriter<W> {
    /// 创建一个新的 MSIL 写入器
    ///
    /// # 参数
    ///
    /// * `writer` - 底层的写入器
    ///
    /// # 返回值
    ///
    /// 返回一个新的 `MsilWriter` 实例
    pub fn new(writer: W) -> Self {
        Self { writer: TextWriter::new(writer) }
    }

    /// 写入程序集声明
    pub fn write_assembly(&mut self, name: &str) -> Result<()> {
        self.writer.write_line(&format!(".assembly {}", name))?;
        self.writer.write_line("{")?;
        self.writer.write_line("}")?;
        Ok(())
    }

    /// 开始方法定义
    pub fn start_method(&mut self, name: &str, parameters: &[&str], return_type: Option<&str>) -> Result<()> {
        let return_type_str = return_type.unwrap_or("void");

        self.writer.write_line(&format!(".method public static {} {}() cil managed", return_type_str, name))?;
        self.writer.write_line("{")?;

        // 如果是 main 方法，添加入口点标记
        if name == "main" {
            self.writer.write_line(".entrypoint")?;
        }

        Ok(())
    }

    /// 结束方法定义
    pub fn end_method(&mut self) -> Result<()> {
        self.writer.write_line("}")?;
        Ok(())
    }

    /// 生成 ldc.i4 指令（加载 32 位整数常量）
    pub fn emit_ldc_i4(&mut self, value: i32) -> Result<()> {
        self.writer.write_line(&format!("ldc.i4 {}", value))?;
        Ok(())
    }

    /// 生成 ldc.i8 指令（加载 64 位整数常量）
    pub fn emit_ldc_i8(&mut self, value: i64) -> Result<()> {
        self.writer.write_line(&format!("ldc.i8 {}", value))?;
        Ok(())
    }

    /// 生成 ldc.r4 指令（加载 32 位浮点常量）
    pub fn emit_ldc_r4(&mut self, value: f32) -> Result<()> {
        self.writer.write_line(&format!("ldc.r4 {}", value))?;
        Ok(())
    }

    /// 生成 ldc.r8 指令（加载 64 位浮点常量）
    pub fn emit_ldc_r8(&mut self, value: f64) -> Result<()> {
        self.writer.write_line(&format!("ldc.r8 {}", value))?;
        Ok(())
    }

    /// 生成 ldstr 指令（加载字符串常量）
    pub fn emit_ldstr(&mut self, value: &str) -> Result<()> {
        self.writer.write_line(&format!("ldstr \"{}\"", value))?;
        Ok(())
    }

    /// 生成 ldnull 指令（加载 null 引用）
    pub fn emit_ldnull(&mut self) -> Result<()> {
        self.writer.write_line("ldnull")?;
        Ok(())
    }

    // 局部变量操作指令

    /// 生成 ldloc 指令（加载局部变量）
    pub fn emit_ldloc(&mut self, index: u32) -> Result<()> {
        self.writer.write_line(&format!("ldloc {}", index))?;
        Ok(())
    }

    /// 生成 stloc 指令（存储局部变量）
    pub fn emit_stloc(&mut self, index: u32) -> Result<()> {
        self.writer.write_line(&format!("stloc {}", index))?;
        Ok(())
    }

    /// 生成 ldloca 指令（加载局部变量地址）
    pub fn emit_ldloca(&mut self, index: u32) -> Result<()> {
        self.writer.write_line(&format!("ldloca {}", index))?;
        Ok(())
    }

    // 参数操作指令

    /// 生成 ldarg 指令（加载参数）
    pub fn emit_ldarg(&mut self, index: u32) -> Result<()> {
        self.writer.write_line(&format!("ldarg {}", index))?;
        Ok(())
    }

    // 算术运算指令

    /// 生成 add 指令（加法）
    pub fn emit_add(&mut self) -> Result<()> {
        self.writer.write_line("add")?;
        Ok(())
    }

    /// 生成 sub 指令（减法）
    pub fn emit_sub(&mut self) -> Result<()> {
        self.writer.write_line("sub")?;
        Ok(())
    }

    /// 生成 mul 指令（乘法）
    pub fn emit_mul(&mut self) -> Result<()> {
        self.writer.write_line("mul")?;
        Ok(())
    }

    /// 生成 div 指令（除法）
    pub fn emit_div(&mut self) -> Result<()> {
        self.writer.write_line("div")?;
        Ok(())
    }

    // 比较指令

    /// 生成 ceq 指令（相等比较）
    pub fn emit_ceq(&mut self) -> Result<()> {
        self.writer.write_line("ceq")?;
        Ok(())
    }

    /// 生成 clt 指令（小于比较）
    pub fn emit_clt(&mut self) -> Result<()> {
        self.writer.write_line("clt")?;
        Ok(())
    }

    /// 生成 cgt 指令（大于比较）
    pub fn emit_cgt(&mut self) -> Result<()> {
        self.writer.write_line("cgt")?;
        Ok(())
    }

    // 分支指令

    /// 生成 br 指令（无条件分支）
    pub fn emit_br(&mut self, label: &str) -> Result<()> {
        self.writer.write_line(&format!("br {}", label))?;
        Ok(())
    }

    /// 生成 brtrue 指令（条件分支 - 真）
    pub fn emit_brtrue(&mut self, label: &str) -> Result<()> {
        self.writer.write_line(&format!("brtrue {}", label))?;
        Ok(())
    }

    /// 生成 brfalse 指令（条件分支 - 假）
    pub fn emit_brfalse(&mut self, label: &str) -> Result<()> {
        self.writer.write_line(&format!("brfalse {}", label))?;
        Ok(())
    }

    // 方法调用指令

    /// 生成 call 指令（方法调用）
    pub fn emit_call(&mut self, method_name: &str) -> Result<()> {
        self.writer.write_line(&format!("call {}", method_name))?;
        Ok(())
    }

    /// 生成 ret 指令（返回）
    pub fn emit_ret(&mut self) -> Result<()> {
        self.writer.write_line("ret")?;
        Ok(())
    }

    // 栈操作指令

    /// 生成 dup 指令（复制栈顶元素）
    pub fn emit_dup(&mut self) -> Result<()> {
        self.writer.write_line("dup")?;
        Ok(())
    }

    /// 生成 pop 指令（弹出栈顶元素）
    pub fn emit_pop(&mut self) -> Result<()> {
        self.writer.write_line("pop")?;
        Ok(())
    }

    // 间接访问指令

    /// 生成 ldind.i4 指令（间接加载 32 位整数）
    pub fn emit_ldind_i4(&mut self) -> Result<()> {
        self.writer.write_line("ldind.i4")?;
        Ok(())
    }

    /// 生成 ldind.i8 指令（间接加载 64 位整数）
    pub fn emit_ldind_i8(&mut self) -> Result<()> {
        self.writer.write_line("ldind.i8")?;
        Ok(())
    }

    /// 生成 ldind.r4 指令（间接加载 32 位浮点数）
    pub fn emit_ldind_r4(&mut self) -> Result<()> {
        self.writer.write_line("ldind.r4")?;
        Ok(())
    }

    /// 生成 ldind.r8 指令（间接加载 64 位浮点数）
    pub fn emit_ldind_r8(&mut self) -> Result<()> {
        self.writer.write_line("ldind.r8")?;
        Ok(())
    }

    /// 生成 ldind.ref 指令（间接加载引用）
    pub fn emit_ldind_ref(&mut self) -> Result<()> {
        self.writer.write_line("ldind.ref")?;
        Ok(())
    }

    /// 生成 stind.i4 指令（间接存储 32 位整数）
    pub fn emit_stind_i4(&mut self) -> Result<()> {
        self.writer.write_line("stind.i4")?;
        Ok(())
    }

    /// 生成 stind.i8 指令（间接存储 64 位整数）
    pub fn emit_stind_i8(&mut self) -> Result<()> {
        self.writer.write_line("stind.i8")?;
        Ok(())
    }

    /// 生成 stind.r4 指令（间接存储 32 位浮点数）
    pub fn emit_stind_r4(&mut self) -> Result<()> {
        self.writer.write_line("stind.r4")?;
        Ok(())
    }

    /// 生成 stind.r8 指令（间接存储 64 位浮点数）
    pub fn emit_stind_r8(&mut self) -> Result<()> {
        self.writer.write_line("stind.r8")?;
        Ok(())
    }

    /// 生成 stind.ref 指令（间接存储引用）
    pub fn emit_stind_ref(&mut self) -> Result<()> {
        self.writer.write_line("stind.ref")?;
        Ok(())
    }

    // 类型转换指令

    /// 生成 conv.i4 指令（转换为 32 位整数）
    pub fn emit_conv_i4(&mut self) -> Result<()> {
        self.writer.write_line("conv.i4")?;
        Ok(())
    }

    /// 生成 conv.i8 指令（转换为 64 位整数）
    pub fn emit_conv_i8(&mut self) -> Result<()> {
        self.writer.write_line("conv.i8")?;
        Ok(())
    }

    /// 生成 conv.r4 指令（转换为 32 位浮点数）
    pub fn emit_conv_r4(&mut self) -> Result<()> {
        self.writer.write_line("conv.r4")?;
        Ok(())
    }

    /// 生成 conv.r8 指令（转换为 64 位浮点数）
    pub fn emit_conv_r8(&mut self) -> Result<()> {
        self.writer.write_line("conv.r8")?;
        Ok(())
    }

    // 装箱/拆箱指令

    /// 生成 box 指令（装箱）
    pub fn emit_box(&mut self, type_name: &str) -> Result<()> {
        self.writer.write_line(&format!("box {}", type_name))?;
        Ok(())
    }

    /// 生成 unbox 指令（拆箱）
    pub fn emit_unbox(&mut self, type_name: &str) -> Result<()> {
        self.writer.write_line(&format!("unbox {}", type_name))?;
        Ok(())
    }

    // 标签定义

    /// 定义标签
    pub fn define_label(&mut self, name: &str) -> Result<()> {
        self.writer.write_line(&format!("{}:", name))?;
        Ok(())
    }

    /// 获取结果
    pub fn finish(self) -> W {
        self.writer.finish()
    }
}

use crate::program::*;
use crate::formats::class::writer::utils::calculate_parameter_count;
use std::io::Write;
use super::ClassWriter;

impl<W: Write> ClassWriter<W> {
    /// 生成方法的字节码及标签位置
    pub fn generate_method_bytecode(&mut self, method: &JvmMethod) -> (Vec<u8>, std::collections::HashMap<String, i32>) {
        let mut bytecode = Vec::new();
        let mut label_positions = std::collections::HashMap::new();
        let mut jump_patches = Vec::new(); // (instr_pos, patch_pos, target_label_name, is_wide)

        // 第一遍：计算指令位置并记录标签位置
        for instruction in &method.instructions {
            match instruction {
                JvmInstruction::Label { name } => {
                    label_positions.insert(name.clone(), bytecode.len() as i32);
                }
                _ => {
                    let pos = bytecode.len();
                    self.emit_instruction(instruction, &mut bytecode, &mut jump_patches, pos);
                }
            }
        }

        // 第二遍：解析跳转偏移量
        for (instr_pos, patch_pos, target_name, is_wide) in jump_patches {
            if let Some(&target_pos) = label_positions.get(&target_name) {
                let offset = target_pos - instr_pos as i32;
                if is_wide {
                    // 4 字节偏移量
                    bytecode[patch_pos] = ((offset >> 24) & 0xFF) as u8;
                    bytecode[patch_pos + 1] = ((offset >> 16) & 0xFF) as u8;
                    bytecode[patch_pos + 2] = ((offset >> 8) & 0xFF) as u8;
                    bytecode[patch_pos + 3] = (offset & 0xFF) as u8;
                }
                else {
                    // 2 字节偏移量
                    bytecode[patch_pos] = ((offset >> 8) & 0xFF) as u8;
                    bytecode[patch_pos + 1] = (offset & 0xFF) as u8;
                }
            }
        }

        // 如果方法没有指令，添加一个 return 指令
        if bytecode.is_empty() {
            bytecode.push(0xB1); // return
        }

        (bytecode, label_positions)
    }

    /// 发射单条指令的字节码
    pub fn emit_instruction(
        &mut self,
        instruction: &JvmInstruction,
        bytecode: &mut Vec<u8>,
        jump_patches: &mut Vec<(usize, usize, String, bool)>,
        pos: usize,
    ) {
        match instruction {
            JvmInstruction::Nop => bytecode.push(0x00),
            JvmInstruction::AconstNull => bytecode.push(0x01),
            JvmInstruction::IconstM1 => bytecode.push(0x02),
            JvmInstruction::Iconst0 => bytecode.push(0x03),
            JvmInstruction::Iconst1 => bytecode.push(0x04),
            JvmInstruction::Iconst2 => bytecode.push(0x05),
            JvmInstruction::Iconst3 => bytecode.push(0x06),
            JvmInstruction::Iconst4 => bytecode.push(0x07),
            JvmInstruction::Iconst5 => bytecode.push(0x08),
            JvmInstruction::Lconst0 => bytecode.push(0x09),
            JvmInstruction::Lconst1 => bytecode.push(0x0A),
            JvmInstruction::Fconst0 => bytecode.push(0x0B),
            JvmInstruction::Fconst1 => bytecode.push(0x0C),
            JvmInstruction::Fconst2 => bytecode.push(0x0D),
            JvmInstruction::Dconst0 => bytecode.push(0x0E),
            JvmInstruction::Dconst1 => bytecode.push(0x0F),

            JvmInstruction::Bipush { value } => {
                bytecode.push(0x10);
                bytecode.push(*value as u8);
            }
            JvmInstruction::Sipush { value } => {
                bytecode.push(0x11);
                bytecode.push((*value >> 8) as u8);
                bytecode.push((*value & 0xFF) as u8);
            }

            JvmInstruction::Ldc { symbol } => {
                let index = self.add_string(symbol.clone());
                if index < 256 {
                    bytecode.push(0x12); // ldc
                    bytecode.push(index as u8);
                }
                else {
                    bytecode.push(0x13); // ldc_w
                    bytecode.push((index >> 8) as u8);
                    bytecode.push((index & 0xFF) as u8);
                }
            }
            JvmInstruction::LdcW { symbol } => {
                let index = self.add_string(symbol.clone());
                bytecode.push(0x13); // ldc_w
                bytecode.push((index >> 8) as u8);
                bytecode.push((index & 0xFF) as u8);
            }
            JvmInstruction::Ldc2W { symbol } => {
                let index = self.add_string(symbol.clone());
                bytecode.push(0x14); // ldc2_w
                bytecode.push((index >> 8) as u8);
                bytecode.push((index & 0xFF) as u8);
            }

            JvmInstruction::Iload { index } => {
                if *index <= 3 {
                    bytecode.push(0x1A + *index as u8);
                }
                else if *index <= 255 {
                    bytecode.push(0x15);
                    bytecode.push(*index as u8);
                }
                else {
                    bytecode.push(0xC4); // wide
                    bytecode.push(0x15);
                    bytecode.push((*index >> 8) as u8);
                    bytecode.push((*index & 0xFF) as u8);
                }
            }
            JvmInstruction::Iload0 => bytecode.push(0x1A),
            JvmInstruction::Iload1 => bytecode.push(0x1B),
            JvmInstruction::Iload2 => bytecode.push(0x1C),
            JvmInstruction::Iload3 => bytecode.push(0x1D),

            JvmInstruction::Lload { index } => {
                if *index <= 3 {
                    bytecode.push(0x1E + *index as u8);
                }
                else if *index <= 255 {
                    bytecode.push(0x16);
                    bytecode.push(*index as u8);
                }
                else {
                    bytecode.push(0xC4); // wide
                    bytecode.push(0x16);
                    bytecode.push((*index >> 8) as u8);
                    bytecode.push((*index & 0xFF) as u8);
                }
            }
            JvmInstruction::Lload0 => bytecode.push(0x1E),
            JvmInstruction::Lload1 => bytecode.push(0x1F),
            JvmInstruction::Lload2 => bytecode.push(0x20),
            JvmInstruction::Lload3 => bytecode.push(0x21),

            JvmInstruction::Fload { index } => {
                if *index <= 3 {
                    bytecode.push(0x22 + *index as u8);
                }
                else if *index <= 255 {
                    bytecode.push(0x17);
                    bytecode.push(*index as u8);
                }
                else {
                    bytecode.push(0xC4); // wide
                    bytecode.push(0x17);
                    bytecode.push((*index >> 8) as u8);
                    bytecode.push((*index & 0xFF) as u8);
                }
            }
            JvmInstruction::Fload0 => bytecode.push(0x22),
            JvmInstruction::Fload1 => bytecode.push(0x23),
            JvmInstruction::Fload2 => bytecode.push(0x24),
            JvmInstruction::Fload3 => bytecode.push(0x25),

            JvmInstruction::Dload { index } => {
                if *index <= 3 {
                    bytecode.push(0x26 + *index as u8);
                }
                else if *index <= 255 {
                    bytecode.push(0x18);
                    bytecode.push(*index as u8);
                }
                else {
                    bytecode.push(0xC4); // wide
                    bytecode.push(0x18);
                    bytecode.push((*index >> 8) as u8);
                    bytecode.push((*index & 0xFF) as u8);
                }
            }
            JvmInstruction::Dload0 => bytecode.push(0x26),
            JvmInstruction::Dload1 => bytecode.push(0x27),
            JvmInstruction::Dload2 => bytecode.push(0x28),
            JvmInstruction::Dload3 => bytecode.push(0x29),

            JvmInstruction::Aload { index } => {
                if *index <= 3 {
                    bytecode.push(0x2A + *index as u8);
                }
                else if *index <= 255 {
                    bytecode.push(0x19);
                    bytecode.push(*index as u8);
                }
                else {
                    bytecode.push(0xC4); // wide
                    bytecode.push(0x19);
                    bytecode.push((*index >> 8) as u8);
                    bytecode.push((*index & 0xFF) as u8);
                }
            }
            JvmInstruction::Aload0 => bytecode.push(0x2A),
            JvmInstruction::Aload1 => bytecode.push(0x2B),
            JvmInstruction::Aload2 => bytecode.push(0x2C),
            JvmInstruction::Aload3 => bytecode.push(0x2D),

            JvmInstruction::Istore { index } => {
                if *index <= 3 {
                    bytecode.push(0x3B + *index as u8);
                }
                else if *index <= 255 {
                    bytecode.push(0x36);
                    bytecode.push(*index as u8);
                }
                else {
                    bytecode.push(0xC4); // wide
                    bytecode.push(0x36);
                    bytecode.push((*index >> 8) as u8);
                    bytecode.push((*index & 0xFF) as u8);
                }
            }
            JvmInstruction::Istore0 => bytecode.push(0x3B),
            JvmInstruction::Istore1 => bytecode.push(0x3C),
            JvmInstruction::Istore2 => bytecode.push(0x3D),
            JvmInstruction::Istore3 => bytecode.push(0x3E),

            JvmInstruction::Lstore { index } => {
                if *index <= 3 {
                    bytecode.push(0x3F + *index as u8);
                }
                else if *index <= 255 {
                    bytecode.push(0x37);
                    bytecode.push(*index as u8);
                }
                else {
                    bytecode.push(0xC4); // wide
                    bytecode.push(0x37);
                    bytecode.push((*index >> 8) as u8);
                    bytecode.push((*index & 0xFF) as u8);
                }
            }
            JvmInstruction::Lstore0 => bytecode.push(0x3F),
            JvmInstruction::Lstore1 => bytecode.push(0x40),
            JvmInstruction::Lstore2 => bytecode.push(0x41),
            JvmInstruction::Lstore3 => bytecode.push(0x42),

            JvmInstruction::Fstore { index } => {
                if *index <= 3 {
                    bytecode.push(0x43 + *index as u8);
                }
                else if *index <= 255 {
                    bytecode.push(0x38);
                    bytecode.push(*index as u8);
                }
                else {
                    bytecode.push(0xC4); // wide
                    bytecode.push(0x38);
                    bytecode.push((*index >> 8) as u8);
                    bytecode.push((*index & 0xFF) as u8);
                }
            }
            JvmInstruction::Fstore0 => bytecode.push(0x43),
            JvmInstruction::Fstore1 => bytecode.push(0x44),
            JvmInstruction::Fstore2 => bytecode.push(0x45),
            JvmInstruction::Fstore3 => bytecode.push(0x46),

            JvmInstruction::Dstore { index } => {
                if *index <= 3 {
                    bytecode.push(0x47 + *index as u8);
                }
                else if *index <= 255 {
                    bytecode.push(0x39);
                    bytecode.push(*index as u8);
                }
                else {
                    bytecode.push(0xC4); // wide
                    bytecode.push(0x39);
                    bytecode.push((*index >> 8) as u8);
                    bytecode.push((*index & 0xFF) as u8);
                }
            }
            JvmInstruction::Dstore0 => bytecode.push(0x47),
            JvmInstruction::Dstore1 => bytecode.push(0x48),
            JvmInstruction::Dstore2 => bytecode.push(0x49),
            JvmInstruction::Dstore3 => bytecode.push(0x4A),

            JvmInstruction::Astore { index } => {
                if *index <= 3 {
                    bytecode.push(0x4B + *index as u8);
                }
                else if *index <= 255 {
                    bytecode.push(0x3A);
                    bytecode.push(*index as u8);
                }
                else {
                    bytecode.push(0xC4); // wide
                    bytecode.push(0x3A);
                    bytecode.push((*index >> 8) as u8);
                    bytecode.push((*index & 0xFF) as u8);
                }
            }
            JvmInstruction::Astore0 => bytecode.push(0x4B),
            JvmInstruction::Astore1 => bytecode.push(0x4C),
            JvmInstruction::Astore2 => bytecode.push(0x4D),
            JvmInstruction::Astore3 => bytecode.push(0x4E),

            JvmInstruction::Pop => bytecode.push(0x57),
            JvmInstruction::Pop2 => bytecode.push(0x58),
            JvmInstruction::Dup => bytecode.push(0x59),
            JvmInstruction::DupX1 => bytecode.push(0x5A),
            JvmInstruction::DupX2 => bytecode.push(0x5B),
            JvmInstruction::Dup2 => bytecode.push(0x5C),
            JvmInstruction::Dup2X1 => bytecode.push(0x5D),
            JvmInstruction::Dup2X2 => bytecode.push(0x5E),
            JvmInstruction::Swap => bytecode.push(0x5F),

            JvmInstruction::Iadd => bytecode.push(0x60),
            JvmInstruction::Ladd => bytecode.push(0x61),
            JvmInstruction::Fadd => bytecode.push(0x62),
            JvmInstruction::Dadd => bytecode.push(0x63),
            JvmInstruction::Isub => bytecode.push(0x64),
            JvmInstruction::Lsub => bytecode.push(0x65),
            JvmInstruction::Fsub => bytecode.push(0x66),
            JvmInstruction::Dsub => bytecode.push(0x67),
            JvmInstruction::Imul => bytecode.push(0x68),
            JvmInstruction::Lmul => bytecode.push(0x69),
            JvmInstruction::Fmul => bytecode.push(0x6A),
            JvmInstruction::Dmul => bytecode.push(0x6B),
            JvmInstruction::Idiv => bytecode.push(0x6C),
            JvmInstruction::Ldiv => bytecode.push(0x6D),
            JvmInstruction::Fdiv => bytecode.push(0x6E),
            JvmInstruction::Ddiv => bytecode.push(0x6F),
            JvmInstruction::Irem => bytecode.push(0x70),
            JvmInstruction::Lrem => bytecode.push(0x71),
            JvmInstruction::Frem => bytecode.push(0x72),
            JvmInstruction::Drem => bytecode.push(0x73),
            JvmInstruction::Ineg => bytecode.push(0x74),
            JvmInstruction::Lneg => bytecode.push(0x75),
            JvmInstruction::Fneg => bytecode.push(0x76),
            JvmInstruction::Dneg => bytecode.push(0x77),

            JvmInstruction::Ishl => bytecode.push(0x78),
            JvmInstruction::Lshl => bytecode.push(0x79),
            JvmInstruction::Ishr => bytecode.push(0x7A),
            JvmInstruction::Lshr => bytecode.push(0x7B),
            JvmInstruction::Iushr => bytecode.push(0x7C),
            JvmInstruction::Lushr => bytecode.push(0x7D),
            JvmInstruction::Iand => bytecode.push(0x7E),
            JvmInstruction::Land => bytecode.push(0x7F),
            JvmInstruction::Ior => bytecode.push(0x80),
            JvmInstruction::Lor => bytecode.push(0x81),
            JvmInstruction::Ixor => bytecode.push(0x82),
            JvmInstruction::Lxor => bytecode.push(0x83),

            JvmInstruction::Lcmp => bytecode.push(0x94),
            JvmInstruction::Fcmpl => bytecode.push(0x95),
            JvmInstruction::Fcmpg => bytecode.push(0x96),
            JvmInstruction::Dcmpl => bytecode.push(0x97),
            JvmInstruction::Dcmpg => bytecode.push(0x98),

            JvmInstruction::Ifeq { target } => {
                jump_patches.push((pos, pos + 1, target.clone(), false));
                bytecode.push(0x99);
                bytecode.push(0);
                bytecode.push(0);
            }
            JvmInstruction::Ifne { target } => {
                jump_patches.push((pos, pos + 1, target.clone(), false));
                bytecode.push(0x9A);
                bytecode.push(0);
                bytecode.push(0);
            }
            JvmInstruction::Iflt { target } => {
                jump_patches.push((pos, pos + 1, target.clone(), false));
                bytecode.push(0x9B);
                bytecode.push(0);
                bytecode.push(0);
            }
            JvmInstruction::Ifge { target } => {
                jump_patches.push((pos, pos + 1, target.clone(), false));
                bytecode.push(0x9C);
                bytecode.push(0);
                bytecode.push(0);
            }
            JvmInstruction::Ifgt { target } => {
                jump_patches.push((pos, pos + 1, target.clone(), false));
                bytecode.push(0x9D);
                bytecode.push(0);
                bytecode.push(0);
            }
            JvmInstruction::Ifle { target } => {
                jump_patches.push((pos, pos + 1, target.clone(), false));
                bytecode.push(0x9E);
                bytecode.push(0);
                bytecode.push(0);
            }
            JvmInstruction::IfIcmpeq { target } => {
                jump_patches.push((pos, pos + 1, target.clone(), false));
                bytecode.push(0x9F);
                bytecode.push(0);
                bytecode.push(0);
            }
            JvmInstruction::IfIcmpne { target } => {
                jump_patches.push((pos, pos + 1, target.clone(), false));
                bytecode.push(0xA0);
                bytecode.push(0);
                bytecode.push(0);
            }
            JvmInstruction::IfIcmplt { target } => {
                jump_patches.push((pos, pos + 1, target.clone(), false));
                bytecode.push(0xA1);
                bytecode.push(0);
                bytecode.push(0);
            }
            JvmInstruction::IfIcmpge { target } => {
                jump_patches.push((pos, pos + 1, target.clone(), false));
                bytecode.push(0xA2);
                bytecode.push(0);
                bytecode.push(0);
            }
            JvmInstruction::IfIcmpgt { target } => {
                jump_patches.push((pos, pos + 1, target.clone(), false));
                bytecode.push(0xA3);
                bytecode.push(0);
                bytecode.push(0);
            }
            JvmInstruction::IfIcmple { target } => {
                jump_patches.push((pos, pos + 1, target.clone(), false));
                bytecode.push(0xA4);
                bytecode.push(0);
                bytecode.push(0);
            }
            JvmInstruction::IfAcmpeq { target } => {
                jump_patches.push((pos, pos + 1, target.clone(), false));
                bytecode.push(0xA5);
                bytecode.push(0);
                bytecode.push(0);
            }
            JvmInstruction::IfAcmpne { target } => {
                jump_patches.push((pos, pos + 1, target.clone(), false));
                bytecode.push(0xA6);
                bytecode.push(0);
                bytecode.push(0);
            }
            JvmInstruction::Ifnull { target } => {
                jump_patches.push((pos, pos + 1, target.clone(), false));
                bytecode.push(0xC6);
                bytecode.push(0);
                bytecode.push(0);
            }
            JvmInstruction::Ifnonnull { target } => {
                jump_patches.push((pos, pos + 1, target.clone(), false));
                bytecode.push(0xC7);
                bytecode.push(0);
                bytecode.push(0);
            }
            JvmInstruction::Goto { target } => {
                jump_patches.push((pos, pos + 1, target.clone(), false));
                bytecode.push(0xA7);
                bytecode.push(0);
                bytecode.push(0);
            }
            JvmInstruction::GotoW { target } => {
                jump_patches.push((pos, pos + 1, target.clone(), true));
                bytecode.push(0xC8);
                bytecode.push(0);
                bytecode.push(0);
                bytecode.push(0);
                bytecode.push(0);
            }

            JvmInstruction::Return => bytecode.push(0xB1),
            JvmInstruction::Ireturn => bytecode.push(0xAC),
            JvmInstruction::Lreturn => bytecode.push(0xAD),
            JvmInstruction::Freturn => bytecode.push(0xAE),
            JvmInstruction::Dreturn => bytecode.push(0xAF),
            JvmInstruction::Areturn => bytecode.push(0xB0),

            JvmInstruction::Arraylength => bytecode.push(0xBE),
            JvmInstruction::Athrow => bytecode.push(0xBF),
            JvmInstruction::Monitorenter => bytecode.push(0xC2),
            JvmInstruction::Monitorexit => bytecode.push(0xC3),

            JvmInstruction::Iaload => bytecode.push(0x2E),
            JvmInstruction::Laload => bytecode.push(0x2F),
            JvmInstruction::Faload => bytecode.push(0x30),
            JvmInstruction::Daload => bytecode.push(0x31),
            JvmInstruction::Aaload => bytecode.push(0x32),
            JvmInstruction::Baload => bytecode.push(0x33),
            JvmInstruction::Saload => bytecode.push(0x34),

            JvmInstruction::Iastore => bytecode.push(0x4F),
            JvmInstruction::Lastore => bytecode.push(0x50),
            JvmInstruction::Fastore => bytecode.push(0x51),
            JvmInstruction::Dastore => bytecode.push(0x52),
            JvmInstruction::Aastore => bytecode.push(0x53),
            JvmInstruction::Bastore => bytecode.push(0x54),
            JvmInstruction::Sastore => bytecode.push(0x55),

            JvmInstruction::Getstatic { class_name, field_name, descriptor } => {
                let index = self.add_field_ref(class_name.clone(), field_name.clone(), descriptor.clone());
                bytecode.push(0xB2); // getstatic
                bytecode.push((index >> 8) as u8);
                bytecode.push((index & 0xFF) as u8);
            }
            JvmInstruction::Putstatic { class_name, field_name, descriptor } => {
                let index = self.add_field_ref(class_name.clone(), field_name.clone(), descriptor.clone());
                bytecode.push(0xB3); // putstatic
                bytecode.push((index >> 8) as u8);
                bytecode.push((index & 0xFF) as u8);
            }
            JvmInstruction::Getfield { class_name, field_name, descriptor } => {
                let index = self.add_field_ref(class_name.clone(), field_name.clone(), descriptor.clone());
                bytecode.push(0xB4); // getfield
                bytecode.push((index >> 8) as u8);
                bytecode.push((index & 0xFF) as u8);
            }
            JvmInstruction::Putfield { class_name, field_name, descriptor } => {
                let index = self.add_field_ref(class_name.clone(), field_name.clone(), descriptor.clone());
                bytecode.push(0xB5); // putfield
                bytecode.push((index >> 8) as u8);
                bytecode.push((index & 0xFF) as u8);
            }
            JvmInstruction::Invokevirtual { class_name, method_name, descriptor } => {
                let index = self.add_method_ref(class_name.clone(), method_name.clone(), descriptor.clone());
                bytecode.push(0xB6); // invokevirtual
                bytecode.push((index >> 8) as u8);
                bytecode.push((index & 0xFF) as u8);
            }
            JvmInstruction::Invokespecial { class_name, method_name, descriptor } => {
                let index = self.add_method_ref(class_name.clone(), method_name.clone(), descriptor.clone());
                bytecode.push(0xB7); // invokespecial
                bytecode.push((index >> 8) as u8);
                bytecode.push((index & 0xFF) as u8);
            }
            JvmInstruction::Invokestatic { class_name, method_name, descriptor } => {
                let index = self.add_method_ref(class_name.clone(), method_name.clone(), descriptor.clone());
                bytecode.push(0xB8); // invokestatic
                bytecode.push((index >> 8) as u8);
                bytecode.push((index & 0xFF) as u8);
            }
            JvmInstruction::Invokeinterface { class_name, method_name, descriptor } => {
                let index = self.add_interface_method_ref(class_name.clone(), method_name.clone(), descriptor.clone());
                bytecode.push(0xB9); // invokeinterface
                bytecode.push((index >> 8) as u8);
                bytecode.push((index & 0xFF) as u8);

                // 计算参数数量（包括 this）
                let count = calculate_parameter_count(descriptor) + 1;
                bytecode.push(count as u8);
                bytecode.push(0);
            }
            JvmInstruction::Invokedynamic { class_name, method_name, descriptor } => {
                let index = self.add_method_ref(class_name.clone(), method_name.clone(), descriptor.clone());
                bytecode.push(0xBA); // invokedynamic
                bytecode.push((index >> 8) as u8);
                bytecode.push((index & 0xFF) as u8);
                bytecode.push(0);
                bytecode.push(0);
            }
            JvmInstruction::New { class_name } => {
                let index = self.add_class(class_name.clone());
                bytecode.push(0xBB); // new
                bytecode.push((index >> 8) as u8);
                bytecode.push((index & 0xFF) as u8);
            }
            JvmInstruction::Newarray { atype } => {
                bytecode.push(0xBC); // newarray
                bytecode.push(*atype);
            }
            JvmInstruction::Anewarray { class_name } => {
                let index = self.add_class(class_name.clone());
                bytecode.push(0xBD); // anewarray
                bytecode.push((index >> 8) as u8);
                bytecode.push((index & 0xFF) as u8);
            }
            JvmInstruction::Multianewarray { class_name, dimensions } => {
                let index = self.add_class(class_name.clone());
                bytecode.push(0xC5); // multianewarray
                bytecode.push((index >> 8) as u8);
                bytecode.push((index & 0xFF) as u8);
                bytecode.push(*dimensions);
            }
            JvmInstruction::Checkcast { class_name } => {
                let index = self.add_class(class_name.clone());
                bytecode.push(0xC0); // checkcast
                bytecode.push((index >> 8) as u8);
                bytecode.push((index & 0xFF) as u8);
            }
            JvmInstruction::Instanceof { class_name } => {
                let index = self.add_class(class_name.clone());
                bytecode.push(0xC1); // instanceof
                bytecode.push((index >> 8) as u8);
                bytecode.push((index & 0xFF) as u8);
            }
            JvmInstruction::Jsr { target } => {
                jump_patches.push((pos, pos + 1, target.clone(), false));
                bytecode.push(0xA8); // jsr
                bytecode.push(0);
                bytecode.push(0);
            }
            JvmInstruction::Ret { index } => {
                if *index <= 255 {
                    bytecode.push(0xA9); // ret
                    bytecode.push(*index as u8);
                }
                else {
                    bytecode.push(0xC4); // wide
                    bytecode.push(0xA9);
                    bytecode.push((*index >> 8) as u8);
                    bytecode.push((*index & 0xFF) as u8);
                }
            }
            JvmInstruction::Iinc { index, increment } => {
                if *index <= 255 && *increment >= -128 && *increment <= 127 {
                    bytecode.push(0x84); // iinc
                    bytecode.push(*index as u8);
                    bytecode.push(*increment as u8);
                }
                else {
                    bytecode.push(0xC4); // wide
                    bytecode.push(0x84);
                    bytecode.push((*index >> 8) as u8);
                    bytecode.push((*index & 0xFF) as u8);
                    bytecode.push((*increment >> 8) as u8);
                    bytecode.push((*increment & 0xFF) as u8);
                }
            }
            JvmInstruction::JsrW { target } => {
                jump_patches.push((pos, pos + 1, target.clone(), true));
                bytecode.push(0xC9); // jsr_w
                bytecode.push(0);
                bytecode.push(0);
                bytecode.push(0);
                bytecode.push(0);
            }

            JvmInstruction::Lookupswitch { default, pairs } => {
                bytecode.push(0xAB); // lookupswitch

                // 填充以对齐到 4 字节边界
                let padding = (4 - (bytecode.len() % 4)) % 4;
                for _ in 0..padding {
                    bytecode.push(0);
                }

                // default 偏移量
                let default_patch_pos = bytecode.len();
                jump_patches.push((pos, default_patch_pos, default.clone(), true));
                bytecode.push(0);
                bytecode.push(0);
                bytecode.push(0);
                bytecode.push(0);

                // npairs
                let npairs = pairs.len() as i32;
                bytecode.push((npairs >> 24) as u8);
                bytecode.push((npairs >> 16) as u8);
                bytecode.push((npairs >> 8) as u8);
                bytecode.push(npairs as u8);

                // match-offset pairs
                for (val, target) in pairs {
                    // match
                    bytecode.push((val >> 24) as u8);
                    bytecode.push((val >> 16) as u8);
                    bytecode.push((val >> 8) as u8);
                    bytecode.push(*val as u8);

                    // offset
                    let patch_pos = bytecode.len();
                    jump_patches.push((pos, patch_pos, target.clone(), true));
                    bytecode.push(0);
                    bytecode.push(0);
                    bytecode.push(0);
                    bytecode.push(0);
                }
            }
            JvmInstruction::Tableswitch { low, high, default, targets } => {
                bytecode.push(0xAA); // tableswitch

                // 填充以对齐到 4 字节边界
                let padding = (4 - (bytecode.len() % 4)) % 4;
                for _ in 0..padding {
                    bytecode.push(0);
                }

                // default 偏移量
                let default_patch_pos = bytecode.len();
                jump_patches.push((pos, default_patch_pos, default.clone(), true));
                bytecode.push(0);
                bytecode.push(0);
                bytecode.push(0);
                bytecode.push(0);

                // low
                bytecode.push((low >> 24) as u8);
                bytecode.push((low >> 16) as u8);
                bytecode.push((low >> 8) as u8);
                bytecode.push(*low as u8);

                // high
                bytecode.push((high >> 24) as u8);
                bytecode.push((high >> 16) as u8);
                bytecode.push((high >> 8) as u8);
                bytecode.push(*high as u8);

                // offsets
                for target in targets {
                    let patch_pos = bytecode.len();
                    jump_patches.push((pos, patch_pos, target.clone(), true));
                    bytecode.push(0);
                    bytecode.push(0);
                    bytecode.push(0);
                    bytecode.push(0);
                }
            }
            JvmInstruction::Label { .. } => {}           // 已经在第一遍处理过
            JvmInstruction::Wide => bytecode.push(0xC4), // standalone wide is unusual but possible
        }
    }
}

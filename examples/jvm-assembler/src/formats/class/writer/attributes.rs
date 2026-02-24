use crate::program::*;
use gaia_types::Result;
use std::io::Write;
use super::ClassWriter;

impl<W: Write> ClassWriter<W> {
    pub fn collect_attribute_constants(&mut self, attribute: &JvmAttribute) {
        match attribute {
            JvmAttribute::SourceFile { filename } => {
                self.add_utf8("SourceFile".to_string());
                self.add_utf8(filename.clone());
            }
            JvmAttribute::ConstantValue { value } => {
                self.add_utf8("ConstantValue".to_string());
                match value {
                    JvmConstantPoolEntry::Integer { .. } => {}
                    JvmConstantPoolEntry::Float { .. } => {}
                    JvmConstantPoolEntry::Long { .. } => {}
                    JvmConstantPoolEntry::Double { .. } => {}
                    JvmConstantPoolEntry::String { value } => {
                        self.add_string(value.clone());
                    }
                    _ => {}
                }
            }
            JvmAttribute::Exceptions { exceptions } => {
                self.add_utf8("Exceptions".to_string());
                for exc in exceptions {
                    self.add_class(exc.clone());
                }
            }
            JvmAttribute::Signature { signature } => {
                self.add_utf8("Signature".to_string());
                self.add_utf8(signature.clone());
            }
            JvmAttribute::StackMapTable { frames } => {
                self.add_utf8("StackMapTable".to_string());
                for frame in frames {
                    match frame {
                        JvmStackMapFrame::SameLocals1StackItem { stack, .. }
                        | JvmStackMapFrame::SameLocals1StackItemExtended { stack, .. } => {
                            self.collect_verification_type_constants(stack);
                        }
                        JvmStackMapFrame::Append { locals, .. } => {
                            for vt in locals {
                                self.collect_verification_type_constants(vt);
                            }
                        }
                        JvmStackMapFrame::Full { locals, stack, .. } => {
                            for vt in locals {
                                self.collect_verification_type_constants(vt);
                            }
                            for vt in stack {
                                self.collect_verification_type_constants(vt);
                            }
                        }
                        _ => {}
                    }
                }
            }
            JvmAttribute::InnerClasses { classes } => {
                self.add_utf8("InnerClasses".to_string());
                for inner in classes {
                    self.add_class(inner.inner_class.clone());
                    if let Some(outer) = &inner.outer_class {
                        self.add_class(outer.clone());
                    }
                    if let Some(name) = &inner.inner_name {
                        self.add_utf8(name.clone());
                    }
                }
            }
            JvmAttribute::EnclosingMethod { class_name, method_name, method_descriptor } => {
                self.add_utf8("EnclosingMethod".to_string());
                self.add_class(class_name.clone());
                if let (Some(name), Some(desc)) = (method_name, method_descriptor) {
                    self.add_name_and_type(name.clone(), desc.clone());
                }
            }
            JvmAttribute::Code { attributes, exception_table, .. } => {
                self.add_utf8("Code".to_string());
                for attr in attributes {
                    self.collect_attribute_constants(attr);
                }
                for handler in exception_table {
                    if handler.catch_type_index > 0 {
                        // catch_type_index 已经在常量池中了，这里不需要额外操作
                    }
                }
            }
            JvmAttribute::LineNumberTable { .. } => {
                self.add_utf8("LineNumberTable".to_string());
            }
            JvmAttribute::LocalVariableTable { entries } => {
                self.add_utf8("LocalVariableTable".to_string());
                for entry in entries {
                    self.add_utf8(entry.name.clone());
                    self.add_utf8(entry.descriptor.clone());
                }
            }
            JvmAttribute::Unknown { name, .. } => {
                self.add_utf8(name.clone());
            }
        }
    }

    pub fn collect_verification_type_constants(&mut self, vt: &JvmVerificationType) {
        if let JvmVerificationType::Object { class_name } = vt {
            self.add_class(class_name.clone());
        }
    }

    /// 写入属性
    pub fn write_attribute(&mut self, attribute: &JvmAttribute) -> Result<()> {
        match attribute {
            JvmAttribute::SourceFile { filename } => {
                let name_idx = self.add_utf8("SourceFile".to_string());
                let file_idx = self.add_utf8(filename.clone());
                self.writer.write_u16(name_idx)?;
                self.writer.write_u32(2)?;
                self.writer.write_u16(file_idx)?;
            }
            JvmAttribute::ConstantValue { value } => {
                let name_idx = self.add_utf8("ConstantValue".to_string());
                let val_idx = match value {
                    JvmConstantPoolEntry::Integer { value } => self.add_int(*value),
                    JvmConstantPoolEntry::Float { value } => self.add_float(*value),
                    JvmConstantPoolEntry::Long { value } => self.add_long(*value),
                    JvmConstantPoolEntry::Double { value } => self.add_double(*value),
                    JvmConstantPoolEntry::String { value } => self.add_string(value.clone()),
                    _ => 0,
                };
                self.writer.write_u16(name_idx)?;
                self.writer.write_u32(2)?;
                self.writer.write_u16(val_idx)?;
            }
            JvmAttribute::Exceptions { exceptions } => {
                let name_idx = self.add_utf8("Exceptions".to_string());
                let attr_len = 2 + exceptions.len() * 2;
                self.writer.write_u16(name_idx)?;
                self.writer.write_u32(attr_len as u32)?;
                self.writer.write_u16(exceptions.len() as u16)?;
                for exc in exceptions {
                    let exc_idx = self.add_class(exc.clone());
                    self.writer.write_u16(exc_idx)?;
                }
            }
            JvmAttribute::Signature { signature } => {
                let name_idx = self.add_utf8("Signature".to_string());
                let sig_idx = self.add_utf8(signature.clone());
                self.writer.write_u16(name_idx)?;
                self.writer.write_u32(2)?;
                self.writer.write_u16(sig_idx)?;
            }
            JvmAttribute::StackMapTable { frames } => {
                let name_idx = self.add_utf8("StackMapTable".to_string());
                let mut buf = Vec::new();
                buf.extend_from_slice(&(frames.len() as u16).to_be_bytes());
                for frame in frames {
                    self.write_stack_map_frame_to_buf(frame, &mut buf)?;
                }
                self.writer.write_u16(name_idx)?;
                self.writer.write_u32(buf.len() as u32)?;
                self.writer.write_all(&buf)?;
            }
            JvmAttribute::InnerClasses { classes } => {
                let name_idx = self.add_utf8("InnerClasses".to_string());
                let attr_len = 2 + classes.len() * 8;
                self.writer.write_u16(name_idx)?;
                self.writer.write_u32(attr_len as u32)?;
                self.writer.write_u16(classes.len() as u16)?;
                for inner in classes {
                    let inner_idx = self.add_class(inner.inner_class.clone());
                    let outer_idx = if let Some(outer) = &inner.outer_class { self.add_class(outer.clone()) } else { 0 };
                    let name_idx = if let Some(name) = &inner.inner_name { self.add_utf8(name.clone()) } else { 0 };
                    self.writer.write_u16(inner_idx)?;
                    self.writer.write_u16(outer_idx)?;
                    self.writer.write_u16(name_idx)?;
                    self.writer.write_u16(inner.access_flags.to_flags())?;
                }
            }
            JvmAttribute::EnclosingMethod { class_name, method_name, method_descriptor } => {
                let name_idx = self.add_utf8("EnclosingMethod".to_string());
                let class_idx = self.add_class(class_name.clone());
                let nt_idx = if let (Some(name), Some(desc)) = (method_name, method_descriptor) {
                    self.add_name_and_type(name.clone(), desc.clone())
                }
                else {
                    0
                };
                self.writer.write_u16(name_idx)?;
                self.writer.write_u32(4)?;
                self.writer.write_u16(class_idx)?;
                self.writer.write_u16(nt_idx)?;
            }
            JvmAttribute::Unknown { name, data } => {
                let name_idx = self.add_utf8(name.clone());
                self.writer.write_u16(name_idx)?;
                self.writer.write_u32(data.len() as u32)?;
                self.writer.write_all(data)?;
            }
            _ => {
                // Code, LineNumberTable, LocalVariableTable 等需要特殊处理或暂不支持
            }
        }
        Ok(())
    }

    pub fn write_verification_type_to_buf(&mut self, vt: &JvmVerificationType, buf: &mut Vec<u8>) -> Result<()> {
        match vt {
            JvmVerificationType::Top => buf.push(0),
            JvmVerificationType::Integer => buf.push(1),
            JvmVerificationType::Float => buf.push(2),
            JvmVerificationType::Double => buf.push(3),
            JvmVerificationType::Long => buf.push(4),
            JvmVerificationType::Null => buf.push(5),
            JvmVerificationType::UninitializedThis => buf.push(6),
            JvmVerificationType::Object { class_name } => {
                buf.push(7);
                let class_idx = self.add_class(class_name.clone());
                buf.extend_from_slice(&class_idx.to_be_bytes());
            }
            JvmVerificationType::Uninitialized { offset } => {
                buf.push(8);
                buf.extend_from_slice(&offset.to_be_bytes());
            }
        }
        Ok(())
    }

    pub fn write_stack_map_frame_to_buf(&mut self, frame: &JvmStackMapFrame, buf: &mut Vec<u8>) -> Result<()> {
        match frame {
            JvmStackMapFrame::Same { offset_delta } => {
                if *offset_delta <= 63 {
                    buf.push(*offset_delta as u8);
                }
                else {
                    buf.push(251);
                    buf.extend_from_slice(&offset_delta.to_be_bytes());
                }
            }
            JvmStackMapFrame::SameLocals1StackItem { offset_delta, stack } => {
                if *offset_delta <= 63 {
                    buf.push((*offset_delta + 64) as u8);
                    self.write_verification_type_to_buf(stack, buf)?;
                }
                else {
                    buf.push(247);
                    buf.extend_from_slice(&offset_delta.to_be_bytes());
                    self.write_verification_type_to_buf(stack, buf)?;
                }
            }
            JvmStackMapFrame::SameLocals1StackItemExtended { offset_delta, stack } => {
                buf.push(247);
                buf.extend_from_slice(&offset_delta.to_be_bytes());
                self.write_verification_type_to_buf(stack, buf)?;
            }
            JvmStackMapFrame::Chop { offset_delta, k } => {
                buf.push(251 - k);
                buf.extend_from_slice(&offset_delta.to_be_bytes());
            }
            JvmStackMapFrame::SameExtended { offset_delta } => {
                buf.push(251);
                buf.extend_from_slice(&offset_delta.to_be_bytes());
            }
            JvmStackMapFrame::Append { offset_delta, locals } => {
                buf.push((251 + locals.len()) as u8);
                buf.extend_from_slice(&offset_delta.to_be_bytes());
                for vt in locals {
                    self.write_verification_type_to_buf(vt, buf)?;
                }
            }
            JvmStackMapFrame::Full { offset_delta, locals, stack } => {
                buf.push(255);
                buf.extend_from_slice(&offset_delta.to_be_bytes());
                buf.extend_from_slice(&(locals.len() as u16).to_be_bytes());
                for vt in locals {
                    self.write_verification_type_to_buf(vt, buf)?;
                }
                buf.extend_from_slice(&(stack.len() as u16).to_be_bytes());
                for vt in stack {
                    self.write_verification_type_to_buf(vt, buf)?;
                }
            }
        }
        Ok(())
    }
}

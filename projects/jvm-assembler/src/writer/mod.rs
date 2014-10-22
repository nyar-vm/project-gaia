//! JVM 字节码写入器

use crate::helpers::*;
use byteorder::{BigEndian, WriteBytesExt};
use gaia_types::*;
use std::io::{Cursor, Write};

mod jvm_writer;

/// JVM 字节码写入器
pub struct JvmWriter {
    /// 输出缓冲区
    buffer: Cursor<Vec<u8>>,
}

pub struct JvmWriter2<W: WriteBytesExt> {
    reader: BinaryAssembler<W, BigEndian>,
}

/// 便捷函数：将 JVM 类写入字节码
pub fn write_class_to_bytes(class: &JvmClass) -> Result<Vec<u8>> {
    let mut writer = JvmWriter::new();
    writer.write_class(class)?;
    Ok(writer.into_bytes())
}

/// 便捷函数：将指令列表写入字节码
pub fn write_instructions_to_bytes(instructions: &[JvmInstruction]) -> Result<Vec<u8>> {
    let mut writer = JvmWriter::new();
    for instruction in instructions {
        writer.write_instruction(instruction)?;
    }
    Ok(writer.into_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::helpers::opcodes::*;

    #[test]
    fn test_write_simple_instruction() {
        let instruction = JvmInstruction::new(NOP, vec![]);
        let result = write_instructions_to_bytes(&[instruction]).unwrap();
        assert_eq!(result, vec![0x00]);
    }

    #[test]
    fn test_write_instruction_with_operands() {
        let instruction = JvmInstruction::new(BIPUSH, vec![42]);
        let result = write_instructions_to_bytes(&[instruction]).unwrap();
        assert_eq!(result, vec![0x10, 42]);
    }

    #[test]
    fn test_write_multiple_instructions() {
        let instructions = vec![
            JvmInstruction::new(ICONST_1, vec![]),
            JvmInstruction::new(ICONST_2, vec![]),
            JvmInstruction::new(IADD, vec![]),
        ];
        let result = write_instructions_to_bytes(&instructions).unwrap();
        assert_eq!(result, vec![0x04, 0x05, 0x60]);
    }
}

use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LuacInstruction {
    Resume,
    PushNull,
    LoadName(u8),
    LoadFast(u8),
    CallFunction(u8),
    PopTop,
    ReturnValue,
    ReturnConst(u8),
    LoadConst(u8),
}

impl LuacInstruction {
    pub fn to_bytecode(&self) -> impl Iterator<Item = u32> {
        let bytecode_value = match self {
            LuacInstruction::Resume => 0x00000000,                            // Placeholder
            LuacInstruction::PushNull => 0x01000000,                          // Placeholder
            LuacInstruction::LoadName(arg) => 0x02000000 | (*arg as u32),     // Placeholder
            LuacInstruction::LoadFast(arg) => 0x03000000 | (*arg as u32),     // Placeholder
            LuacInstruction::CallFunction(arg) => 0x04000000 | (*arg as u32), // Placeholder
            LuacInstruction::PopTop => 0x05000000,                            // Placeholder
            LuacInstruction::ReturnValue => 0x06000000,                       // Placeholder
            LuacInstruction::ReturnConst(arg) => 0x07000000 | (*arg as u32),  // Placeholder
            LuacInstruction::LoadConst(arg) => 0x08000000 | (*arg as u32),    // Placeholder
        };
        vec![bytecode_value].into_iter()
    }
}

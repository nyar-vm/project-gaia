use crate::program::*;
use std::collections::{BTreeSet, HashMap};

/// StackMapTable 分析器框架
pub struct StackMapAnalyzer<'a> {
    pub _class_name: String,
    pub method: &'a JvmMethod,
    pub label_positions: &'a HashMap<String, i32>,
}

impl<'a> StackMapAnalyzer<'a> {
    pub fn new(_class_name: String, method: &'a JvmMethod, label_positions: &'a HashMap<String, i32>) -> Self {
        Self { _class_name, method, label_positions }
    }

    /// 分析方法并生成 StackMapTable 帧
    pub fn analyze(&self) -> Vec<JvmStackMapFrame> {
        let jump_targets = self.get_jump_targets();
        if jump_targets.is_empty() {
            return Vec::new();
        }

        let mut frames = Vec::new();
        let mut last_offset = 0;
        let mut first = true;

        for target_offset in jump_targets {
            let offset_delta = if first { target_offset as u16 } else { (target_offset - last_offset - 1) as u16 };

            // 基础框架实现：暂时假设为 Full 帧，以保证最大兼容性
            // 之后可以根据状态差异优化为 Same, Append, Chop 等
            frames.push(self.generate_full_frame(offset_delta));

            last_offset = target_offset;
            first = false;
        }

        frames
    }

    /// 获取所有的跳转目标偏移量
    fn get_jump_targets(&self) -> BTreeSet<i32> {
        let mut targets = BTreeSet::new();
        for inst in &self.method.instructions {
            match inst {
                JvmInstruction::Ifeq { target }
                | JvmInstruction::Ifne { target }
                | JvmInstruction::Iflt { target }
                | JvmInstruction::Ifge { target }
                | JvmInstruction::Ifgt { target }
                | JvmInstruction::Ifle { target }
                | JvmInstruction::IfIcmpeq { target }
                | JvmInstruction::IfIcmpne { target }
                | JvmInstruction::IfIcmplt { target }
                | JvmInstruction::IfIcmpge { target }
                | JvmInstruction::IfIcmpgt { target }
                | JvmInstruction::IfIcmple { target }
                | JvmInstruction::IfAcmpeq { target }
                | JvmInstruction::IfAcmpne { target }
                | JvmInstruction::Ifnull { target }
                | JvmInstruction::Ifnonnull { target }
                | JvmInstruction::Goto { target }
                | JvmInstruction::GotoW { target } => {
                    if let Some(&pos) = self.label_positions.get(target) {
                        targets.insert(pos);
                    }
                }
                JvmInstruction::Tableswitch { default, targets: switch_targets, .. } => {
                    if let Some(&pos) = self.label_positions.get(default) {
                        targets.insert(pos);
                    }
                    for target in switch_targets {
                        if let Some(&pos) = self.label_positions.get(target) {
                            targets.insert(pos);
                        }
                    }
                }
                JvmInstruction::Lookupswitch { default, pairs, .. } => {
                    if let Some(&pos) = self.label_positions.get(default) {
                        targets.insert(pos);
                    }
                    for (_, target) in pairs {
                        if let Some(&pos) = self.label_positions.get(target) {
                            targets.insert(pos);
                        }
                    }
                }
                _ => {}
            }
        }

        // 异常处理器也是跳转目标
        for handler in &self.method.exception_handlers {
            if let Some(&pos) = self.label_positions.get(&handler.handler_label) {
                targets.insert(pos);
            }
        }

        targets
    }

    /// 生成一个 Full 帧（目前作为通用方案）
    fn generate_full_frame(&self, offset_delta: u16) -> JvmStackMapFrame {
        // TODO: 真正实现局部变量和操作数栈的状态追踪
        // 目前返回一个空的 locals 和 stack，这通常不正确，但展示了框架结构
        JvmStackMapFrame::Full { offset_delta, locals: Vec::new(), stack: Vec::new() }
    }
}

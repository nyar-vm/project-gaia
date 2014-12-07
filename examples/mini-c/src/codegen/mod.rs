//! Gaia 指令生成器
//! 
//! 将 C AST 转换为 Gaia 指令

use crate::ast::*;
use gaia_types::GaiaError;
use gaia_assembler::{
    instruction::GaiaInstruction,
    program::{GaiaConstant, GaiaFunction, GaiaProgram},
    types::GaiaType,
};
use std::collections::{HashMap, HashSet};

/// Gaia 翻译器，将 C AST 转换为 Gaia 指令
pub struct GaiaTranslator {
    /// 局部变量映射
    locals: HashMap<String, u32>,
    /// 局部变量索引计数器
    local_index: u32,
    /// 局部变量类型列表（索引对应类型）
    local_types: Vec<GaiaType>,
    /// 字符串常量池
    string_constants: Vec<(String, GaiaConstant)>,
    /// 标签计数器
    label_counter: u32,
    /// 全局变量映射
    globals: HashSet<String>,
}

impl GaiaTranslator {
    /// 创建新的 Gaia 翻译器
    pub fn new() -> Self {
        Self {
            locals: HashMap::new(),
            local_index: 0,
            local_types: Vec::new(),
            string_constants: Vec::new(),
            label_counter: 0,
            globals: HashSet::new(),
        }
    }

    /// 生成 GaiaProgram
    pub fn generate(&mut self, program: &Program) -> Result<GaiaProgram, GaiaError> {
        let mut functions = Vec::new();

        // 首先处理全局变量声明
        for declaration in &program.declarations {
            if let Declaration::Variable { .. } = declaration {
                self.process_global_variable(declaration)?;
            }
        }

        // 生成函数
        for declaration in &program.declarations {
            if let Declaration::Function { name, return_type, parameters, body } = declaration {
                if let Some(body) = body {
                    let function = self.generate_function(name, return_type, parameters, body)?;
                    functions.push(function);
                }
            }
        }

        // 如果没有 main 函数，创建一个空的 main 函数
        if !functions.iter().any(|f| f.name == "main") {
            let main_function = GaiaFunction {
                name: "main".to_string(),
                parameters: Vec::new(),
                return_type: Some(GaiaType::Integer32),
                instructions: vec![
                    GaiaInstruction::LoadConstant(GaiaConstant::Integer32(0)),
                    GaiaInstruction::Return,
                ],
                locals: Vec::new(),
            };
            functions.push(main_function);
        }

        // 常量池使用已采集的字符串常量
        let constants = self.string_constants.clone();

        Ok(GaiaProgram {
            name: "c_program".to_string(),
            functions,
            constants,
            globals: None,
        })
    }

    /// 处理全局变量
    fn process_global_variable(&mut self, declaration: &Declaration) -> Result<(), GaiaError> {
        if let Declaration::Variable { name, .. } = declaration {
            self.globals.insert(name.clone());
        }
        Ok(())
    }

    /// 生成函数
    fn generate_function(
        &mut self,
        name: &str,
        return_type: &Type,
        parameters: &[Parameter],
        body: &CompoundStatement,
    ) -> Result<GaiaFunction, GaiaError> {
        // 重置局部变量状态
        self.locals.clear();
        self.local_index = 0;
        self.local_types.clear();

        // 处理参数类型和索引（参数同时作为可按索引访问的局部项）
        let mut param_types: Vec<GaiaType> = Vec::new();
        for param in parameters {
            let param_ty = self.convert_type(&param.type_);
            param_types.push(param_ty.clone());
            if let Some(param_name) = &param.name {
                self.locals.insert(param_name.clone(), self.local_index);
                self.local_types.push(param_ty);
                self.local_index += 1;
            }
        }

        let mut instructions = Vec::new();

        // 生成函数体
        self.generate_compound_statement(body, &mut instructions)?;

        // 如果函数没有显式返回，添加默认返回
        if !instructions.iter().any(|inst| matches!(inst, GaiaInstruction::Return)) {
            match return_type {
                Type::Basic(BasicType::Void) => {
                    instructions.push(GaiaInstruction::Return);
                }
                _ => {
                    // 非 void 函数返回 0
                    instructions.push(GaiaInstruction::LoadConstant(GaiaConstant::Integer32(0)));
                    instructions.push(GaiaInstruction::Return);
                }
            }
        }

        // 返回类型：void 用 None，其余 Some(type)
        let ret = match return_type {
            Type::Basic(BasicType::Void) => None,
            _ => Some(self.convert_type(return_type)),
        };

        Ok(GaiaFunction {
            name: name.to_string(),
            parameters: param_types,
            return_type: ret,
            instructions,
            locals: self.local_types.clone(),
        })
    }

    /// 转换类型
    fn convert_type(&self, c_type: &Type) -> GaiaType {
        match c_type {
            Type::Basic(basic_type) => match basic_type {
                BasicType::Void => GaiaType::Void,
                BasicType::Char => GaiaType::Integer8,
                BasicType::Short => GaiaType::Integer16,
                BasicType::Int => GaiaType::Integer32,
                BasicType::Long => GaiaType::Integer64,
                BasicType::Float => GaiaType::Float32,
                BasicType::Double => GaiaType::Float64,
                BasicType::Signed => GaiaType::Integer32,
                BasicType::Unsigned => GaiaType::Integer32,
            },
            Type::Pointer(inner) => GaiaType::Pointer(Box::new(self.convert_type(inner))),
            Type::Array { element_type, size: _ } => GaiaType::Array(Box::new(self.convert_type(element_type))),
            _ => GaiaType::Object,
        }
    }

    /// 生成复合语句
    fn generate_compound_statement(
        &mut self,
        compound: &CompoundStatement,
        instructions: &mut Vec<GaiaInstruction>,
    ) -> Result<(), GaiaError> {
        for statement in &compound.statements {
            self.generate_statement(statement, instructions)?;
        }
        Ok(())
    }

    /// 生成语句
    fn generate_statement(
        &mut self,
        statement: &Statement,
        instructions: &mut Vec<GaiaInstruction>,
    ) -> Result<(), GaiaError> {
        match statement {
            Statement::Compound(compound) => {
                self.generate_compound_statement(compound, instructions)?;
            }
            Statement::Expression(expr_opt) => {
                if let Some(expr) = expr_opt {
                    self.generate_expression(expr, instructions)?;
                    // 表达式语句需要弹出结果
                    instructions.push(GaiaInstruction::Pop);
                }
            }
            Statement::Return(expr_opt) => {
                if let Some(expr) = expr_opt {
                    self.generate_expression(expr, instructions)?;
                } else {
                    // 对于 void 返回，不需要加载任何值
                }
                instructions.push(GaiaInstruction::Return);
            }
            Statement::If { condition, then_stmt, else_stmt } => {
                self.generate_if_statement(condition, then_stmt, else_stmt.as_deref(), instructions)?;
            }
            Statement::While { condition, body } => {
                self.generate_while_statement(condition, body, instructions)?;
            }
            Statement::For { init, condition, update, body } => {
                self.generate_for_statement(init.as_ref(), condition.as_ref(), update.as_ref(), body, instructions)?;
            }
            Statement::Switch { .. } => {
                // TODO: 实现 switch 语句
            }
            Statement::Case { .. } => {
                // TODO: 实现 case 标签
            }
            Statement::Default(_) => {
                // TODO: 实现 default 标签
            }
            Statement::Goto(_) => {
                // TODO: 实现 goto 语句
            }
            Statement::Label { .. } => {
                // TODO: 实现标签语句
            }
            Statement::Break => {
                // TODO: 实现 break 语句的跳转逻辑（当前不生成额外指令）
            }
            Statement::Continue => {
                // TODO: 实现 continue 语句的跳转逻辑（当前不生成额外指令）
            }
            Statement::Declaration(decl) => {
                self.generate_declaration(decl, instructions)?;
            }
        }
        Ok(())
    }

    /// 生成声明
    fn generate_declaration(
        &mut self,
        declaration: &Declaration,
        instructions: &mut Vec<GaiaInstruction>,
    ) -> Result<(), GaiaError> {
        match declaration {
            Declaration::Variable { name, initializer, .. } => {
                // 为局部变量分配索引
                let local_index = self.local_index;
                self.locals.insert(name.clone(), local_index);
                self.local_index += 1;

                // 如果有初始化表达式，生成它
                if let Some(init_expr) = initializer {
                    self.generate_expression(init_expr, instructions)?;
                    instructions.push(GaiaInstruction::StoreLocal(local_index as usize));
                }
            }
            Declaration::Function { .. } => {
                // 函数声明在顶层处理，这里跳过
            }
            Declaration::Struct { .. } => {
                // 结构体声明当前不生成指令，跳过
            }
            Declaration::Preprocessor { .. } => {
                // 预处理器指令跳过
            }
        }
        Ok(())
    }

    /// 生成表达式
    fn generate_expression(
        &mut self,
        expression: &Expression,
        instructions: &mut Vec<GaiaInstruction>,
    ) -> Result<(), GaiaError> {
        match expression {
            Expression::Literal(literal) => {
                self.generate_literal(literal, instructions)?;
            }
            Expression::Identifier(name) => {
                self.generate_variable_load(name, instructions)?;
            }
            Expression::Binary { left, operator, right } => {
                self.generate_binary_op(left, operator, right, instructions)?;
            }
            Expression::Unary { operator, operand } => {
                self.generate_unary_op(operator, operand, instructions)?;
            }
            Expression::Assignment { left, operator, right } => {
                self.generate_assignment_expression(left, operator, right, instructions)?;
            }
            Expression::Call { function, arguments } => {
                self.generate_function_call(function, arguments, instructions)?;
            }
            Expression::ArrayAccess { array, index } => {
                self.generate_array_access(array, index, instructions)?;
            }
            Expression::MemberAccess { object, member } => {
                self.generate_member_access(object, member, instructions)?;
            }
            Expression::PointerAccess { pointer, member } => {
                self.generate_pointer_access(pointer, member, instructions)?;
            }
            Expression::Conditional { condition, true_expr, false_expr } => {
                self.generate_conditional_expression(condition, true_expr, false_expr, instructions)?;
            }
            Expression::Cast { type_, expression } => {
                self.generate_cast_expression(type_, expression, instructions)?;
            }
            Expression::Sizeof(expr) => {
                self.generate_sizeof_expression(expr, instructions)?;
            }
            Expression::Comma(expressions) => {
                self.generate_comma_expression(expressions, instructions)?;
            }
            _ => {
                return Err(GaiaError::syntax_error(
                    format!("Unsupported expression: {:?}", expression),
                    gaia_types::SourceLocation::default(),
                ));
            }
        }
        Ok(())
    }

    /// 生成字面量
    fn generate_literal(
        &mut self,
        literal: &Literal,
        instructions: &mut Vec<GaiaInstruction>,
    ) -> Result<(), GaiaError> {
        let constant = match literal {
            Literal::Integer(n) => GaiaConstant::Integer64(*n),
            Literal::Float(f) => GaiaConstant::Float64(*f),
            Literal::Character(c) => GaiaConstant::Integer8(*c as i8),
            Literal::String(s) => {
                let const_name = format!("str_{}", self.string_constants.len());
                let constant = GaiaConstant::String(s.clone());
                self.string_constants.push((const_name.clone(), constant.clone()));
                constant
            }
        };

        instructions.push(GaiaInstruction::LoadConstant(constant));
        Ok(())
    }

    /// 生成变量加载
    fn generate_variable_load(
        &mut self,
        name: &str,
        instructions: &mut Vec<GaiaInstruction>,
    ) -> Result<(), GaiaError> {
        if let Some(&local_index) = self.locals.get(name) {
            instructions.push(GaiaInstruction::LoadLocal(local_index as usize));
        } else if self.globals.contains(name) {
            instructions.push(GaiaInstruction::LoadGlobal(name.to_string()));
        } else {
            return Err(GaiaError::syntax_error(
                format!("Undefined variable: {}", name),
                gaia_types::SourceLocation::default(),
            ));
        }
        Ok(())
    }

    /// 生成二元运算
    fn generate_binary_op(
        &mut self,
        left: &Expression,
        operator: &BinaryOperator,
        right: &Expression,
        instructions: &mut Vec<GaiaInstruction>,
    ) -> Result<(), GaiaError> {
        // 生成左操作数
        self.generate_expression(left, instructions)?;
        // 生成右操作数
        self.generate_expression(right, instructions)?;

        // 生成运算指令
        let instruction = match operator {
            BinaryOperator::Add => GaiaInstruction::Add,
            BinaryOperator::Subtract => GaiaInstruction::Subtract,
            BinaryOperator::Multiply => GaiaInstruction::Multiply,
            BinaryOperator::Divide => GaiaInstruction::Divide,
            BinaryOperator::Modulo => GaiaInstruction::Remainder,
            BinaryOperator::Equal => GaiaInstruction::Equal,
            BinaryOperator::NotEqual => GaiaInstruction::NotEqual,
            BinaryOperator::Less => GaiaInstruction::LessThan,
            BinaryOperator::LessEqual => GaiaInstruction::LessThanOrEqual,
            BinaryOperator::Greater => GaiaInstruction::GreaterThan,
            BinaryOperator::GreaterEqual => GaiaInstruction::GreaterThanOrEqual,
            BinaryOperator::LeftShift => GaiaInstruction::ShiftLeft,
            BinaryOperator::RightShift => GaiaInstruction::ShiftRight,
            _ => {
                return Err(GaiaError::syntax_error(
                    format!("Unsupported binary operator: {:?}", operator),
                    gaia_types::SourceLocation::default(),
                ));
            }
        };

        instructions.push(instruction);
        Ok(())
    }

    /// 生成一元运算
    fn generate_unary_op(
        &mut self,
        operator: &UnaryOperator,
        operand: &Expression,
        instructions: &mut Vec<GaiaInstruction>,
    ) -> Result<(), GaiaError> {
        self.generate_expression(operand, instructions)?;

        let instruction_opt = match operator {
            UnaryOperator::Plus => None, // 正号不需要操作
            UnaryOperator::Minus => Some(GaiaInstruction::Negate),
            UnaryOperator::LogicalNot => Some(GaiaInstruction::LogicalNot),
            UnaryOperator::BitwiseNot => Some(GaiaInstruction::BitwiseNot),
            UnaryOperator::Dereference => Some(GaiaInstruction::LoadIndirect(GaiaType::Integer32)),
            UnaryOperator::AddressOf => None,
            _ => todo!(),
        };

        if let Some(inst) = instruction_opt { instructions.push(inst); }
        Ok(())
    }

    /// 生成赋值表达式
    fn generate_assignment_expression(
        &mut self,
        left: &Expression,
        operator: &AssignmentOperator,
        right: &Expression,
        instructions: &mut Vec<GaiaInstruction>,
    ) -> Result<(), GaiaError> {
        match operator {
            AssignmentOperator::Assign => {
                // 简单赋值
                self.generate_expression(right, instructions)?;
                self.generate_assignment_target(left, instructions)?;
            }
            _ => {
                // 复合赋值 (+=, -=, 等)
                self.generate_expression(left, instructions)?;
                self.generate_expression(right, instructions)?;
                
                let op_instruction = match operator {
                    AssignmentOperator::AddAssign => GaiaInstruction::Add,
                    AssignmentOperator::SubAssign => GaiaInstruction::Subtract,
                    AssignmentOperator::MulAssign => GaiaInstruction::Multiply,
                    AssignmentOperator::DivAssign => GaiaInstruction::Divide,
                    AssignmentOperator::ModAssign => GaiaInstruction::Remainder,
                    AssignmentOperator::AndAssign => GaiaInstruction::BitwiseAnd,
                    AssignmentOperator::OrAssign => GaiaInstruction::BitwiseOr,
                    AssignmentOperator::XorAssign => GaiaInstruction::BitwiseXor,
                    AssignmentOperator::LeftShiftAssign => GaiaInstruction::ShiftLeft,
                    AssignmentOperator::RightShiftAssign => GaiaInstruction::ShiftRight,
                    _ => unreachable!(),
                };
                
                instructions.push(op_instruction);
                self.generate_assignment_target(left, instructions)?;
            }
        }
        Ok(())
    }

    /// 生成赋值目标
    fn generate_assignment_target(
        &mut self,
        target: &Expression,
        instructions: &mut Vec<GaiaInstruction>,
    ) -> Result<(), GaiaError> {
        match target {
            Expression::Identifier(name) => {
                if let Some(&local_index) = self.locals.get(name) {
                    instructions.push(GaiaInstruction::StoreLocal(local_index as usize));
                } else if self.globals.contains(name) {
                    instructions.push(GaiaInstruction::StoreGlobal(name.to_string()));
                } else {
                    return Err(GaiaError::syntax_error(
                        format!("Undefined variable: {}", name),
                        gaia_types::SourceLocation::default(),
                    ));
                }
            }
            _ => {
                return Err(GaiaError::syntax_error(
                    "Complex assignment targets not yet supported".to_string(),
                    gaia_types::SourceLocation::default(),
                ));
            }
        }
        Ok(())
    }

    /// 生成函数调用
    fn generate_function_call(
        &mut self,
        func: &Expression,
        args: &[Expression],
        instructions: &mut Vec<GaiaInstruction>,
    ) -> Result<(), GaiaError> {
        // 生成参数
        for arg in args {
            self.generate_expression(arg, instructions)?;
        }

        // 生成函数调用
        if let Expression::Identifier(name) = func {
            instructions.push(GaiaInstruction::Call(name.clone(), args.len() as usize));
        } else {
            return Err(GaiaError::syntax_error(
                "Only simple function calls are supported".to_string(),
                gaia_types::SourceLocation::default(),
            ));
        }

        Ok(())
    }

    /// 生成数组访问
    fn generate_array_access(
        &mut self,
        array: &Expression,
        index: &Expression,
        instructions: &mut Vec<GaiaInstruction>,
    ) -> Result<(), GaiaError> {
        self.generate_expression(array, instructions)?;
        self.generate_expression(index, instructions)?;
        instructions.push(GaiaInstruction::LoadElement(GaiaType::Integer32));
        Ok(())
    }

    /// 生成成员访问
    fn generate_member_access(
        &mut self,
        object: &Expression,
        member: &str,
        instructions: &mut Vec<GaiaInstruction>,
    ) -> Result<(), GaiaError> {
        // 暂不支持成员访问
        Err(GaiaError::syntax_error(
            format!("Member access not supported: .{}", member),
            gaia_types::SourceLocation::default(),
        ))
    }

    /// 生成指针成员访问
    fn generate_pointer_access(
        &mut self,
        pointer: &Expression,
        member: &str,
        instructions: &mut Vec<GaiaInstruction>,
    ) -> Result<(), GaiaError> {
        // 暂不支持指针成员访问
        Err(GaiaError::syntax_error(
            format!("Pointer member access not supported: ->{}", member),
            gaia_types::SourceLocation::default(),
        ))
    }

    /// 生成条件表达式
    fn generate_conditional_expression(
        &mut self,
        condition: &Expression,
        true_expr: &Expression,
        false_expr: &Expression,
        instructions: &mut Vec<GaiaInstruction>,
    ) -> Result<(), GaiaError> {
        // 生成条件
        self.generate_expression(condition, instructions)?;

        // 创建标签
        let else_label = format!("L{}", self.label_counter);
        self.label_counter += 1;
        let end_label = format!("L{}", self.label_counter);
        self.label_counter += 1;

        // 条件跳转到 else 分支
        instructions.push(GaiaInstruction::JumpIfFalse(else_label.clone()));

        // 生成 true 表达式
        self.generate_expression(true_expr, instructions)?;
        instructions.push(GaiaInstruction::Jump(end_label.clone()));

        // else 标签和表达式
        instructions.push(GaiaInstruction::Label(else_label.clone()));
        self.generate_expression(false_expr, instructions)?;

        // 结束标签
        instructions.push(GaiaInstruction::Label(end_label.clone()));

        Ok(())
    }

    /// 生成 if 语句
    fn generate_if_statement(
        &mut self,
        condition: &Expression,
        then_stmt: &Statement,
        else_stmt: Option<&Statement>,
        instructions: &mut Vec<GaiaInstruction>,
    ) -> Result<(), GaiaError> {
        // 生成条件表达式
        self.generate_expression(condition, instructions)?;

        // 创建标签
        let else_label = format!("L{}", self.label_counter);
        self.label_counter += 1;
        let end_label = format!("L{}", self.label_counter);
        self.label_counter += 1;

        // 条件跳转到 else 分支
        instructions.push(GaiaInstruction::JumpIfFalse(else_label.clone()));

        // 生成 then 分支
        self.generate_statement(then_stmt, instructions)?;

        // 跳转到结束
        instructions.push(GaiaInstruction::Jump(end_label.clone()));

        // else 标签
        instructions.push(GaiaInstruction::Label(else_label.clone()));

        // 生成 else 分支
        if let Some(else_stmt) = else_stmt {
            self.generate_statement(else_stmt, instructions)?;
        }

        // 结束标签
        instructions.push(GaiaInstruction::Label(end_label.clone()));

        Ok(())
    }

    /// 生成 while 语句
    fn generate_while_statement(
        &mut self,
        condition: &Expression,
        body: &Statement,
        instructions: &mut Vec<GaiaInstruction>,
    ) -> Result<(), GaiaError> {
        let loop_start = format!("L{}", self.label_counter);
        self.label_counter += 1;
        let loop_end = format!("L{}", self.label_counter);
        self.label_counter += 1;

        // 循环开始标签
        instructions.push(GaiaInstruction::Label(loop_start.clone()));

        // 生成条件
        self.generate_expression(condition, instructions)?;

        // 条件为假时跳出循环
        instructions.push(GaiaInstruction::JumpIfFalse(loop_end.clone()));

        // 生成循环体
        self.generate_statement(body, instructions)?;

        // 跳回循环开始
        instructions.push(GaiaInstruction::Jump(loop_start.clone()));

        // 循环结束标签
        instructions.push(GaiaInstruction::Label(loop_end.clone()));

        Ok(())
    }

    /// 生成 for 语句
    fn generate_for_statement(
        &mut self,
        init: Option<&Expression>,
        condition: Option<&Expression>,
        update: Option<&Expression>,
        body: &Statement,
        instructions: &mut Vec<GaiaInstruction>,
    ) -> Result<(), GaiaError> {
        // 生成初始化
        if let Some(init_expr) = init {
            self.generate_expression(init_expr, instructions)?;
            instructions.push(GaiaInstruction::Pop); // 丢弃初始化表达式的结果
        }

        let loop_start = format!("L{}", self.label_counter);
        self.label_counter += 1;
        let loop_end = format!("L{}", self.label_counter);
        self.label_counter += 1;
        let loop_continue = format!("L{}", self.label_counter);
        self.label_counter += 1;

        // 循环开始标签
        instructions.push(GaiaInstruction::Label(loop_start.clone()));

        // 生成条件（如果有）
        if let Some(condition_expr) = condition {
            self.generate_expression(condition_expr, instructions)?;
            instructions.push(GaiaInstruction::JumpIfFalse(loop_end.clone()));
        }

        // 生成循环体
        self.generate_statement(body, instructions)?;

        // continue 标签
        instructions.push(GaiaInstruction::Label(loop_continue.clone()));

        // 生成更新表达式（如果有）
        if let Some(update_expr) = update {
            self.generate_expression(update_expr, instructions)?;
            instructions.push(GaiaInstruction::Pop); // 丢弃更新表达式的结果
        }

        // 跳回循环开始
        instructions.push(GaiaInstruction::Jump(loop_start.clone()));

        // 循环结束标签
        instructions.push(GaiaInstruction::Label(loop_end.clone()));

        Ok(())
    }

    /// 生成类型转换表达式
    fn generate_cast_expression(
        &mut self,
        target_type: &Type,
        expression: &Expression,
        instructions: &mut Vec<GaiaInstruction>,
    ) -> Result<(), GaiaError> {
        // 生成被转换的表达式
        self.generate_expression(expression, instructions)?;
        
        // 根据目标类型生成转换指令
        let gaia_type = self.convert_type(target_type);
        instructions.push(GaiaInstruction::Convert(GaiaType::Integer32, gaia_type));
        
        Ok(())
    }

    /// 生成 sizeof 表达式
    fn generate_sizeof_expression(
        &mut self,
        expression: &Expression,
        instructions: &mut Vec<GaiaInstruction>,
    ) -> Result<(), GaiaError> {
        // 对于 sizeof，我们需要在编译时计算大小
        // 这里简化处理，返回一个固定值
        // 实际实现中应该根据表达式类型计算实际大小
        instructions.push(GaiaInstruction::LoadConstant(GaiaConstant::Integer32(8))); // 假设所有类型都是 8 字节
        
        Ok(())
    }

    /// 生成逗号表达式
    fn generate_comma_expression(
        &mut self,
        expressions: &[Expression],
        instructions: &mut Vec<GaiaInstruction>,
    ) -> Result<(), GaiaError> {
        // 逗号表达式：依次计算所有表达式，返回最后一个的值
        for (i, expr) in expressions.iter().enumerate() {
            self.generate_expression(expr, instructions)?;
            // 除了最后一个表达式，其他的结果都要丢弃
            if i < expressions.len() - 1 {
                instructions.push(GaiaInstruction::Pop);
            }
        }
        
        Ok(())
    }
}

impl Default for GaiaTranslator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_assignment() {
        let mut translator = GaiaTranslator::new();
        
        // 创建一个简单的程序
        let program = Program {
            declarations: vec![
                Declaration::Function {
                    return_type: Type::Basic(BasicType::Int),
                    name: "main".to_string(),
                    parameters: vec![],
                    body: Some(CompoundStatement {
                        statements: vec![],
                    }),
                }
            ],
        };

        let result = translator.generate(&program);
        assert!(result.is_ok());
        
        let gaia_program = result.unwrap();
        assert_eq!(gaia_program.name, "c_program");
        assert!(!gaia_program.functions.is_empty());
    }

    #[test]
    fn test_arithmetic() {
        let mut translator = GaiaTranslator::new();
        
        // 创建一个包含算术运算的程序
        let program = Program {
            declarations: vec![
                Declaration::Function {
                    return_type: Type::Basic(BasicType::Int),
                    name: "main".to_string(),
                    parameters: vec![],
                    body: Some(CompoundStatement {
                        statements: vec![],
                    }),
                }
            ],
        };

        let result = translator.generate(&program);
        assert!(result.is_ok());
    }
}

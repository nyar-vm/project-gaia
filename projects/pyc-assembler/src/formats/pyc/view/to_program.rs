use crate::{
    formats::pyc::{view::PycView, PycReadConfig},
    instructions::PythonInstruction,
    program::{LocalVar, PythonCodeObject, PythonProgram, PythonVersion, Upvalue},
};
use gaia_types::{GaiaDiagnostics, GaiaError};

const HAVE_ARGUMENT: u8 = 90;

impl PycView {
    /// 将 PycView 转换为 PythonProgram。
    pub fn to_program(self, config: &PycReadConfig) -> GaiaDiagnostics<PythonProgram> {
        let mut convert = Pyc2Program { config, program: PythonProgram::default(), errors: vec![] };
        match convert.transform(self) {
            Ok(_) => GaiaDiagnostics { result: Ok(convert.program), diagnostics: convert.errors },
            Err(e) => GaiaDiagnostics { result: Err(e), diagnostics: convert.errors },
        }
    }
}

struct Pyc2Program<'config> {
    config: &'config PycReadConfig,
    program: PythonProgram,
    errors: Vec<GaiaError>,
}

impl<'config> Pyc2Program<'config> {
    pub fn transform(&mut self, view: PycView) -> Result<(), GaiaError> {
        // 动态判断 Python 版本：优先使用配置，其次用 .pyc 头部 MAGIC_NUMBER
        let version = if self.config.version != PythonVersion::Unknown {
            self.config.version
        }
        else {
            PythonVersion::from_magic(view.header.magic)
        };

        // 将魔数转换为 u32 用于指令解码
        let magic_number = u32::from_le_bytes(view.header.magic);

        // 解码字节码指令（现在字节码已经在 reader 中被解析出来了）
        let code_instructions = self.decode(&view.code_object_bytes, magic_number);

        // 构建 PythonCodeObject，使用 reader 解析的数据
        self.program.code_object = PythonCodeObject {
            source_name: view.filename.clone(),
            first_line: view.firstlineno,
            last_line: view.firstlineno, // 简化处理
            num_params: view.argcount as u8,
            is_vararg: if view.flags & 0x04 != 0 { 1 } else { 0 }, // CO_VARARGS
            max_stack_size: view.stacksize as u8,
            nested_functions: vec![],
            upvalues: view
                .freevars
                .iter()
                .enumerate()
                .map(|(_i, name)| Upvalue { in_stack: 0, idx: _i as u8, name: name.clone() })
                .collect(),
            local_vars: view
                .varnames
                .iter()
                .enumerate()
                .map(|(_i, name)| LocalVar { name: name.clone(), start_pc: 0, end_pc: 0 })
                .collect(),
            line_info: vec![], // 简化处理，不解析行号信息
            co_argcount: view.argcount as u8,
            co_nlocal: view.nlocals as u8,
            co_stacks: view.stacksize as u8,
            num_upval: view.freevars.len() as u8,
            co_code: code_instructions,
            co_consts: view.constants,
            upvalue_n: view.freevars.len() as u8,
        };

        self.program.header = view.header;
        self.program.version = version;
        Ok(())
    }

    fn decode(&self, code_bytes: &[u8], magic_number: u32) -> Vec<PythonInstruction> {
        let mut instructions = Vec::new();
        let mut i = 0;
        let mut ext_arg: u32 = 0; // 处理 EXTENDED_ARG 的累积值（按 8bit 扩展）
        while i < code_bytes.len() {
            let opcode_byte = code_bytes[i];
            let mut arg: Option<u32> = None;

            // 读取参数（1 字节），处理 EXTENDED_ARG 扩展
            if i + 1 < code_bytes.len() {
                let base = code_bytes[i + 1] as u32;
                let full = (ext_arg << 8) | base;
                if opcode_byte >= HAVE_ARGUMENT {
                    arg = Some(full);
                }
                // EXTENDED_ARG (144) 累积参数，其他指令重置
                if opcode_byte == 144 {
                    ext_arg = full;
                }
                else {
                    ext_arg = 0;
                }
            }
            // 所有指令都占用 2 字节（opcode + arg）
            i += 2;

            let python_opcode = match opcode_byte {
                // 基础指令 (0-20) - 版本相关
                0 if magic_number >= 0xa80d0d0a => PythonInstruction::CACHE, // Python 3.11+
                1 => PythonInstruction::POP_TOP,
                2 => PythonInstruction::PUSH_NULL,
                3 => PythonInstruction::INTERPRETER_EXIT,
                4 => PythonInstruction::END_FOR,
                5 => PythonInstruction::END_SEND,
                9 => PythonInstruction::NOP,
                11 => PythonInstruction::UNARY_NEGATIVE,
                12 => PythonInstruction::UNARY_NOT,
                15 => PythonInstruction::UNARY_INVERT,
                17 => PythonInstruction::RESERVED,

                // 二元操作指令 (20-30)
                25 => PythonInstruction::BINARY_SUBSCR,
                26 => PythonInstruction::BINARY_SLICE,
                27 => PythonInstruction::STORE_SLICE,

                // 匹配和异常处理指令 (30-40)
                30 => PythonInstruction::GET_LEN,
                31 => PythonInstruction::MATCH_MAPPING,
                32 => PythonInstruction::MATCH_SEQUENCE,
                33 => PythonInstruction::MATCH_KEYS,
                35 => PythonInstruction::PUSH_EXC_INFO,
                36 => PythonInstruction::CHECK_EXC_MATCH,
                37 => PythonInstruction::CHECK_EG_MATCH,

                // 异步和上下文管理指令 (49-60)
                49 => PythonInstruction::WITH_EXCEPT_START,
                50 => PythonInstruction::GET_AITER(arg.unwrap_or(0)),
                51 => PythonInstruction::GET_ANEXT(arg.unwrap_or(0)),
                52 => PythonInstruction::BEFORE_ASYNC_WITH,
                53 => PythonInstruction::BEFORE_WITH,
                54 => PythonInstruction::END_ASYNC_FOR,
                55 => PythonInstruction::CLEANUP_THROW,

                // 存储和删除指令 (60-90)
                60 => PythonInstruction::STORE_SUBSCR(arg.unwrap_or(0)),
                61 => PythonInstruction::DELETE_SUBSCR(arg.unwrap_or(0)),
                68 => PythonInstruction::GET_ITER,
                69 => PythonInstruction::GET_YIELD_FROM_ITER,
                71 => PythonInstruction::LOAD_BUILD_CLASS,
                74 => PythonInstruction::LOAD_ASSERTION_ERROR,
                75 => PythonInstruction::RETURN_GENERATOR,
                83 => PythonInstruction::RETURN_VALUE,
                85 => PythonInstruction::SETUP_ANNOTATIONS,
                87 => PythonInstruction::LOAD_LOCALS,
                89 => PythonInstruction::POP_EXCEPT,

                // 有参数的指令 (90+)
                90 => PythonInstruction::STORE_NAME(arg.unwrap_or(0)),
                91 => PythonInstruction::DELETE_NAME(arg.unwrap_or(0)),
                92 => PythonInstruction::UNPACK_SEQUENCE(arg.unwrap_or(0)),
                93 => PythonInstruction::FOR_ITER(arg.unwrap_or(0)),
                94 => PythonInstruction::UNPACK_EX(arg.unwrap_or(0)),
                95 => PythonInstruction::STORE_ATTR(arg.unwrap_or(0)),
                96 => PythonInstruction::DELETE_ATTR(arg.unwrap_or(0)),
                97 => PythonInstruction::STORE_GLOBAL(arg.unwrap_or(0)),
                98 => PythonInstruction::DELETE_GLOBAL(arg.unwrap_or(0)),
                99 => PythonInstruction::SWAP(arg.unwrap_or(0)),

                // 加载和构建指令 (100-120)
                100 => PythonInstruction::LOAD_CONST(arg.unwrap_or(0)),
                101 => PythonInstruction::LOAD_NAME(arg.unwrap_or(0)),
                102 => PythonInstruction::BUILD_TUPLE(arg.unwrap_or(0)),
                103 => PythonInstruction::BUILD_LIST(arg.unwrap_or(0)),
                104 => PythonInstruction::BUILD_SET(arg.unwrap_or(0)),
                105 => PythonInstruction::BUILD_MAP(arg.unwrap_or(0)),
                106 => PythonInstruction::LOAD_ATTR(arg.unwrap_or(0)),
                107 => PythonInstruction::COMPARE_OP(arg.unwrap_or(0)),
                108 => PythonInstruction::IMPORT_NAME(arg.unwrap_or(0)),
                109 => PythonInstruction::IMPORT_FROM(arg.unwrap_or(0)),
                110 => PythonInstruction::JUMP_FORWARD(arg.unwrap_or(0)),
                111 => PythonInstruction::JUMP_IF_FALSE_OR_POP(arg.unwrap_or(0)),
                112 => PythonInstruction::JUMP_IF_TRUE_OR_POP(arg.unwrap_or(0)),
                113 => PythonInstruction::JUMP_ABSOLUTE(arg.unwrap_or(0)),
                114 => PythonInstruction::POP_JUMP_IF_FALSE(arg.unwrap_or(0)),
                115 => PythonInstruction::POP_JUMP_IF_TRUE(arg.unwrap_or(0)),
                116 => PythonInstruction::LOAD_GLOBAL(arg.unwrap_or(0)),
                117 => PythonInstruction::IS_OP(arg.unwrap_or(0)),
                118 => PythonInstruction::CONTAINS_OP(arg.unwrap_or(0)),
                119 => PythonInstruction::RERAISE(arg.unwrap_or(0)),
                120 => PythonInstruction::COPY(arg.unwrap_or(0)),
                121 => PythonInstruction::BINARY_OP(arg.unwrap_or(0)),
                122 => PythonInstruction::SEND(arg.unwrap_or(0)),
                124 => PythonInstruction::LOAD_FAST(arg.unwrap_or(0)),
                125 => PythonInstruction::STORE_FAST(arg.unwrap_or(0)),
                126 => PythonInstruction::DELETE_FAST(arg.unwrap_or(0)),
                127 => PythonInstruction::POP_JUMP_FORWARD_IF_NOT_NONE(arg.unwrap_or(0)),
                128 => PythonInstruction::POP_JUMP_FORWARD_IF_NONE(arg.unwrap_or(0)),
                129 => PythonInstruction::RAISE_VARARGS(arg.unwrap_or(0)),
                130 => PythonInstruction::GET_AWAITABLE(arg.unwrap_or(0)),
                131 => PythonInstruction::MAKE_FUNCTION(arg.unwrap_or(0)),
                132 => PythonInstruction::BUILD_LIST(arg.unwrap_or(0)),
                133 => PythonInstruction::JUMP_BACKWARD(arg.unwrap_or(0)),
                134 => PythonInstruction::MAKE_CELL,
                135 => PythonInstruction::LOAD_CLOSURE,
                136 => PythonInstruction::LOAD_DEREF(arg.unwrap_or(0)),
                137 => PythonInstruction::STORE_DEREF(arg.unwrap_or(0)),
                138 => PythonInstruction::DELETE_DEREF(arg.unwrap_or(0)),
                139 => PythonInstruction::JUMP_BACKWARD(arg.unwrap_or(0)),
                140 => PythonInstruction::LOAD_SUPER_ATTR(arg.unwrap_or(0)),
                141 => PythonInstruction::CALL_FUNCTION_EX(arg.unwrap_or(0)),
                142 => PythonInstruction::LOAD_BUILD_CLASS,
                143 => PythonInstruction::YIELD_VALUE,
                144 => PythonInstruction::EXTENDED_ARG(arg.unwrap_or(0)),
                145 => PythonInstruction::LIST_APPEND(arg.unwrap_or(0)),
                146 => PythonInstruction::SET_ADD(arg.unwrap_or(0)),
                147 => PythonInstruction::MAP_ADD(arg.unwrap_or(0)),
                149 => PythonInstruction::COPY_FREE_VARS,
                150 => PythonInstruction::YIELD_VALUE,

                // 新版本指令 (151-180)
                151 if magic_number >= 0xa80d0d0a => PythonInstruction::RESUME, // Python 3.11+
                152 => PythonInstruction::MATCH_CLASS(arg.unwrap_or(0)),
                155 => PythonInstruction::FORMAT_VALUE(arg.unwrap_or(0)),
                156 => PythonInstruction::BUILD_CONST_KEY_MAP(arg.unwrap_or(0)),
                157 => PythonInstruction::BUILD_STRING(arg.unwrap_or(0)),
                162 => PythonInstruction::LIST_EXTEND(arg.unwrap_or(0)),
                163 => PythonInstruction::SET_UPDATE(arg.unwrap_or(0)),
                164 => PythonInstruction::DICT_MERGE(arg.unwrap_or(0)),
                165 => PythonInstruction::DICT_UPDATE(arg.unwrap_or(0)),

                // 调用指令 (171)
                171 => PythonInstruction::CALL(arg.unwrap_or(0)),
                172 => PythonInstruction::KW_NAMES(arg.unwrap_or(0)),
                173 => PythonInstruction::CALL_INTRINSIC_1(arg.unwrap_or(0)),
                174 => PythonInstruction::CALL_INTRINSIC_2(arg.unwrap_or(0)),
                175 => PythonInstruction::LOAD_FROM_DICT_OR_DEREF(arg.unwrap_or(0)),
                176 => PythonInstruction::LOAD_FROM_DICT_OR_DEREF(arg.unwrap_or(0)),

                // 仪器化指令 (237-255) - 主要在Python 3.12+
                237 if magic_number >= 0xcb0d0d0a => PythonInstruction::INSTRUMENTED_LOAD_SUPER_ATTR,
                238 if magic_number >= 0xcb0d0d0a => PythonInstruction::INSTRUMENTED_POP_JUMP_IF_NONE,
                239 if magic_number >= 0xcb0d0d0a => PythonInstruction::INSTRUMENTED_POP_JUMP_IF_NOT_NONE,
                240 if magic_number >= 0xcb0d0d0a => PythonInstruction::INSTRUMENTED_RESUME,
                241 if magic_number >= 0xcb0d0d0a => PythonInstruction::INSTRUMENTED_CALL,
                242 if magic_number >= 0xcb0d0d0a => PythonInstruction::INSTRUMENTED_RETURN_VALUE,
                243 if magic_number >= 0xcb0d0d0a => PythonInstruction::INSTRUMENTED_YIELD_VALUE,
                244 if magic_number >= 0xcb0d0d0a => PythonInstruction::INSTRUMENTED_CALL_FUNCTION_EX,
                245 if magic_number >= 0xcb0d0d0a => PythonInstruction::INSTRUMENTED_JUMP_FORWARD,
                246 if magic_number >= 0xcb0d0d0a => PythonInstruction::INSTRUMENTED_JUMP_BACKWARD,
                247 if magic_number >= 0xcb0d0d0a => PythonInstruction::INSTRUMENTED_RETURN_CONST,
                248 if magic_number >= 0xcb0d0d0a => PythonInstruction::INSTRUMENTED_FOR_ITER,
                249 if magic_number >= 0xcb0d0d0a => PythonInstruction::INSTRUMENTED_POP_JUMP_IF_FALSE,
                250 if magic_number >= 0xcb0d0d0a => PythonInstruction::INSTRUMENTED_POP_JUMP_IF_TRUE,
                251 if magic_number >= 0xcb0d0d0a => PythonInstruction::INSTRUMENTED_END_FOR,
                252 if magic_number >= 0xcb0d0d0a => PythonInstruction::INSTRUMENTED_END_SEND,
                253 if magic_number >= 0xcb0d0d0a => PythonInstruction::INSTRUMENTED_INSTRUCTION,
                254 if magic_number >= 0xcb0d0d0a => PythonInstruction::INSTRUMENTED_LINE,

                _ => PythonInstruction::UNKNOWN(opcode_byte, arg),
            };
            instructions.push(python_opcode);
        }
        instructions
    }
}

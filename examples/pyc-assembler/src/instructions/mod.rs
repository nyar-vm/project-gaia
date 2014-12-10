#![doc = include_str!("readme.md")]
use serde::{Deserialize, Serialize};

/// 表示 Python 字节码指令。
#[allow(non_camel_case_types)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PythonInstruction {
    /// 缓存操作码。
    CACHE,
    /// 二进制模运算。
    BINARY_MODULO,
    /// 就地模运算。
    INPLACE_MODULO,
    /// 弹出栈顶元素。
    POP_TOP,
    /// 推送 None 到栈顶。
    PUSH_NULL,
    /// 解释器退出。
    INTERPRETER_EXIT,
    /// 结束 for 循环。
    END_FOR,
    /// 结束 send 操作。
    END_SEND,
    /// 无操作。
    NOP,
    /// 一元负号。
    UNARY_NEGATIVE,
    /// 逻辑非。
    UNARY_NOT,
    /// 按位取反。
    UNARY_INVERT,
    /// 保留操作码。
    RESERVED,
    /// 二进制切片。
    BINARY_SLICE,
    /// 获取长度。
    GET_LEN,
    /// 匹配映射。
    MATCH_MAPPING,
    /// 匹配序列。
    MATCH_SEQUENCE,
    /// 匹配键。
    MATCH_KEYS,
    /// 推送异常信息。
    PUSH_EXC_INFO,
    /// 检查异常匹配。
    CHECK_EXC_MATCH,
    /// 检查异常组匹配。
    CHECK_EG_MATCH,
    /// 带有异常处理的 with 语句开始。
    WITH_EXCEPT_START,
    /// 异步 with 语句之前。
    BEFORE_ASYNC_WITH,
    /// with 语句之前。
    BEFORE_WITH,
    /// 结束异步 for 循环。
    END_ASYNC_FOR,
    /// 清理并抛出异常。
    CLEANUP_THROW,
    /// 获取迭代器。
    GET_ITER,
    /// 获取 yield from 迭代器。
    GET_YIELD_FROM_ITER,
    /// 加载 build_class。
    LOAD_BUILD_CLASS,
    /// 加载 AssertionError。
    LOAD_ASSERTION_ERROR,
    /// 返回生成器。
    RETURN_GENERATOR,
    /// 设置注解。
    SETUP_ANNOTATIONS,
    /// 加载局部变量。
    LOAD_LOCALS,
    /// 弹出异常。
    POP_EXCEPT,
    /// 创建 cell 对象。
    MAKE_CELL,
    /// 加载闭包。
    LOAD_CLOSURE,
    /// 复制自由变量。
    COPY_FREE_VARS,
    /// yield 值。
    YIELD_VALUE,
    /// 恢复执行。
    RESUME,
    /// 带有工具的 for 循环结束。
    INSTRUMENTED_END_FOR,
    /// 带有工具的 send 结束。
    INSTRUMENTED_END_SEND,
    /// 带有工具的指令。
    INSTRUMENTED_INSTRUCTION,
    /// 带有工具的行。
    INSTRUMENTED_LINE,
    /// 弹出块。
    POP_BLOCK,
    /// 带有工具的如果为假则跳转。
    INSTRUMENTED_POP_JUMP_IF_FALSE,
    /// 带有工具的如果为真则跳转。
    INSTRUMENTED_POP_JUMP_IF_TRUE,
    /// 带有工具的向前跳转。
    INSTRUMENTED_JUMP_FORWARD,
    /// 带有工具的向后跳转。
    INSTRUMENTED_JUMP_BACKWARD,
    /// 带有工具的返回值。
    INSTRUMENTED_RETURN_VALUE,
    /// 带有工具的 yield 值。
    INSTRUMENTED_YIELD_VALUE,
    /// 带有工具的异步 for 循环结束。
    INSTRUMENTED_END_ASYNC_FOR,
    /// 带有工具的设置 finally 块。
    INSTRUMENTED_SETUP_FINALLY,
    /// 带有工具的恢复执行。
    INSTRUMENTED_RESUME,
    /// 带有工具的删除下标。
    INSTRUMENTED_DELETE_SUBSCR,
    /// 带有工具的存储下标。
    INSTRUMENTED_STORE_SUBSCR,
    /// 带有工具的调用。
    INSTRUMENTED_CALL,
    /// 带有工具的扩展参数。
    INSTRUMENTED_EXTENDED_ARG,
    /// 带有工具的列表追加。
    INSTRUMENTED_LIST_APPEND,
    /// 带有工具的集合添加。
    INSTRUMENTED_SET_ADD,
    /// 带有工具的映射添加。
    INSTRUMENTED_MAP_ADD,
    /// 带有工具的 for 迭代器。
    INSTRUMENTED_FOR_ITER,
    /// 带有工具的加载 super 属性。
    INSTRUMENTED_LOAD_SUPER_ATTR,
    /// 带有工具的如果为 None 则弹出并跳转。
    INSTRUMENTED_POP_JUMP_IF_NONE,
    /// 带有工具的如果不为 None 则弹出并跳转。
    INSTRUMENTED_POP_JUMP_IF_NOT_NONE,
    /// 带有工具的调用函数。
    INSTRUMENTED_CALL_FUNCTION_EX,
    /// 带有工具的返回常量。
    INSTRUMENTED_RETURN_CONST,
    /// 旋转栈顶两个元素。
    ROT_TWO,
    /// 旋转栈顶三个元素。
    ROT_THREE,
    /// 复制栈顶元素。
    DUP_TOP,
    /// 复制栈顶两个元素。
    DUP_TOP_TWO,
    /// 旋转栈顶四个元素。
    ROT_FOUR,
    /// 一元正号。
    UNARY_POSITIVE,
    /// 弹出 finally 块。
    POP_FINALLY,
    /// 返回值。
    RETURN_VALUE,
    /// 设置清理。
    SETUP_CLEANUP(u32),
    /// 设置 with 语句。
    SETUP_WITH(u32),
    /// 结束异步 with 语句。
    END_ASYNC_WITH,
    /// with 语句清理开始。
    WITH_CLEANUP_START,
    /// with 语句清理完成。
    WITH_CLEANUP_FINISH,
    /// 异步 with 语句清理开始。
    ASYNC_WITH_CLEANUP_START,
    /// 异步 with 语句清理完成。
    ASYNC_WITH_CLEANUP_FINISH,
    /// 生成器开始。
    GEN_START,
    /// 获取可等待协程。
    GET_AWAITABLE_CORO,
    /// 获取异步迭代器协程。
    GET_AITER_CORO,
    /// 获取异步 next 协程。
    GET_ANEXT_CORO,
    /// 结束异步 for 协程。
    END_ASYNC_FOR_CORO,
    /// 发送值。
    SEND(u32),
    /// 弹出栈帧。
    POP_FRAME,
    /// 返回常量。
    RETURN_CONST(u32),
    /// 设置异步 with 语句。
    SETUP_ASYNC_WITH(u32),
    /// 就地加法。
    INPLACE_ADD(u32),
    /// 就地减法。
    INPLACE_SUBTRACT(u32),
    /// 就地乘法。
    INPLACE_MULTIPLY(u32),
    /// 就地真除法。
    INPLACE_TRUE_DIVIDE(u32),
    /// 存储下标。
    STORE_SUBSCR(u32),
    /// 删除下标。
    DELETE_SUBSCR(u32),
    /// 存储映射。
    STORE_MAP(u32),
    /// 调用函数（扩展）。
    CALL_FUNCTION_EX(u32),
    /// 格式化值。
    FORMAT_VALUE(u32),
    /// 如果为假则弹出并跳转。
    POP_JUMP_IF_FALSE(u32),
    /// 如果为真则弹出并跳转。
    POP_JUMP_IF_TRUE(u32),
    /// 如果为 None 则弹出并跳转。
    POP_JUMP_IF_NONE(u32),
    /// 如果不为 None 则弹出并跳转。
    POP_JUMP_IF_NOT_NONE(u32),
    /// 如果为假则跳转或弹出。
    JUMP_IF_FALSE_OR_POP(u32),
    /// 如果为真则跳转或弹出。
    JUMP_IF_TRUE_OR_POP(u32),
    /// 向前跳转。
    JUMP_FORWARD(u32),
    /// 向后跳转。
    JUMP_BACKWARD(u32),
    /// 加载常量。
    LOAD_CONST(u32),
    /// 加载名称。
    LOAD_NAME(u32),
    /// 存储名称。
    STORE_NAME(u32),
    /// 删除名称。
    DELETE_NAME(u32),
    /// 加载快速变量。
    LOAD_FAST(u32),
    /// 存储快速变量。
    STORE_FAST(u32),
    /// 删除快速变量。
    DELETE_FAST(u32),
    /// 加载全局变量。
    LOAD_GLOBAL(u32),
    /// 存储全局变量。
    STORE_GLOBAL(u32),
    /// 删除全局变量。
    DELETE_GLOBAL(u32),
    /// 加载属性。
    LOAD_ATTR(u32),
    /// 存储属性。
    STORE_ATTR(u32),
    /// 删除属性。
    DELETE_ATTR(u32),
    /// 比较操作。
    COMPARE_OP(u32),
    /// 二进制操作。
    BINARY_OP(u32),
    /// 导入名称。
    IMPORT_NAME(u32),
    /// 从模块导入。
    IMPORT_FROM(u32),
    /// for 循环迭代器。
    FOR_ITER(u32),
    /// 获取可等待对象。
    GET_AWAITABLE(u32),
    /// 获取异步迭代器。
    GET_AITER(u32),
    /// 获取异步 next。
    GET_ANEXT(u32),
    /// 如果异常不匹配则跳转。
    JUMP_IF_NOT_EXC_MATCH(u32),
    /// 设置 finally 块。
    SETUP_FINALLY(u32),
    /// 设置 except 块。
    SETUP_EXCEPT(u32),
    /// 设置循环块。
    SETUP_LOOP(u32),
    /// 扩展参数。
    EXTENDED_ARG(u32),
    /// 加载解引用变量。
    LOAD_DEREF(u32),
    /// 存储解引用变量。
    STORE_DEREF(u32),
    /// 删除解引用变量。
    DELETE_DEREF(u32),
    /// 加载类解引用变量。
    LOAD_CLASSDEREF(u32),
    /// 复制局部变量。
    COPY_LOCAL(u32),
    /// 加载 super 属性。
    LOAD_SUPER_ATTR(u32),
    /// 创建函数。
    MAKE_FUNCTION(u32),
    /// 调用函数。
    CALL_FUNCTION(u32),
    /// 调用带关键字参数的函数。
    CALL_FUNCTION_KW(u32),
    /// 加载方法。
    LOAD_METHOD(u32),
    /// 调用方法。
    CALL_METHOD(u32),
    /// 构建元组。
    BUILD_TUPLE(u32),
    /// 构建列表。
    BUILD_LIST(u32),
    /// 构建集合。
    BUILD_SET(u32),
    /// 构建字典。
    BUILD_MAP(u32),
    /// 构建常量键字典。
    BUILD_CONST_KEY_MAP(u32),
    /// 构建字符串。
    BUILD_STRING(u32),
    /// 列表追加。
    LIST_APPEND(u32),
    /// 集合添加。
    SET_ADD(u32),
    /// 映射添加。
    MAP_ADD(u32),
    /// 列表扩展。
    LIST_EXTEND(u32),
    /// 集合更新。
    SET_UPDATE(u32),
    /// 字典更新。
    DICT_UPDATE(u32),
    /// 字典合并。
    DICT_MERGE(u32),
    /// 匹配类。
    MATCH_CLASS(u32),
    /// 复制不带键的字典。
    COPY_DICT_WITHOUT_KEYS(u32),
    /// yield from。
    YIELD_FROM(u32),
    /// 调用 finally。
    CALL_FINALLY(u32),
    /// 调用内置函数 1。
    CALL_INTRINSIC_1(u32),
    /// 调用内置函数 2。
    CALL_INTRINSIC_2(u32),
    /// 关键字参数名称。
    KW_NAMES(u32),
    /// 跳转。
    JUMP(u32),
    /// 无中断跳转。
    JUMP_NO_INTERRUPT(u32),
    /// 是操作。
    IS_OP(u32),
    /// 包含操作。
    CONTAINS_OP(u32),
    /// 复制。
    COPY(u32),
    /// 交换。
    SWAP(u32),
    /// 解包序列。
    UNPACK_SEQUENCE(u32),
    /// 解包扩展。
    UNPACK_EX(u32),
    /// 调用。
    CALL(u32),
    /// 关键字调用。
    CALL_KW(u32),
    /// 从字典或解引用加载。
    LOAD_FROM_DICT_OR_DEREF(u32),
    /// 加载 super 方法。
    LOAD_SUPER_METHOD(u32),
    /// 加载零参数 super 方法。
    LOAD_ZERO_SUPER_METHOD(u32),
    /// 存储快速变量（可能为 None）。
    STORE_FAST_MAYBE_NULL(u32),
    /// 重新抛出异常。
    RERAISE(u32),
    /// 二进制下标操作。
    BINARY_SUBSCR,
    /// 存储切片。
    STORE_SLICE,
    /// 绝对跳转指令，包含跳转目标地址。
    JUMP_ABSOLUTE(u32),
    /// 如果不为 None 则向前弹出并跳转。
    POP_JUMP_FORWARD_IF_NOT_NONE(u32),
    /// 如果为 None 则向前弹出并跳转。
    POP_JUMP_FORWARD_IF_NONE(u32),
    /// 抛出可变参数异常。
    RAISE_VARARGS(u32),
    /// 解析失败的未知的指令，包含操作码和可能的参数。
    UNKNOWN(u8, Option<u32>),
}

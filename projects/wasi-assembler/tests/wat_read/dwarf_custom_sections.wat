;; DWARF 调试信息和自定义字段测试文件
;; 包含 DWARF 调试信息的自定义段和其他自定义字段

(component $dwarf-example
  ;; 自定义段：外部调试信息引用
  (@custom "external_debug_info" "debug/dwarf-example.wasm")
  
  ;; 自定义段：源映射 URL
  (@custom "sourceMappingURL" "debug/dwarf-example.map")
  
  ;; 自定义段：构建信息
  (@custom "build_info" 
    "version=1.0.0\n"
    "compiler=wasi-wat-compiler\n"
    "build_date=2024-01-15T10:30:00Z\n"
    "optimization_level=2\n"
    "target=wasm32-wasi"
  )
  
  ;; 自定义段：许可证信息
  (@custom "license" 
    "MIT License\n"
    "Copyright (c) 2024 WASI WAT Project\n"
    "Permission is hereby granted..."
  )
  
  ;; 自定义段：元数据
  (@custom "metadata" 
    "author=WASI WAT Team\n"
    "description=Example component with DWARF debug info\n"
    "keywords=wasm,wasi,component-model,dwarf\n"
    "repository=https://github.com/example/wasi-wat"
  )
  
  ;; 自定义段：性能分析信息
  (@custom "profiling_info"
    "enable_profiling=true\n"
    "sample_rate=1000\n"
    "profile_memory=true\n"
    "profile_cpu=true"
  )
  
  ;; 导入 WASI 接口
  (import "wasi:filesystem/types@0.2.0" (instance $fs-types
    (export "descriptor" (type (sub resource)))
    (export "error-code" (type (variant
      (case "access" (tuple))
      (case "invalid" (tuple))
      (case "io" (tuple))
    )))
  ))
  
  ;; 定义带有调试信息的类型
  (type $debug-record (record
    (field "file-name" string)      ;; 源文件名
    (field "line-number" u32)       ;; 行号
    (field "column-number" u32)     ;; 列号
    (field "function-name" string)  ;; 函数名
    (field "local-variables" (list (record
      (field "name" string)
      (field "type" string)
      (field "value" string)
    )))
  ))
  
  ;; 定义调试事件类型
  (type $debug-event (variant
    (case "breakpoint" $debug-record)
    (case "step" $debug-record)
    (case "exception" (record
      (field "location" $debug-record)
      (field "message" string)
      (field "stack-trace" (list $debug-record))
    ))
    (case "function-entry" $debug-record)
    (case "function-exit" (record
      (field "location" $debug-record)
      (field "return-value" (option string))
    ))
  ))
  
  ;; 核心模块：包含调试信息的数学运算
  (core module $math-with-debug
    ;; 自定义段：DWARF 调试信息段
    (@custom ".debug_info" 
      ;; 这里应该包含实际的 DWARF 调试信息二进制数据
      ;; 为了演示，我们使用文本描述
      "DWARF Debug Info for math-with-debug module\n"
      "Compilation Unit: math-with-debug.wat\n"
      "Producer: wasi-wat-compiler v1.0.0\n"
      "Language: WebAssembly Text Format\n"
    )
    
    (@custom ".debug_line"
      ;; 行号信息
      "Line Number Program for math-with-debug\n"
      "File: math-with-debug.wat\n"
      "Line mappings for functions and instructions\n"
    )
    
    (@custom ".debug_abbrev"
      ;; 缩写表
      "DWARF Abbreviation Table\n"
      "Abbreviations for debug info entries\n"
    )
    
    (@custom ".debug_str"
      ;; 字符串表
      "DWARF String Table\n"
      "Strings used in debug information\n"
    )
    
    ;; 内存定义
    (memory (export "memory") 1)
    
    ;; 全局变量：调试计数器
    (global $debug-counter (export "debug-counter") (mut i32) (i32.const 0))
    
    ;; 函数：带调试信息的加法
    (func $add-with-debug (export "add") (param $a i32) (param $b i32) (result i32)
      ;; 增加调试计数器
      global.get $debug-counter
      i32.const 1
      i32.add
      global.set $debug-counter
      
      ;; 执行加法运算
      local.get $a
      local.get $b
      i32.add
      
      ;; 这里可以插入调试断点
      ;; 在实际的 DWARF 实现中，这些位置会被映射到源代码行
    )
    
    ;; 函数：带调试信息的乘法
    (func $multiply-with-debug (export "multiply") (param $a i32) (param $b i32) (result i32)
      ;; 增加调试计数器
      global.get $debug-counter
      i32.const 1
      i32.add
      global.set $debug-counter
      
      ;; 执行乘法运算
      local.get $a
      local.get $b
      i32.mul
    )
    
    ;; 函数：带错误处理的除法
    (func $divide-with-debug (export "divide") (param $a i32) (param $b i32) (result i32)
      ;; 增加调试计数器
      global.get $debug-counter
      i32.const 1
      i32.add
      global.set $debug-counter
      
      ;; 检查除零错误
      local.get $b
      i32.const 0
      i32.eq
      if
        ;; 在调试器中，这里会触发异常断点
        unreachable
      end
      
      ;; 执行除法运算
      local.get $a
      local.get $b
      i32.div_s
    )
    
    ;; 函数：获取调试信息
    (func $get-debug-info (export "get-debug-info") (result i32)
      global.get $debug-counter
    )
  )
  
  ;; 实例化数学模块
  (core instance $math-instance (instantiate $math-with-debug))
  
  ;; 组件函数：创建调试记录
  (func $create-debug-record 
    (param "file" string)
    (param "line" u32)
    (param "column" u32)
    (param "function" string)
    (result $debug-record)
    
    ;; 创建调试记录
    (record
      (field "file-name" (local.get 0))
      (field "line-number" (local.get 1))
      (field "column-number" (local.get 2))
      (field "function-name" (local.get 3))
      (field "local-variables" (list))
    )
  )
  
  ;; 组件函数：处理调试事件
  (func $handle-debug-event
    (param "event" $debug-event)
    (result string)
    
    ;; 根据事件类型处理
    (match (local.get 0)
      (case $breakpoint (record) => 
        (string.concat "Breakpoint hit at " 
          (struct.get $debug-record 0 (local.get 0))
          ":" 
          (i32.to_string (struct.get $debug-record 1 (local.get 0)))
        )
      )
      (case $step (record) => 
        (string.concat "Step at " 
          (struct.get $debug-record 3 (local.get 0))
        )
      )
      (case $exception (record) => 
        (string.concat "Exception: " 
          (struct.get 1 (local.get 0))
        )
      )
      (case $function-entry (record) => 
        (string.concat "Entering function " 
          (struct.get $debug-record 3 (local.get 0))
        )
      )
      (case $function-exit (record) => 
        (string.concat "Exiting function " 
          (struct.get $debug-record 3 (struct.get 0 (local.get 0)))
        )
      )
    )
  )
  
  ;; 组件函数：启用/禁用调试
  (func $set-debug-enabled (param "enabled" bool))
  
  ;; 组件函数：获取调试统计信息
  (func $get-debug-stats (result (record
    (field "total-calls" u32)
    (field "breakpoints-hit" u32)
    (field "exceptions-thrown" u32)
  )))
  
  ;; 导出调试相关函数
  (export "create-debug-record" (func $create-debug-record))
  (export "handle-debug-event" (func $handle-debug-event))
  (export "set-debug-enabled" (func $set-debug-enabled))
  (export "get-debug-stats" (func $get-debug-stats))
  
  ;; 导出类型
  (export "debug-record" (type $debug-record))
  (export "debug-event" (type $debug-event))
  
  ;; 导出核心模块的数学函数
  (export "add" (core func $math-instance "add"))
  (export "multiply" (core func $math-instance "multiply"))
  (export "divide" (core func $math-instance "divide"))
  (export "get-debug-info" (core func $math-instance "get-debug-info"))
)
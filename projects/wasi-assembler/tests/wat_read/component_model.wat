;; Component Model WAT 测试文件
;; 包含复杂的组件定义、类型系统和实例化

(component $my-component
  ;; 导入外部组件
  (import "wasi:filesystem/types@0.2.0" (instance $filesystem-types
    (export "descriptor" (type (sub resource)))
    (export "error-code" (type (variant
      (case "access" (tuple))
      (case "would-block" (tuple))
      (case "already" (tuple))
      (case "bad-descriptor" (tuple))
      (case "busy" (tuple))
      (case "deadlock" (tuple))
      (case "quota" (tuple))
      (case "exist" (tuple))
      (case "file-too-large" (tuple))
      (case "illegal-byte-sequence" (tuple))
      (case "in-progress" (tuple))
      (case "interrupted" (tuple))
      (case "invalid" (tuple))
      (case "io" (tuple))
      (case "is-directory" (tuple))
      (case "loop" (tuple))
      (case "too-many-links" (tuple))
      (case "message-size" (tuple))
      (case "name-too-long" (tuple))
      (case "no-device" (tuple))
      (case "no-entry" (tuple))
      (case "no-lock" (tuple))
      (case "insufficient-memory" (tuple))
      (case "insufficient-space" (tuple))
      (case "not-directory" (tuple))
      (case "not-empty" (tuple))
      (case "not-recoverable" (tuple))
      (case "unsupported" (tuple))
      (case "no-tty" (tuple))
      (case "no-such-device" (tuple))
      (case "overflow" (tuple))
      (case "not-permitted" (tuple))
      (case "pipe" (tuple))
      (case "read-only" (tuple))
      (case "invalid-seek" (tuple))
      (case "text-file-busy" (tuple))
      (case "cross-device" (tuple))
    )))
  ))

  ;; 定义自定义类型
  (type $my-record (record
    (field "name" string)
    (field "age" u32)
    (field "active" bool)
  ))

  (type $my-variant (variant
    (case "success" $my-record)
    (case "error" string)
  ))

  (type $my-resource (resource
    (method "get-name" (func (result string)))
    (method "set-name" (func (param "name" string)))
    (static "create" (func (param "name" string) (result (own $my-resource))))
  ))

  ;; 定义函数类型
  (type $process-data (func
    (param "input" $my-record)
    (param "options" (option (record
      (field "timeout" u64)
      (field "retry-count" u32)
    )))
    (result $my-variant)
  ))

  ;; 核心模块定义
  (core module $utils
    (memory (export "memory") 1)
    
    (func $add (export "add") (param i32 i32) (result i32)
      local.get 0
      local.get 1
      i32.add
    )
    
    (func $multiply (export "multiply") (param i32 i32) (result i32)
      local.get 0
      local.get 1
      i32.mul
    )
    
    (global $counter (export "counter") (mut i32) (i32.const 0))
    
    (func $increment-counter (export "increment-counter")
      global.get $counter
      i32.const 1
      i32.add
      global.set $counter
    )
  )

  ;; 实例化核心模块
  (core instance $utils-instance (instantiate $utils))

  ;; 定义组件函数
  (func $process (type $process-data)
    ;; 函数实现将在这里
  )

  ;; 导出函数
  (export "process" (func $process))
  
  ;; 导出类型
  (export "my-record" (type $my-record))
  (export "my-variant" (type $my-variant))
  (export "my-resource" (type $my-resource))
)
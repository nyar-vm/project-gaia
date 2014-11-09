(module
  ;; 简单的 a + b 函数，用于 DWARF 调试测试
  (func $add (param $a i32) (param $b i32) (result i32)
    local.get $a
    local.get $b
    i32.add
  )
  
  ;; 导出函数供外部调用
  (export "add" (func $add))
  
  ;; 自定义段：源映射信息
  (@custom "sourceMappingURL" "simple_add.wat")
  
  ;; 自定义段：名称信息
  (@custom "name" 
    (func $add "add")
  )
)
;; 核心 WebAssembly 模块测试文件
;; 包含内存管理、表操作、全局变量和复杂函数

(module $core-test
  ;; 导入外部函数
  (import "env" "print" (func $print (param i32)))
  (import "env" "memory" (memory 1))
  
  ;; 定义内存
  (memory $local-memory 2 4)
  
  ;; 定义表
  (table $function-table 10 funcref)
  
  ;; 定义全局变量
  (global $heap-pointer (mut i32) (i32.const 1024))
  (global $max-memory (export "max-memory") i32 (i32.const 65536))
  
  ;; 数据段
  (data (i32.const 0) "Hello, WebAssembly!")
  (data $message (i32.const 32) "Error occurred")
  
  ;; 元素段
  (elem (i32.const 0) $fibonacci $factorial $gcd)
  
  ;; 辅助函数：分配内存
  (func $malloc (param $size i32) (result i32)
    (local $current-ptr i32)
    
    ;; 获取当前堆指针
    global.get $heap-pointer
    local.set $current-ptr
    
    ;; 更新堆指针
    global.get $heap-pointer
    local.get $size
    i32.add
    global.set $heap-pointer
    
    ;; 返回分配的地址
    local.get $current-ptr
  )
  
  ;; 斐波那契数列
  (func $fibonacci (export "fibonacci") (param $n i32) (result i32)
    (local $a i32)
    (local $b i32)
    (local $temp i32)
    (local $i i32)
    
    ;; 处理边界情况
    local.get $n
    i32.const 2
    i32.lt_s
    if (result i32)
      local.get $n
    else
      ;; 初始化
      i32.const 0
      local.set $a
      i32.const 1
      local.set $b
      i32.const 2
      local.set $i
      
      ;; 循环计算
      loop $fib-loop
        local.get $i
        local.get $n
        i32.le_s
        if
          ;; temp = a + b
          local.get $a
          local.get $b
          i32.add
          local.set $temp
          
          ;; a = b
          local.get $b
          local.set $a
          
          ;; b = temp
          local.get $temp
          local.set $b
          
          ;; i++
          local.get $i
          i32.const 1
          i32.add
          local.set $i
          
          br $fib-loop
        end
      end
      
      local.get $b
    end
  )
  
  ;; 阶乘函数
  (func $factorial (export "factorial") (param $n i32) (result i32)
    local.get $n
    i32.const 1
    i32.le_s
    if (result i32)
      i32.const 1
    else
      local.get $n
      local.get $n
      i32.const 1
      i32.sub
      call $factorial
      i32.mul
    end
  )
  
  ;; 最大公约数
  (func $gcd (export "gcd") (param $a i32) (param $b i32) (result i32)
    local.get $b
    i32.const 0
    i32.eq
    if (result i32)
      local.get $a
    else
      local.get $b
      local.get $a
      local.get $b
      i32.rem_s
      call $gcd
    end
  )
  
  ;; 字符串长度计算
  (func $strlen (export "strlen") (param $str i32) (result i32)
    (local $len i32)
    
    i32.const 0
    local.set $len
    
    loop $strlen-loop
      local.get $str
      local.get $len
      i32.add
      i32.load8_u
      i32.const 0
      i32.ne
      if
        local.get $len
        i32.const 1
        i32.add
        local.set $len
        br $strlen-loop
      end
    end
    
    local.get $len
  )
  
  ;; 内存复制
  (func $memcpy (export "memcpy") (param $dest i32) (param $src i32) (param $size i32)
    (local $i i32)
    
    i32.const 0
    local.set $i
    
    loop $copy-loop
      local.get $i
      local.get $size
      i32.lt_u
      if
        local.get $dest
        local.get $i
        i32.add
        local.get $src
        local.get $i
        i32.add
        i32.load8_u
        i32.store8
        
        local.get $i
        i32.const 1
        i32.add
        local.set $i
        
        br $copy-loop
      end
    end
  )
  
  ;; 启动函数
  (func $start
    ;; 初始化一些数据
    i32.const 100
    call $fibonacci
    drop
    
    i32.const 5
    call $factorial
    drop
  )
  
  (start $start)
  
  ;; 导出内存和函数
  (export "memory" (memory $local-memory))
  (export "malloc" (func $malloc))
)
;; 高级组件模型测试文件
;; 包含复杂的类型系统、资源管理、异步操作和错误处理

(component $advanced-example
  ;; 自定义段：组件元数据
  (@custom "component_metadata"
    "name=advanced-example\n"
    "version=2.1.0\n"
    "api_version=0.2.0\n"
    "features=async,resources,streaming,error-handling\n"
    "min_wasm_version=2.0"
  )
  
  ;; 导入 WASI 接口
  (import "wasi:io/streams@0.2.0" (instance $streams
    (export "input-stream" (type (sub resource)))
    (export "output-stream" (type (sub resource)))
    (export "pollable" (type (sub resource)))
    (export "stream-error" (type (variant
      (case "last-operation-failed" (tuple))
      (case "closed" (tuple))
    )))
  ))
  
  (import "wasi:filesystem/types@0.2.0" (instance $fs-types
    (export "descriptor" (type (sub resource)))
    (export "directory-entry-stream" (type (sub resource)))
    (export "error-code" (type (variant
      (case "access" (tuple))
      (case "exist" (tuple))
      (case "not-directory" (tuple))
      (case "not-found" (tuple))
      (case "io" (tuple))
      (case "invalid" (tuple))
    )))
  ))
  
  (import "wasi:clocks/wall-clock@0.2.0" (instance $wall-clock
    (export "datetime" (type (record
      (field "seconds" u64)
      (field "nanoseconds" u32)
    )))
    (export "now" (func (result $datetime)))
  ))
  
  ;; 复杂类型定义
  
  ;; 泛型结果类型
  (type $result (func (param "ok" (type)) (param "err" (type)) (result (variant
    (case "ok" (type 0))
    (case "error" (type 1))
  ))))
  
  ;; 异步操作状态
  (type $async-status (variant
    (case "pending" (tuple))
    (case "completed" (tuple))
    (case "failed" string)
    (case "cancelled" (tuple))
  ))
  
  ;; 流处理配置
  (type $stream-config (record
    (field "buffer-size" u32)
    (field "timeout-ms" u32)
    (field "retry-count" u32)
    (field "compression" (option (variant
      (case "gzip" (tuple))
      (case "brotli" (tuple))
      (case "lz4" (tuple))
    )))
  ))
  
  ;; 文件元数据
  (type $file-metadata (record
    (field "size" u64)
    (field "created" $datetime)
    (field "modified" $datetime)
    (field "accessed" $datetime)
    (field "permissions" (record
      (field "readable" bool)
      (field "writable" bool)
      (field "executable" bool)
    ))
    (field "mime-type" (option string))
    (field "checksum" (option (record
      (field "algorithm" (variant
        (case "md5" (tuple))
        (case "sha1" (tuple))
        (case "sha256" (tuple))
        (case "sha512" (tuple))
      ))
      (field "value" (list u8))
    )))
  ))
  
  ;; 数据库连接配置
  (type $db-config (record
    (field "host" string)
    (field "port" u16)
    (field "database" string)
    (field "username" string)
    (field "password" string)
    (field "ssl" bool)
    (field "pool-size" u32)
    (field "timeout-seconds" u32)
  ))
  
  ;; SQL 查询结果
  (type $query-result (record
    (field "columns" (list string))
    (field "rows" (list (list (variant
      (case "null" (tuple))
      (case "bool" bool)
      (case "int" s64)
      (case "float" f64)
      (case "string" string)
      (case "bytes" (list u8))
    ))))
    (field "affected-rows" u64)
  ))
  
  ;; HTTP 请求/响应类型
  (type $http-method (variant
    (case "get" (tuple))
    (case "post" (tuple))
    (case "put" (tuple))
    (case "delete" (tuple))
    (case "patch" (tuple))
    (case "head" (tuple))
    (case "options" (tuple))
  ))
  
  (type $http-headers (list (tuple string string)))
  
  (type $http-request (record
    (field "method" $http-method)
    (field "url" string)
    (field "headers" $http-headers)
    (field "body" (option (list u8)))
  ))
  
  (type $http-response (record
    (field "status" u16)
    (field "headers" $http-headers)
    (field "body" (list u8))
  ))
  
  ;; 资源类型定义
  (type $file-handle (resource
    (method "read" (param "count" u32) (result (list u8)))
    (method "write" (param "data" (list u8)) (result u32))
    (method "seek" (param "offset" s64) (param "whence" (variant
      (case "start" (tuple))
      (case "current" (tuple))
      (case "end" (tuple))
    )) (result u64))
    (method "flush" (result (result unit string)))
    (method "metadata" (result $file-metadata))
    (method "close" (result (result unit string)))
  ))
  
  (type $database-connection (resource
    (method "execute" (param "query" string) (param "params" (list string)) 
      (result (result $query-result string)))
    (method "prepare" (param "query" string) 
      (result (result $prepared-statement string)))
    (method "begin-transaction" (result (result $transaction string)))
    (method "close" (result (result unit string)))
  ))
  
  (type $prepared-statement (resource
    (method "execute" (param "params" (list string)) 
      (result (result $query-result string)))
    (method "close" (result (result unit string)))
  ))
  
  (type $transaction (resource
    (method "commit" (result (result unit string)))
    (method "rollback" (result (result unit string)))
  ))
  
  (type $http-client (resource
    (method "request" (param "req" $http-request) 
      (result (result $http-response string)))
    (method "request-async" (param "req" $http-request) 
      (result $async-operation))
  ))
  
  (type $async-operation (resource
    (method "status" (result $async-status))
    (method "result" (result (option (result $http-response string))))
    (method "cancel" (result bool))
    (method "wait" (param "timeout-ms" u32) (result bool))
  ))
  
  ;; 核心模块：高级数据处理
  (core module $data-processor
    ;; 内存和表定义
    (memory (export "memory") 10 100)
    (table (export "table") 100 funcref)
    
    ;; 全局变量
    (global $heap-ptr (export "heap-ptr") (mut i32) (i32.const 65536))
    (global $error-code (export "error-code") (mut i32) (i32.const 0))
    
    ;; 数据段：错误消息
    (data (i32.const 1024) "OutOfMemory\00")
    (data (i32.const 1040) "InvalidInput\00")
    (data (i32.const 1056) "ProcessingError\00")
    (data (i32.const 1072) "NetworkError\00")
    
    ;; 内存管理函数
    (func $malloc (export "malloc") (param $size i32) (result i32)
      (local $ptr i32)
      global.get $heap-ptr
      local.set $ptr
      
      ;; 检查内存边界
      local.get $ptr
      local.get $size
      i32.add
      memory.size
      i32.const 65536
      i32.mul
      i32.gt_u
      if
        ;; 尝试增长内存
        local.get $size
        i32.const 65536
        i32.div_u
        i32.const 1
        i32.add
        memory.grow
        i32.const -1
        i32.eq
        if
          ;; 内存分配失败
          global.set $error-code (i32.const 1)
          i32.const 0
          return
        end
      end
      
      ;; 更新堆指针
      local.get $ptr
      local.get $size
      i32.add
      global.set $heap-ptr
      
      local.get $ptr
    )
    
    (func $free (export "free") (param $ptr i32))
    
    ;; 字符串处理函数
    (func $strlen (export "strlen") (param $str i32) (result i32)
      (local $len i32)
      loop $loop
        local.get $str
        local.get $len
        i32.add
        i32.load8_u
        i32.eqz
        if
          local.get $len
          return
        end
        local.get $len
        i32.const 1
        i32.add
        local.set $len
        br $loop
      end
    )
    
    (func $strcmp (export "strcmp") (param $str1 i32) (param $str2 i32) (result i32)
      (local $i i32)
      loop $loop
        local.get $str1
        local.get $i
        i32.add
        i32.load8_u
        local.get $str2
        local.get $i
        i32.add
        i32.load8_u
        i32.ne
        if
          local.get $str1
          local.get $i
          i32.add
          i32.load8_u
          local.get $str2
          local.get $i
          i32.add
          i32.load8_u
          i32.sub
          return
        end
        
        local.get $str1
        local.get $i
        i32.add
        i32.load8_u
        i32.eqz
        if
          i32.const 0
          return
        end
        
        local.get $i
        i32.const 1
        i32.add
        local.set $i
        br $loop
      end
    )
    
    ;; 数据压缩函数（简化版 LZ4）
    (func $compress-lz4 (export "compress-lz4") 
      (param $input i32) (param $input-len i32) 
      (param $output i32) (param $output-len i32) 
      (result i32)
      ;; 简化的压缩实现
      local.get $input-len
    )
    
    ;; 数据解压函数
    (func $decompress-lz4 (export "decompress-lz4")
      (param $input i32) (param $input-len i32)
      (param $output i32) (param $output-len i32)
      (result i32)
      ;; 简化的解压实现
      local.get $input-len
    )
    
    ;; 哈希函数（SHA-256 简化版）
    (func $hash-sha256 (export "hash-sha256")
      (param $data i32) (param $len i32) (param $hash i32)
      ;; 简化的 SHA-256 实现
    )
    
    ;; JSON 解析函数
    (func $parse-json (export "parse-json")
      (param $json i32) (param $len i32) (param $result i32)
      (result i32)
      ;; 简化的 JSON 解析实现
      i32.const 1
    )
    
    ;; 错误处理函数
    (func $get-error-message (export "get-error-message") (param $code i32) (result i32)
      local.get $code
      i32.const 1
      i32.eq
      if
        i32.const 1024
        return
      end
      
      local.get $code
      i32.const 2
      i32.eq
      if
        i32.const 1040
        return
      end
      
      local.get $code
      i32.const 3
      i32.eq
      if
        i32.const 1056
        return
      end
      
      i32.const 1072
    )
  )
  
  ;; 实例化核心模块
  (core instance $processor (instantiate $data-processor))
  
  ;; 组件函数：文件操作
  (func $open-file 
    (param "path" string) 
    (param "mode" (variant
      (case "read" (tuple))
      (case "write" (tuple))
      (case "append" (tuple))
      (case "read-write" (tuple))
    ))
    (result (result (own $file-handle) string))
    
    ;; 实现文件打开逻辑
    (ok (new $file-handle))
  )
  
  (func $create-directory
    (param "path" string)
    (param "recursive" bool)
    (result (result unit string))
    
    ;; 实现目录创建逻辑
    (ok)
  )
  
  (func $list-directory
    (param "path" string)
    (result (result (list (record
      (field "name" string)
      (field "type" (variant
        (case "file" (tuple))
        (case "directory" (tuple))
        (case "symlink" (tuple))
      ))
      (field "metadata" $file-metadata)
    )) string))
    
    ;; 实现目录列表逻辑
    (ok (list))
  )
  
  ;; 组件函数：数据库操作
  (func $connect-database
    (param "config" $db-config)
    (result (result (own $database-connection) string))
    
    ;; 实现数据库连接逻辑
    (ok (new $database-connection))
  )
  
  ;; 组件函数：HTTP 客户端
  (func $create-http-client
    (param "config" (record
      (field "timeout-seconds" u32)
      (field "max-redirects" u32)
      (field "user-agent" string)
    ))
    (result (own $http-client))
    
    ;; 创建 HTTP 客户端
    (new $http-client)
  )
  
  ;; 组件函数：流处理
  (func $create-stream-processor
    (param "config" $stream-config)
    (result (result (record
      (field "process" (func 
        (param "input" (borrow $input-stream))
        (param "output" (borrow $output-stream))
        (result (result unit string))
      ))
      (field "close" (func (result unit)))
    ) string))
    
    ;; 创建流处理器
    (ok (record
      (field "process" (func))
      (field "close" (func))
    ))
  )
  
  ;; 组件函数：异步任务管理
  (func $spawn-task
    (param "task" (func (result (result string string))))
    (result (own $async-operation))
    
    ;; 创建异步任务
    (new $async-operation)
  )
  
  (func $join-all
    (param "operations" (list (borrow $async-operation)))
    (param "timeout-ms" u32)
    (result (list (option (result string string))))
    
    ;; 等待所有异步操作完成
    (list)
  )
  
  ;; 组件函数：错误处理和日志
  (func $log-error
    (param "level" (variant
      (case "debug" (tuple))
      (case "info" (tuple))
      (case "warn" (tuple))
      (case "error" (tuple))
      (case "fatal" (tuple))
    ))
    (param "message" string)
    (param "context" (option (record
      (field "file" string)
      (field "line" u32)
      (field "function" string)
      (field "extra" (list (tuple string string)))
    )))
  )
  
  (func $create-error-handler
    (result (func 
      (param "error" string)
      (param "context" (option string))
      (result (variant
        (case "retry" (tuple))
        (case "abort" (tuple))
        (case "ignore" (tuple))
      ))
    ))
    
    ;; 返回错误处理函数
    (func)
  )
  
  ;; 导出所有功能
  (export "open-file" (func $open-file))
  (export "create-directory" (func $create-directory))
  (export "list-directory" (func $list-directory))
  (export "connect-database" (func $connect-database))
  (export "create-http-client" (func $create-http-client))
  (export "create-stream-processor" (func $create-stream-processor))
  (export "spawn-task" (func $spawn-task))
  (export "join-all" (func $join-all))
  (export "log-error" (func $log-error))
  (export "create-error-handler" (func $create-error-handler))
  
  ;; 导出类型
  (export "file-handle" (type $file-handle))
  (export "database-connection" (type $database-connection))
  (export "http-client" (type $http-client))
  (export "async-operation" (type $async-operation))
  (export "file-metadata" (type $file-metadata))
  (export "query-result" (type $query-result))
  (export "http-request" (type $http-request))
  (export "http-response" (type $http-response))
  (export "stream-config" (type $stream-config))
  (export "async-status" (type $async-status))
)
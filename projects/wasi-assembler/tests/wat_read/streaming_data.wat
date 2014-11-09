;; 流数据处理测试文件
;; 展示 WASI 流接口的使用和复杂的数据流处理

(component $streaming-data-processor
  ;; 自定义段：流处理配置
  (@custom "stream_config"
    "buffer_size=8192\n"
    "chunk_size=1024\n"
    "max_concurrent_streams=10\n"
    "compression_enabled=true\n"
    "encryption_enabled=false"
  )
  
  ;; 导入 WASI 流接口
  (import "wasi:io/streams@0.2.0" (instance $streams
    (export "input-stream" (type (sub resource)))
    (export "output-stream" (type (sub resource)))
    (export "pollable" (type (sub resource)))
    (export "stream-error" (type (variant
      (case "last-operation-failed" (tuple))
      (case "closed" (tuple))
    )))
    (export "read" (func 
      (param "this" (borrow $input-stream))
      (param "len" u64)
      (result (result (list u8) $stream-error))
    ))
    (export "write" (func
      (param "this" (borrow $output-stream))
      (param "contents" (list u8))
      (result (result u64 $stream-error))
    ))
    (export "flush" (func
      (param "this" (borrow $output-stream))
      (result (result unit $stream-error))
    ))
    (export "subscribe" (func
      (param "this" (borrow $input-stream))
      (result (own $pollable))
    ))
  ))
  
  (import "wasi:io/poll@0.2.0" (instance $poll
    (export "pollable" (type (sub resource)))
    (export "poll" (func
      (param "in" (list (borrow $pollable)))
      (result (list u32))
    ))
  ))
  
  ;; 流处理相关类型定义
  
  ;; 数据块类型
  (type $data-chunk (record
    (field "id" u64)
    (field "sequence" u32)
    (field "data" (list u8))
    (field "checksum" u32)
    (field "compressed" bool)
    (field "final" bool)
  ))
  
  ;; 流元数据
  (type $stream-metadata (record
    (field "content-type" string)
    (field "content-length" (option u64))
    (field "encoding" (option (variant
      (case "gzip" (tuple))
      (case "deflate" (tuple))
      (case "brotli" (tuple))
    )))
    (field "created-at" u64)
    (field "source" string)
    (field "tags" (list string))
  ))
  
  ;; 流统计信息
  (type $stream-stats (record
    (field "bytes-read" u64)
    (field "bytes-written" u64)
    (field "chunks-processed" u32)
    (field "errors-count" u32)
    (field "processing-time-ms" u64)
    (field "throughput-bps" f64)
  ))
  
  ;; 流处理器配置
  (type $processor-config (record
    (field "buffer-size" u32)
    (field "chunk-size" u32)
    (field "parallel-workers" u32)
    (field "enable-compression" bool)
    (field "compression-level" u32)
    (field "enable-checksum" bool)
    (field "timeout-ms" u32)
  ))
  
  ;; 流过滤器类型
  (type $stream-filter (variant
    (case "identity" (tuple))
    (case "uppercase" (tuple))
    (case "lowercase" (tuple))
    (case "base64-encode" (tuple))
    (case "base64-decode" (tuple))
    (case "json-pretty" (tuple))
    (case "json-minify" (tuple))
    (case "xml-format" (tuple))
    (case "csv-to-json" (tuple))
    (case "regex-replace" (record
      (field "pattern" string)
      (field "replacement" string)
    ))
    (case "custom" (func 
      (param "input" (list u8)) 
      (result (result (list u8) string))
    ))
  ))
  
  ;; 流处理管道
  (type $processing-pipeline (record
    (field "name" string)
    (field "filters" (list $stream-filter))
    (field "parallel" bool)
    (field "error-handling" (variant
      (case "stop-on-error" (tuple))
      (case "skip-errors" (tuple))
      (case "retry" u32)
    ))
  ))
  
  ;; 资源类型：流处理器
  (type $stream-processor (resource
    (method "process-chunk" 
      (param "chunk" $data-chunk)
      (result (result $data-chunk string))
    )
    (method "set-pipeline" 
      (param "pipeline" $processing-pipeline)
      (result (result unit string))
    )
    (method "get-stats" (result $stream-stats))
    (method "reset-stats")
    (method "close" (result (result unit string)))
  ))
  
  ;; 资源类型：流复制器
  (type $stream-multiplexer (resource
    (method "add-output" 
      (param "stream" (borrow $output-stream))
      (param "filter" (option $stream-filter))
      (result u32)
    )
    (method "remove-output" (param "id" u32) (result bool))
    (method "process" 
      (param "input" (borrow $input-stream))
      (result (result unit string))
    )
    (method "close")
  ))
  
  ;; 资源类型：流缓冲区
  (type $stream-buffer (resource
    (method "write" (param "data" (list u8)) (result (result u32 string)))
    (method "read" (param "len" u32) (result (result (list u8) string)))
    (method "peek" (param "len" u32) (result (result (list u8) string)))
    (method "available" (result u32))
    (method "capacity" (result u32))
    (method "clear")
    (method "close")
  ))
  
  ;; 核心模块：流数据处理引擎
  (core module $stream-engine
    ;; 内存定义
    (memory (export "memory") 20 200)
    
    ;; 全局变量
    (global $buffer-pool-ptr (export "buffer-pool-ptr") (mut i32) (i32.const 65536))
    (global $active-streams (export "active-streams") (mut i32) (i32.const 0))
    (global $total-bytes-processed (export "total-bytes-processed") (mut i64) (i64.const 0))
    
    ;; 数据段：查找表和常量
    (data (i32.const 1024) "STREAM_PROCESSOR_V1.0\00")
    (data (i32.const 1048) "application/octet-stream\00")
    (data (i32.const 1073) "text/plain\00")
    (data (i32.const 1084) "application/json\00")
    (data (i32.const 1101) "text/csv\00")
    
    ;; CRC32 查找表
    (data (i32.const 2048) 
      "\00\00\00\00\96\30\07\77\2c\61\0e\ee\ba\51\09\99"
      "\19\c4\6d\07\8f\f4\6a\70\35\a5\63\e9\a3\95\64\9e"
      ;; ... 完整的 CRC32 表（简化显示）
    )
    
    ;; 内存管理函数
    (func $alloc-buffer (export "alloc-buffer") (param $size i32) (result i32)
      (local $ptr i32)
      global.get $buffer-pool-ptr
      local.set $ptr
      
      ;; 对齐到 8 字节边界
      local.get $size
      i32.const 7
      i32.add
      i32.const -8
      i32.and
      local.set $size
      
      ;; 更新池指针
      local.get $ptr
      local.get $size
      i32.add
      global.set $buffer-pool-ptr
      
      local.get $ptr
    )
    
    (func $free-buffer (export "free-buffer") (param $ptr i32) (param $size i32))
    
    ;; CRC32 校验和计算
    (func $crc32 (export "crc32") (param $data i32) (param $len i32) (result u32)
      (local $crc u32)
      (local $i i32)
      (local $byte u32)
      (local $table-index u32)
      
      i32.const 0xffffffff
      local.set $crc
      
      loop $loop
        local.get $i
        local.get $len
        i32.ge_u
        if
          local.get $crc
          i32.const 0xffffffff
          i32.xor
          return
        end
        
        ;; 获取字节
        local.get $data
        local.get $i
        i32.add
        i32.load8_u
        local.set $byte
        
        ;; 计算表索引
        local.get $crc
        local.get $byte
        i32.xor
        i32.const 0xff
        i32.and
        local.set $table-index
        
        ;; 查表并更新 CRC
        i32.const 2048
        local.get $table-index
        i32.const 4
        i32.mul
        i32.add
        i32.load
        local.get $crc
        i32.const 8
        i32.shr_u
        i32.xor
        local.set $crc
        
        local.get $i
        i32.const 1
        i32.add
        local.set $i
        br $loop
      end
    )
    
    ;; 数据压缩（简化的 LZ77）
    (func $compress-data (export "compress-data")
      (param $input i32) (param $input-len i32)
      (param $output i32) (param $output-capacity i32)
      (result i32)
      
      ;; 简化的压缩实现
      ;; 实际实现会包含滑动窗口和匹配查找
      local.get $input-len
      local.get $output-capacity
      i32.min
      local.tee $copy-len
      
      ;; 复制数据（简化版）
      local.get $output
      local.get $input
      local.get $copy-len
      memory.copy
      
      local.get $copy-len
    )
    
    ;; 数据解压
    (func $decompress-data (export "decompress-data")
      (param $input i32) (param $input-len i32)
      (param $output i32) (param $output-capacity i32)
      (result i32)
      
      ;; 简化的解压实现
      local.get $input-len
      local.get $output-capacity
      i32.min
      local.tee $copy-len
      
      local.get $output
      local.get $input
      local.get $copy-len
      memory.copy
      
      local.get $copy-len
    )
    
    ;; Base64 编码
    (func $base64-encode (export "base64-encode")
      (param $input i32) (param $input-len i32)
      (param $output i32) (param $output-capacity i32)
      (result i32)
      
      ;; Base64 编码实现
      ;; 简化版本，实际需要完整的编码表
      local.get $input-len
      i32.const 4
      i32.mul
      i32.const 3
      i32.div_u
      local.get $output-capacity
      i32.min
    )
    
    ;; Base64 解码
    (func $base64-decode (export "base64-decode")
      (param $input i32) (param $input-len i32)
      (param $output i32) (param $output-capacity i32)
      (result i32)
      
      ;; Base64 解码实现
      local.get $input-len
      i32.const 3
      i32.mul
      i32.const 4
      i32.div_u
      local.get $output-capacity
      i32.min
    )
    
    ;; JSON 格式化
    (func $json-pretty-print (export "json-pretty-print")
      (param $input i32) (param $input-len i32)
      (param $output i32) (param $output-capacity i32)
      (result i32)
      
      ;; JSON 美化实现（简化版）
      local.get $input-len
      local.get $output-capacity
      i32.min
      local.tee $copy-len
      
      local.get $output
      local.get $input
      local.get $copy-len
      memory.copy
      
      local.get $copy-len
    )
    
    ;; 流统计更新
    (func $update-stats (export "update-stats")
      (param $bytes-processed i32)
      
      global.get $total-bytes-processed
      local.get $bytes-processed
      i64.extend_i32_u
      i64.add
      global.set $total-bytes-processed
    )
    
    ;; 获取统计信息
    (func $get-total-bytes (export "get-total-bytes") (result i64)
      global.get $total-bytes-processed
    )
    
    (func $get-active-streams (export "get-active-streams") (result i32)
      global.get $active-streams
    )
    
    (func $increment-active-streams (export "increment-active-streams")
      global.get $active-streams
      i32.const 1
      i32.add
      global.set $active-streams
    )
    
    (func $decrement-active-streams (export "decrement-active-streams")
      global.get $active-streams
      i32.const 1
      i32.sub
      i32.const 0
      i32.max
      global.set $active-streams
    )
  )
  
  ;; 实例化流处理引擎
  (core instance $engine (instantiate $stream-engine))
  
  ;; 组件函数：创建流处理器
  (func $create-stream-processor
    (param "config" $processor-config)
    (result (own $stream-processor))
    
    ;; 创建新的流处理器实例
    (new $stream-processor)
  )
  
  ;; 组件函数：创建流复制器
  (func $create-stream-multiplexer
    (result (own $stream-multiplexer))
    
    (new $stream-multiplexer)
  )
  
  ;; 组件函数：创建流缓冲区
  (func $create-stream-buffer
    (param "capacity" u32)
    (result (own $stream-buffer))
    
    (new $stream-buffer)
  )
  
  ;; 组件函数：流数据复制
  (func $copy-stream
    (param "source" (borrow $input-stream))
    (param "destination" (borrow $output-stream))
    (param "buffer-size" u32)
    (result (result $stream-stats string))
    
    ;; 实现流复制逻辑
    (ok (record
      (field "bytes-read" (u64.const 0))
      (field "bytes-written" (u64.const 0))
      (field "chunks-processed" (u32.const 0))
      (field "errors-count" (u32.const 0))
      (field "processing-time-ms" (u64.const 0))
      (field "throughput-bps" (f64.const 0.0))
    ))
  )
  
  ;; 组件函数：流数据转换
  (func $transform-stream
    (param "source" (borrow $input-stream))
    (param "destination" (borrow $output-stream))
    (param "pipeline" $processing-pipeline)
    (result (result $stream-stats string))
    
    ;; 实现流转换逻辑
    (ok (record
      (field "bytes-read" (u64.const 0))
      (field "bytes-written" (u64.const 0))
      (field "chunks-processed" (u32.const 0))
      (field "errors-count" (u32.const 0))
      (field "processing-time-ms" (u64.const 0))
      (field "throughput-bps" (f64.const 0.0))
    ))
  )
  
  ;; 组件函数：并行流处理
  (func $parallel-process-streams
    (param "sources" (list (borrow $input-stream)))
    (param "destinations" (list (borrow $output-stream)))
    (param "pipeline" $processing-pipeline)
    (param "max-workers" u32)
    (result (result (list $stream-stats) string))
    
    ;; 实现并行流处理逻辑
    (ok (list))
  )
  
  ;; 组件函数：流数据验证
  (func $validate-stream
    (param "stream" (borrow $input-stream))
    (param "expected-checksum" (option u32))
    (param "expected-size" (option u64))
    (result (result (record
      (field "valid" bool)
      (field "actual-checksum" u32)
      (field "actual-size" u64)
      (field "errors" (list string))
    ) string))
    
    ;; 实现流验证逻辑
    (ok (record
      (field "valid" (bool.const true))
      (field "actual-checksum" (u32.const 0))
      (field "actual-size" (u64.const 0))
      (field "errors" (list))
    ))
  )
  
  ;; 组件函数：创建预定义管道
  (func $create-text-processing-pipeline
    (result $processing-pipeline)
    
    (record
      (field "name" "text-processing")
      (field "filters" (list
        (uppercase)
        (regex-replace (record
          (field "pattern" "\\s+")
          (field "replacement" " ")
        ))
      ))
      (field "parallel" (bool.const false))
      (field "error-handling" (skip-errors))
    )
  )
  
  (func $create-json-processing-pipeline
    (result $processing-pipeline)
    
    (record
      (field "name" "json-processing")
      (field "filters" (list
        (json-pretty)
      ))
      (field "parallel" (bool.const false))
      (field "error-handling" (stop-on-error))
    )
  )
  
  (func $create-encoding-pipeline
    (param "encode" bool)
    (result $processing-pipeline)
    
    (record
      (field "name" "encoding-pipeline")
      (field "filters" (list
        (if (local.get 0)
          (then (base64-encode))
          (else (base64-decode))
        )
      ))
      (field "parallel" (bool.const true))
      (field "error-handling" (retry (u32.const 3)))
    )
  )
  
  ;; 导出所有功能
  (export "create-stream-processor" (func $create-stream-processor))
  (export "create-stream-multiplexer" (func $create-stream-multiplexer))
  (export "create-stream-buffer" (func $create-stream-buffer))
  (export "copy-stream" (func $copy-stream))
  (export "transform-stream" (func $transform-stream))
  (export "parallel-process-streams" (func $parallel-process-streams))
  (export "validate-stream" (func $validate-stream))
  (export "create-text-processing-pipeline" (func $create-text-processing-pipeline))
  (export "create-json-processing-pipeline" (func $create-json-processing-pipeline))
  (export "create-encoding-pipeline" (func $create-encoding-pipeline))
  
  ;; 导出类型
  (export "stream-processor" (type $stream-processor))
  (export "stream-multiplexer" (type $stream-multiplexer))
  (export "stream-buffer" (type $stream-buffer))
  (export "data-chunk" (type $data-chunk))
  (export "stream-metadata" (type $stream-metadata))
  (export "stream-stats" (type $stream-stats))
  (export "processor-config" (type $processor-config))
  (export "stream-filter" (type $stream-filter))
  (export "processing-pipeline" (type $processing-pipeline))
  
  ;; 导出核心模块功能
  (export "crc32" (core func $engine "crc32"))
  (export "compress-data" (core func $engine "compress-data"))
  (export "decompress-data" (core func $engine "decompress-data"))
  (export "base64-encode" (core func $engine "base64-encode"))
  (export "base64-decode" (core func $engine "base64-decode"))
  (export "get-total-bytes-processed" (core func $engine "get-total-bytes"))
  (export "get-active-streams-count" (core func $engine "get-active-streams"))
)
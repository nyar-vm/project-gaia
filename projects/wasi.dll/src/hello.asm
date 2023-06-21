section .data
    libraryName db 'wasi.dll', 0
    functionName db 'hello_world', 0

section .text
    global _start

_start:
    ; 加载动态链接库
    mov rax, 0x2   ; LoadLibraryA 的函数编号
    mov rbx, libraryName
    call qword [rip + LoadLibrary]
    mov rdi, rax   ; 将返回的句柄存储在 rdi 中

    ; 获取函数地址
    mov rax, 0x0   ; GetProcAddress 的函数编号
    mov rcx, rdi   ; 将句柄作为第一个参数传递
    mov rdx, functionName
    call qword [rip + GetProcAddress]
    mov rbx, rax   ; 将函数地址存储在 rbx 中

    ; 调用函数
    ; 将参数传递给函数
    mov rdi, bytes ; 将数据指针存储在 rdi 中

    ; 调用函数
    call rbx

    ; 清理资源
    mov rax, 0x0   ; FreeLibrary 的函数编号
    mov rcx, rdi   ; 将句柄作为第一个参数传递
    call qword [rip + FreeLibrary]

    ; 退出程序
    mov eax, 0x1   ; 退出的系统调用号
    xor edi, edi   ; 返回值为 0
    syscall

section .data
    LoadLibrary db 'LoadLibraryA', 0
    GetProcAddress db 'GetProcAddress', 0
    FreeLibrary db 'FreeLibrary', 0
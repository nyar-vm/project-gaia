use gaia_jit::JitMemory;

#[test]
fn test_jit_alloc() {
    let jit = JitMemory::new(4096).unwrap();
    assert!(jit.capacity() >= 4096);
}

#[test]
fn test_jit_execute_identity() {
    #[cfg(target_arch = "x86_64")]
    let code = if cfg!(windows) {
        vec![0x89, 0xc8, 0xc3] // mov eax, ecx; ret
    }
    else {
        vec![0x89, 0xf8, 0xc3] // mov eax, edi; ret
    };

    #[cfg(target_arch = "aarch64")]
    let code = vec![0xd6, 0x5f, 0x03, 0xc0]; // ret

    #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
    let code: Vec<u8> = vec![];

    if code.is_empty() {
        return;
    }

    let mut jit = JitMemory::new(4096).unwrap();
    jit.write(&code).unwrap();
    jit.make_executable().unwrap();

    unsafe {
        let f: unsafe extern "C" fn(i32) -> i32 = jit.as_fn();
        assert_eq!(f(42), 42);
        assert_eq!(f(123), 123);
    }
}

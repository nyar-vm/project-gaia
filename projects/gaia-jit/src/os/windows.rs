use std::{io, ptr::NonNull};
use windows_sys::Win32::System::{
    Diagnostics::Debug::FlushInstructionCache,
    Memory::*,
    SystemInformation::{GetSystemInfo, SYSTEM_INFO},
    Threading::GetCurrentProcess,
};

pub fn get_page_size() -> usize {
    unsafe {
        let mut system_info: SYSTEM_INFO = std::mem::zeroed();
        GetSystemInfo(&mut system_info);
        system_info.dwPageSize as usize
    }
}

pub fn alloc_rw(size: usize) -> io::Result<NonNull<u8>> {
    unsafe {
        let ptr = VirtualAlloc(std::ptr::null(), size, MEM_COMMIT | MEM_RESERVE, PAGE_READWRITE);
        if ptr.is_null() {
            return Err(io::Error::last_os_error());
        }
        Ok(NonNull::new_unchecked(ptr as *mut u8))
    }
}

pub unsafe fn dealloc(ptr: *mut u8, _size: usize) {
    unsafe {
        let _ = VirtualFree(ptr as *mut _, 0, MEM_RELEASE);
    }
}

pub unsafe fn make_executable(ptr: *mut u8, size: usize) -> io::Result<()> {
    unsafe {
        let mut old_prot = 0;
        if VirtualProtect(ptr as _, size, PAGE_EXECUTE_READ, &mut old_prot) == 0 {
            return Err(io::Error::last_os_error());
        }
        FlushInstructionCache(GetCurrentProcess(), ptr as _, size);
    }
    Ok(())
}

pub unsafe fn make_writable(ptr: *mut u8, size: usize) -> io::Result<()> {
    unsafe {
        let mut old_prot = 0;
        if VirtualProtect(ptr as _, size, PAGE_READWRITE, &mut old_prot) == 0 {
            return Err(io::Error::last_os_error());
        }
    }
    Ok(())
}

pub unsafe fn write_protect_scoped<F, R>(ptr: *mut u8, size: usize, f: F) -> io::Result<R>
where
    F: FnOnce() -> R,
{
    let mut old_prot = 0;
    unsafe {
        if VirtualProtect(ptr as _, size, PAGE_READWRITE, &mut old_prot) == 0 {
            return Err(io::Error::last_os_error());
        }
    }

    let result = f();

    unsafe {
        let mut _dummy = 0;
        if VirtualProtect(ptr as _, size, old_prot, &mut _dummy) == 0 {
            return Err(io::Error::last_os_error());
        }
    }

    Ok(result)
}

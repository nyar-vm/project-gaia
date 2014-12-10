use libc::{mmap, mprotect, munmap, MAP_ANON, MAP_PRIVATE, PROT_EXEC, PROT_READ, PROT_WRITE};
use std::{io, ptr::NonNull};

pub fn get_page_size() -> usize {
    unsafe { libc::sysconf(libc::_SC_PAGESIZE) as usize }
}

pub fn alloc_rw(size: usize) -> io::Result<NonNull<u8>> {
    unsafe {
        let ptr = mmap(std::ptr::null_mut(), size, PROT_READ | PROT_WRITE, MAP_PRIVATE | MAP_ANON, -1, 0);
        if ptr == libc::MAP_FAILED {
            return Err(io::Error::last_os_error());
        }
        Ok(NonNull::new_unchecked(ptr as *mut u8))
    }
}

pub unsafe fn dealloc(ptr: *mut u8, size: usize) {
    unsafe {
        munmap(ptr as *mut _, size);
    }
}

pub unsafe fn make_executable(ptr: *mut u8, size: usize) -> io::Result<()> {
    unsafe {
        if mprotect(ptr as _, size, PROT_READ | PROT_EXEC) != 0 {
            return Err(io::Error::last_os_error());
        }
    }
    Ok(())
}

pub unsafe fn make_writable(ptr: *mut u8, size: usize) -> io::Result<()> {
    unsafe {
        if mprotect(ptr as _, size, PROT_READ | PROT_WRITE) != 0 {
            return Err(io::Error::last_os_error());
        }
    }
    Ok(())
}

pub unsafe fn write_protect_scoped<F, R>(ptr: *mut u8, size: usize, f: F) -> io::Result<R>
where
    F: FnOnce() -> R,
{
    // Unix implementation: we don't easily know the old protection without extra state,
    // so we just make it RW and then back to what we assume is needed.
    // For simplicity, we assume the user will call make_executable after they are done writing.
    unsafe {
        if mprotect(ptr as _, size, PROT_READ | PROT_WRITE) != 0 {
            return Err(io::Error::last_os_error());
        }
    }
    Ok(f())
}

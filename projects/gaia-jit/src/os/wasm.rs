use std::{io, ptr::NonNull};

pub fn get_page_size() -> usize {
    65536 // WASM page size is fixed at 64KB
}

pub fn alloc_rw(size: usize) -> io::Result<NonNull<u8>> {
    let layout =
        std::alloc::Layout::from_size_align(size, 65536).map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;
    unsafe {
        let ptr = std::alloc::alloc(layout);
        if ptr.is_null() {
            return Err(io::Error::new(io::ErrorKind::OutOfMemory, "WASM allocation failed"));
        }
        Ok(NonNull::new_unchecked(ptr))
    }
}

pub unsafe fn dealloc(ptr: *mut u8, size: usize) {
    let layout = std::alloc::Layout::from_size_align(size, 65536).unwrap();
    std::alloc::dealloc(ptr, layout);
}

pub unsafe fn make_executable(_ptr: *mut u8, _size: usize) -> io::Result<()> {
    // No-op or error on WASM depending on environment
    Ok(())
}

pub unsafe fn make_writable(_ptr: *mut u8, _size: usize) -> io::Result<()> {
    Ok(())
}

pub unsafe fn write_protect_scoped<F, R>(_ptr: *mut u8, _size: usize, f: F) -> io::Result<R>
where
    F: FnOnce() -> R,
{
    Ok(f())
}

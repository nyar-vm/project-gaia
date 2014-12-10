#![warn(missing_docs)]

use std::{io, ptr::NonNull, slice};

mod os;

/// A buffer for JIT code execution.
///
/// # Example
///
/// ```rust
/// use gaia_jit::JitMemory;
///
/// let mut jit = JitMemory::new(4096).unwrap();
/// // Write some machine code...
/// jit.write(&[0x90, 0xc3]).unwrap(); // nop; ret
/// jit.make_executable().unwrap();
///
/// unsafe {
///     let f: unsafe extern "C" fn() = jit.as_fn();
///     f();
/// }
/// ```
pub struct JitMemory {
    ptr: NonNull<u8>,
    len: usize,
    capacity: usize,
}

impl JitMemory {
    /// Allocate a new JIT memory block with the given capacity.
    pub fn new(capacity: usize) -> io::Result<Self> {
        let page_size = os::get_page_size();
        let rounded_capacity = (capacity + page_size - 1) & !(page_size - 1);

        let ptr = os::alloc_rw(rounded_capacity)?;

        Ok(Self { ptr, len: 0, capacity: rounded_capacity })
    }

    /// Write code into the buffer.
    ///
    /// This method automatically ensures the memory is writable before writing,
    /// and restores the previous protection after if possible.
    pub fn write(&mut self, code: &[u8]) -> io::Result<()> {
        if self.len + code.len() > self.capacity {
            return Err(io::Error::new(io::ErrorKind::OutOfMemory, "Not enough capacity in JIT buffer"));
        }

        unsafe {
            os::write_protect_scoped(self.ptr.as_ptr(), self.capacity, || {
                std::ptr::copy_nonoverlapping(code.as_ptr(), self.ptr.as_ptr().add(self.len), code.len());
            })?;
        }

        self.len += code.len();
        Ok(())
    }

    /// Make the memory executable and return a pointer to the start.
    pub fn make_executable(&self) -> io::Result<*const u8> {
        unsafe {
            os::make_executable(self.ptr.as_ptr(), self.capacity)?;
        }
        Ok(self.ptr.as_ptr())
    }

    /// Make the memory readable and writable again (e.g. for updates).
    pub fn make_writable(&self) -> io::Result<()> {
        unsafe {
            os::make_writable(self.ptr.as_ptr(), self.capacity)?;
        }
        Ok(())
    }

    /// Get the current length of the code in the buffer.
    pub fn len(&self) -> usize {
        self.len
    }

    /// Get the capacity of the buffer.
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Is the buffer empty?
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// As a slice of bytes (read-only).
    pub fn as_slice(&self) -> &[u8] {
        unsafe { slice::from_raw_parts(self.ptr.as_ptr(), self.len) }
    }

    /// Cast the JIT memory to a function pointer of type `S`.
    pub unsafe fn as_fn<S>(&self) -> S {
        unsafe { std::mem::transmute_copy(&self.ptr.as_ptr()) }
    }
}

impl Drop for JitMemory {
    fn drop(&mut self) {
        unsafe {
            os::dealloc(self.ptr.as_ptr(), self.capacity);
        }
    }
}

unsafe impl Send for JitMemory {}
unsafe impl Sync for JitMemory {}

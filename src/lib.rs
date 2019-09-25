#![no_std]

// We need std in test mode to assert
#[cfg(test)]
extern crate std;

#[cfg(not(target_os = "linux"))]
core::compile_error!("This library is linux-specific");

extern crate alloc;

mod cstr;
pub mod directory;
mod error;
pub mod syscalls;

pub use cstr::CStr;
pub use directory::Directory;
pub use error::Error;

struct LibcAllocator;

use core::alloc::{GlobalAlloc, Layout};
unsafe impl GlobalAlloc for LibcAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        libc::malloc(layout.size()) as *mut u8
    }
    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        libc::free(ptr as *mut libc::c_void)
    }
    unsafe fn realloc(&self, ptr: *mut u8, _layout: Layout, new_size: usize) -> *mut u8 {
        libc::realloc(ptr as *mut libc::c_void, new_size) as *mut u8
    }
}

#[cfg(test)]
#[global_allocator]
static ALLOC: LibcAllocator = LibcAllocator;

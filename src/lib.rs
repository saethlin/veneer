#![feature(asm)]
#![no_std]
#![deny(clippy::missing_inline_in_public_items)]

// We need std in test mode to assert
#[cfg(test)]
extern crate std;

#[cfg(not(target_os = "linux"))]
core::compile_error!("This library is linux-specific");

extern crate alloc;

mod cstr;
pub mod directory;
mod error;
pub mod fs;
pub mod io;
pub mod syscalls;

pub use cstr::CStr;
pub use directory::Directory;
pub use error::Error;

use core::alloc::{GlobalAlloc, Layout};

pub struct LibcAllocator;

unsafe impl GlobalAlloc for LibcAllocator {
    #[inline]
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        libc::malloc(layout.size()) as *mut u8
    }

    #[inline]
    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        libc::free(ptr as *mut libc::c_void)
    }

    #[inline]
    unsafe fn realloc(&self, ptr: *mut u8, _layout: Layout, new_size: usize) -> *mut u8 {
        libc::realloc(ptr as *mut libc::c_void, new_size) as *mut u8
    }

    #[inline]
    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        libc::calloc(layout.size(), 1) as *mut u8
    }
}

#[cfg(test)]
#[global_allocator]
static ALLOC: LibcAllocator = LibcAllocator;

#[inline]
pub fn print(bytes: &[u8]) {
    let _ = syscalls::write(libc::STDOUT_FILENO, bytes);
}

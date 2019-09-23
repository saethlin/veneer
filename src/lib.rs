#![no_std]

extern crate alloc;

mod cstr;
pub mod directory;
mod error;
pub mod syscalls;

pub use cstr::CStr;
pub use directory::Directory;
pub use error::Error;

#[cfg(not(target_os = "linux"))]
core::compile_error!("This library is linux-specific");

#![no_std]
#![warn(clippy::missing_inline_in_public_items)]

// We need std in test mode to assert
#[cfg(test)]
extern crate std;

#[cfg(not(target_os = "linux"))]
core::compile_error!("This library is linux-specific");

extern crate alloc;

mod allocator;
mod cstr;
pub mod env;
mod error;
pub mod fmt;
pub mod fs;
pub mod io;
pub mod net;
mod spinlock;
pub mod syscalls;

pub use allocator::Allocator;
pub use cstr::CStr;
pub use error::Error;

#[cfg(not(test))]
#[global_allocator]
static ALLOC: crate::Allocator = crate::Allocator::new();

#[macro_export]
macro_rules! print {
    ($($args:tt)*) => {
        core::fmt::write(&mut veneer::io::Stdout, format_args!($($args)*)).unwrap();
    };
}

#[macro_export]
macro_rules! println {
    ($format:expr, $($args:tt)*) => {
        core::fmt::write(&mut crate::io::Stdout, format_args!(concat!($format, "\n"), $($args)*)).unwrap();
    };
}

#[macro_export]
macro_rules! eprint {
    ($($args:tt)*) => {
        core::fmt::write(&mut veneer::io::Stderr, format_args!($($args)*)).unwrap();
    };
}

#[macro_export]
macro_rules! eprintln {
    ($format:expr, $($args:tt)*) => {
        core::fmt::write(&mut veneer::io::Stderr, format_args!(concat!($format, "\n"), $($args)*)).unwrap();
    };
}

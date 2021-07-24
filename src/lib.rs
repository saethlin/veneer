#![feature(asm)]
#![no_std]
#![deny(clippy::missing_inline_in_public_items)]

// We need std in test mode to assert
#[cfg(test)]
extern crate std;

#[cfg(not(target_os = "linux"))]
core::compile_error!("This library is linux-specific");

extern crate alloc;

mod allocator;
mod cstr;
pub mod directory;
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
pub use directory::Directory;
pub use error::Error;

#[cfg(test)]
#[global_allocator]
static ALLOC: Allocator = Allocator::new();

#[macro_export]
macro_rules! print {
    ($($args:tt)*) => {
        core::fmt::write(&mut veneer::io::Stdout, format_args!($($args)*)).unwrap();
    };
}

#[macro_export]
macro_rules! println {
    ($format:expr, $($args:tt)*) => {
        core::fmt::write(&mut veneer::io::Stdout, format_args!(concat!($format, "\n"), $($args)*)).unwrap();
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

#[macro_export]
macro_rules! prelude {
    () => {
        #[lang = "eh_personality"]
        #[no_mangle]
        pub extern "C" fn rust_eh_personality() {}

        #[panic_handler]
        fn panic(info: &core::panic::PanicInfo) -> ! {
            veneer::eprint!("{}", info);
            let _ = veneer::syscalls::kill(0, libc::SIGABRT);
            veneer::syscalls::exit(-1);
            loop {}
        }

        #[alloc_error_handler]
        fn alloc_error(layout: core::alloc::Layout) -> ! {
            veneer::eprint!(
                "Unable to allocate, size: {}\n",
                itoa::Buffer::new().format(layout.size())
            );
            let _ = veneer::syscalls::kill(0, libc::SIGABRT);
            veneer::syscalls::exit(-1);
            loop {}
        }

        #[global_allocator]
        static ALLOC: veneer::Allocator = veneer::Allocator::new();

        use veneer::{eprint, eprintln, print, println};

        #[start]
        fn start(argc: isize, argp: *const *const u8) -> isize {
            main();
            0
        }
    };
}

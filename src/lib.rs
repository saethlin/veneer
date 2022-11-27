#![cfg_attr(not(test), no_std)]
#![feature(naked_functions, alloc_error_handler, lang_items)]
#![warn(clippy::missing_inline_in_public_items)]
#![feature(cfg_target_has_atomic, core_intrinsics, inline_const, linkage)]
#![allow(internal_features)] // Must use lang_items to implement a Rust runtime

#[cfg(not(target_os = "linux"))]
core::compile_error!(
    "This library is only implemented for Linux,\n\
    because the primary goal of this library is to bypass"
);

#[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
compile_error!("This crate is only implemented for x86_64 and aarch64");

extern crate alloc;

mod allocator;
mod cstr;
pub mod env;
mod error;
pub mod fmt;
pub mod fs;
pub mod io;
mod mem;
pub mod net;
pub mod prelude;
mod spinlock;
pub mod syscalls;

pub use allocator::Allocator;
pub use cstr::CStr;
pub use error::Error;
pub use veneer_macros::main;

#[cfg(all(feature = "rt", not(test)))]
#[lang = "eh_personality"]
#[no_mangle]
pub extern "C" fn eh_personality() {}

#[cfg(all(feature = "rt", not(test)))]
#[alloc_error_handler]
fn alloc_error(layout: core::alloc::Layout) -> ! {
    panic!("memory allocation of {} bytes failed", layout.size());
}

#[cfg(all(feature = "rt", not(test)))]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    crate::eprintln!("{}", info);
    let _ = crate::syscalls::kill(0, 6);
    crate::syscalls::exit(-1)
}

#[cfg(all(feature = "rt", not(test), target_arch = "x86_64"))]
#[no_mangle]
#[naked]
unsafe extern "C" fn _start() {
    // Just move argc and argv into the right registers and call main
    core::arch::asm!(
        "mov rdi, [rsp]", // The value of rsp is actually a pointer to argc
        "mov rsi, rsp",
        "add rsi, 8", // But for argv we just increment the rsp pointer by 1 (offset by 8)
        "call __veneer_init",
        "call __veneer_main",
        options(noreturn)
    )
}

#[cfg(all(feature = "rt", not(test), target_arch = "aarch64"))]
#[no_mangle]
#[naked]
unsafe extern "C" fn _start() {
    core::arch::asm!(
        "ldr x0, [sp]",
        "mov x1, sp",
        "add x1, x1, 0x8",
        "bl __veneer_init",
        "bl __veneer_main",
        options(noreturn)
    )
}

#[cfg(all(feature = "rt", not(test)))]
#[no_mangle]
unsafe extern "C" fn __veneer_init(argc: isize, argv: *mut *const u8) {
    crate::env::ARGC.store(argc, core::sync::atomic::Ordering::SeqCst);
    crate::env::ARGV.store(argv.cast(), core::sync::atomic::Ordering::SeqCst);
}

#[cfg(all(feature = "rt", not(test)))]
#[global_allocator]
static ALLOC: crate::Allocator = crate::Allocator::new();

#[macro_export]
macro_rules! print {
    ($($args:tt)*) => {
        core::fmt::write(&mut $crate::io::Stdout, format_args!($($args)*)).unwrap();
    };
}

#[macro_export]
macro_rules! println {
    () => {
        <$crate::io::Stdout as core::fmt::Write>::write_str(&mut $crate::io::Stdout, "\n").unwrap();
    };
    ($format:expr) => {
        <$crate::io::Stdout as core::fmt::Write>::write_str(&mut $crate::io::Stdout, concat!($format, "\n")).unwrap();
    };
    ($format:expr, $($args:tt)*) => {
        core::fmt::write(&mut $crate::io::Stdout, format_args!(concat!($format, "\n"), $($args)*)).unwrap();
    };
}

#[macro_export]
macro_rules! eprint {
    ($($args:tt)*) => {
        core::fmt::write(&mut $crate::io::Stderr, format_args!($($args)*)).unwrap();
    };
}

#[macro_export]
macro_rules! eprintln {
    ($format:expr, $($args:tt)*) => {
        core::fmt::write(&mut $crate::io::Stderr, format_args!(concat!($format, "\n"), $($args)*)).unwrap();
    };
}

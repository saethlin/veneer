use crate::{CStr, Error};

use libc::c_int;
use syscall::syscall;

pub fn exit(error_code: c_int) -> ! {
    unsafe {
        syscall!(EXIT, error_code);
        core::hint::unreachable_unchecked();
    }
}

pub fn write(fd: c_int, bytes: &[u8]) -> Result<usize, Error> {
    unsafe { syscall!(WRITE, fd, bytes.as_ptr(), bytes.len()) }.to_result_and(|n| n)
}

pub fn close(fd: c_int) -> Result<(), Error> {
    unsafe { syscall!(CLOSE, fd) }.to_result_with(())
}

pub fn kill(pid: usize, signal: i32) -> Result<(), Error> {
    unsafe { syscall!(KILL, pid, signal) }.to_result_with(())
}

pub fn mmap(
    addr: *mut u8,
    len: usize,
    prot: i32,
    flags: i32,
    fd: i32,
    offset: isize,
) -> Result<*mut u8, Error> {
    unsafe { syscall!(MMAP, addr, len, prot, flags, fd, offset).to_result_and(|n| n as *mut u8) }
}

pub fn munmap(addr: *mut u8, len: usize) -> Result<(), Error> {
    unsafe { syscall!(MUNMAP, addr, len) }.to_result_with(())
}

pub fn open_dir(path: CStr) -> Result<c_int, Error> {
    unsafe {
        syscall!(
            OPEN,
            path.as_ptr(),
            libc::O_RDONLY | libc::O_DIRECTORY | libc::O_CLOEXEC
        )
    }
    .to_result_and(|n| n as c_int)
}

pub fn fstatat(fd: c_int, name: CStr) -> Result<libc::stat64, Error> {
    unsafe {
        let mut stats = core::mem::zeroed();
        syscall!(
            NEWFSTATAT,
            fd,
            name.as_ptr(),
            &mut stats as *mut libc::stat64,
            0
        )
        .to_result_with(stats)
    }
}

pub fn lstatat(fd: c_int, name: CStr) -> Result<libc::stat64, Error> {
    unsafe {
        let mut stats = core::mem::zeroed();
        syscall!(
            NEWFSTATAT,
            fd,
            name.as_ptr(),
            &mut stats as *mut libc::stat64,
            libc::AT_SYMLINK_NOFOLLOW
        )
        .to_result_with(stats)
    }
}

pub fn getdents64(fd: c_int, buf: &mut [u8]) -> Result<usize, Error> {
    unsafe { syscall!(GETDENTS64, fd, buf.as_mut_ptr(), buf.len()) }.to_result_and(|n| n)
}

pub fn faccessat(fd: c_int, name: CStr, mode: c_int) -> Result<(), Error> {
    unsafe { syscall!(FACCESSAT, fd, name.as_ptr(), mode) }.to_result_with(())
}

pub fn readlinkat(fd: c_int, name: CStr, buf: &mut [u8]) -> Result<usize, Error> {
    unsafe { syscall!(READLINKAT, fd, name.as_ptr(), buf.as_ptr(), buf.len()) }.to_result_and(|n| n)
}

pub fn winsize() -> Result<libc::winsize, Error> {
    unsafe {
        let mut winsize: libc::winsize = core::mem::zeroed();
        syscall::syscall!(
            IOCTL,
            libc::STDOUT_FILENO,
            libc::TIOCGWINSZ,
            &mut winsize as *mut libc::winsize
        )
        .to_result_with(winsize)
    }
}

trait SyscallRet {
    fn to_result_with<T>(self, t: T) -> Result<T, Error>;
    fn to_result_and<T, F>(self, f: F) -> Result<T, Error>
    where
        F: FnOnce(Self) -> T,
        Self: Sized;
}

impl SyscallRet for usize {
    fn to_result_with<T>(self, t: T) -> Result<T, Error> {
        let ret = self as isize;
        if ret < 0 {
            Err(Error(-ret))
        } else {
            Ok(t)
        }
    }

    fn to_result_and<T, F>(self, f: F) -> Result<T, Error>
    where
        F: FnOnce(Self) -> T,
        Self: Sized,
    {
        let ret = self as isize;
        if ret < 0 {
            Err(Error(-ret))
        } else {
            Ok(f(self))
        }
    }
}

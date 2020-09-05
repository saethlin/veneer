use crate::{CStr, Error};

use libc::c_int;
use sc::syscall;

pub fn read(fd: c_int, bytes: &mut [u8]) -> Result<usize, Error> {
    unsafe { syscall!(READ, fd, bytes.as_ptr(), bytes.len()) }.to_result_and(|n| n)
}

pub fn write(fd: c_int, bytes: &[u8]) -> Result<usize, Error> {
    unsafe { syscall!(WRITE, fd, bytes.as_ptr(), bytes.len()) }.to_result_and(|n| n)
}

// For directories libc::O_RDONLY | libc::O_DIRECTORY | libc::O_CLOEXEC
bitflags::bitflags! {
    pub struct OPEN: libc::c_int {
        const RDONLY = libc::O_RDONLY;
        const WRONLY = libc::O_WRONLY;
        const RDWR = libc::O_RDWR;
        const APPEND = libc::O_APPEND;
        const ASYNC = libc::O_ASYNC;
        const CLOEXEC = libc::O_CLOEXEC;
        const CREAT = libc::O_CREAT;
        const DIRECT = libc::O_DIRECT;
        const DIRECTORY = libc::O_DIRECTORY;
        const DSYNC = libc::O_DSYNC;
        const EXCL = libc::O_EXCL;
        const LARGEFILE = libc::O_LARGEFILE;
        const NOATIME = libc::O_NOATIME;
        const NOCTTY = libc::O_NOCTTY;
        const NOFOLLOW = libc::O_NOFOLLOW;
        const NONBLOCK = libc::O_NONBLOCK;
        const PATH = libc::O_PATH;
        const SYNC = libc::O_SYNC;
        const TMPFILE = libc::O_TMPFILE;
        const TRUNC = libc::O_TRUNC;
    }
}
pub fn open(path: CStr, flags: OPEN) -> Result<c_int, Error> {
    unsafe { syscall!(OPEN, path.as_ptr(), flags.bits) }.to_result_and(|n| n as c_int)
}

pub fn close(fd: c_int) -> Result<(), Error> {
    unsafe { syscall!(CLOSE, fd) }.to_result_with(())
}

pub fn stat(path: CStr) -> Result<libc::stat, Error> {
    unsafe {
        let mut status: libc::stat = core::mem::zeroed();
        syscall!(STAT, path.as_ptr(), &mut status as *mut libc::stat).to_result_with(status)
    }
}

pub fn fstat(fd: c_int) -> Result<libc::stat, Error> {
    unsafe {
        let mut status: libc::stat = core::mem::zeroed();
        syscall!(FSTAT, fd, &mut status as *mut libc::stat).to_result_with(status)
    }
}

pub fn lstat(path: CStr) -> Result<libc::stat, Error> {
    unsafe {
        let mut status: libc::stat = core::mem::zeroed();
        syscall!(FSTAT, path.as_ptr(), &mut status as *mut libc::stat).to_result_with(status)
    }
}

pub fn poll(fds: &mut [libc::pollfd], timeout: c_int) -> Result<usize, Error> {
    unsafe { syscall!(POLL, fds.as_ptr(), fds.len(), timeout) }.to_result_and(|n| n)
}

pub enum SeekFrom {
    Start,
    End,
    Current,
}
pub fn lseek(fd: c_int, seek_mode: SeekFrom, offset: usize) -> Result<usize, Error> {
    let seek_mode = match seek_mode {
        SeekFrom::Start => libc::SEEK_SET,
        SeekFrom::End => libc::SEEK_END,
        SeekFrom::Current => libc::SEEK_CUR,
    };
    unsafe { syscall!(LSEEK, fd, seek_mode, offset) }.to_result_and(|n| n)
}

pub unsafe fn mmap(
    addr: *mut u8,
    len: usize,
    prot: i32,
    flags: i32,
    fd: i32,
    offset: isize,
) -> Result<*mut u8, Error> {
    syscall!(MMAP, addr, len, prot, flags, fd, offset).to_result_and(|n| n as *mut u8)
}

pub unsafe fn mprotect(addr: *mut u8, len: usize, protection: libc::c_int) -> Result<(), Error> {
    syscall!(MPROTECT, addr, len, protection).to_result_with(())
}

pub unsafe fn munmap(addr: *mut u8, len: usize) -> Result<(), Error> {
    syscall!(MUNMAP, addr, len).to_result_with(())
}

pub fn mremap(
    old_address: *mut u8,
    old_size: usize,
    new_size: usize,
    flags: c_int,
) -> Result<*mut u8, Error> {
    unsafe { syscall!(MREMAP, old_address, old_size, new_size, flags) }
        .to_result_and(|n| n as *mut u8)
}

pub fn brk(addr: *mut u8) -> Result<*mut u8, Error> {
    unsafe { syscall!(BRK, addr) }.to_result_and(|n| n as *mut u8)
}

// Require that it non-negative
pub struct Pid(libc::pid_t);

pub enum SignalWhere {
    Exactly(usize),
    CurrentGroup,
    AllValid,
    Group(usize),
}
pub fn kill(pid: usize, signal: i32) -> Result<(), Error> {
    unsafe { syscall!(KILL, pid, signal) }.to_result_with(())
}

// Wraps the rt_sigaction call in the same way that glibc does
// So I guess there's no way to use normal signals, only realtime signals?
pub fn sigaction(
    signal: c_int,
    action: &libc::sigaction,
    old_action: &mut libc::sigaction,
) -> Result<(), Error> {
    unsafe {
        syscall!(
            RT_SIGACTION,
            signal,
            action as *const libc::sigaction,
            old_action as *mut libc::sigaction,
            core::mem::size_of::<libc::sigset_t>()
        )
    }
    .to_result_with(())
}

//pub fn sigprocmask

//pub fn sigreturn

//pub fn ioctl

pub fn exit(error_code: c_int) {
    unsafe {
        syscall!(EXIT, error_code);
    }
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

pub fn readlinkat<'a>(fd: c_int, name: CStr, buf: &'a mut [u8]) -> Result<&'a [u8], Error> {
    match unsafe { syscall!(READLINKAT, fd, name.as_ptr(), buf.as_ptr(), buf.len()) }
        .to_result_and(|n| n)
    {
        Ok(n) => Ok(buf.get(..n).unwrap_or_default()),
        Err(e) => Err(e),
    }
}

pub fn winsize() -> Result<libc::winsize, Error> {
    unsafe {
        let mut winsize: libc::winsize = core::mem::zeroed();
        syscall!(
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

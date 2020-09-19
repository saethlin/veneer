use crate::{CStr, Error};
use core::{marker::PhantomData, mem};
use libc::c_int;
use sc::syscall;

pub fn read(fd: c_int, bytes: &mut [u8]) -> Result<usize, Error> {
    unsafe { syscall!(READ, fd, bytes.as_ptr(), bytes.len()) }.usize_result()
}

pub fn write(fd: c_int, bytes: &[u8]) -> Result<usize, Error> {
    unsafe { syscall!(WRITE, fd, bytes.as_ptr(), bytes.len()) }.usize_result()
}

// For directories RDONLY | DIRECTORY | CLOEXEC
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
    unsafe { syscall!(CLOSE, fd) }.null_result()
}

pub fn stat(path: CStr) -> Result<libc::stat, Error> {
    unsafe {
        let mut status: libc::stat = mem::zeroed();
        syscall!(STAT, path.as_ptr(), &mut status as *mut libc::stat).to_result_with(status)
    }
}

pub fn fstat(fd: c_int) -> Result<libc::stat, Error> {
    unsafe {
        let mut status: libc::stat = mem::zeroed();
        syscall!(FSTAT, fd, &mut status as *mut libc::stat).to_result_with(status)
    }
}

pub fn lstat(path: CStr) -> Result<libc::stat, Error> {
    unsafe {
        let mut status: libc::stat = mem::zeroed();
        syscall!(FSTAT, path.as_ptr(), &mut status as *mut libc::stat).to_result_with(status)
    }
}

pub fn poll(fds: &mut [libc::pollfd], timeout: c_int) -> Result<usize, Error> {
    unsafe { syscall!(POLL, fds.as_ptr(), fds.len(), timeout) }.usize_result()
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
    unsafe { syscall!(LSEEK, fd, seek_mode, offset) }.usize_result()
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
    syscall!(MPROTECT, addr, len, protection).null_result()
}

pub unsafe fn munmap(addr: *mut u8, len: usize) -> Result<(), Error> {
    syscall!(MUNMAP, addr, len).null_result()
}

pub fn brk(addr: *mut u8) -> Result<*mut u8, Error> {
    unsafe { syscall!(BRK, addr) }.to_result_and(|n| n as *mut u8)
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
            mem::size_of::<libc::sigset_t>()
        )
    }
    .to_result_with(())
}

// sigprocmask

// sigreturn

#[macro_export]
macro_rules! ioctl {
    ($fd:expr, $request:expr, $($arg:expr),*) => {
        unsafe { syscall!(IOCTL, $fd, $request, $($arg)*) }.usize_result()
    };
    ($fd:expr, $request:expr, $($arg:expr),*) => {
        ioctl!($fd, $request, $($arg)*)
    };
}

pub fn pread64(fd: c_int, buf: &mut [u8], offset: usize) -> Result<usize, Error> {
    unsafe { syscall!(PREAD64, fd, buf.as_mut_ptr(), buf.len(), offset) }.usize_result()
}

pub fn pwrite64(fd: c_int, buf: &[u8], offset: usize) -> Result<usize, Error> {
    unsafe { syscall!(PWRITE64, fd, buf.as_ptr(), buf.len(), offset) }.usize_result()
}

pub struct IoVec<'a> {
    #[allow(dead_code)]
    iov_base: *mut u8,
    #[allow(dead_code)]
    iov_len: usize,
    _danny: PhantomData<&'a mut u8>,
}

pub fn readv<'a, 'b>(fd: c_int, iovec: &'b mut [IoVec<'a>]) -> Result<usize, Error> {
    unsafe { syscall!(READV, fd, iovec.as_mut_ptr(), iovec.len()) }.usize_result()
}

pub fn writev<'a, 'b>(fd: c_int, iovec: &'b [IoVec<'a>]) -> Result<usize, Error> {
    unsafe { syscall!(READV, fd, iovec.as_ptr(), iovec.len()) }.usize_result()
}

bitflags::bitflags! {
    pub struct Mode: c_int {
        const F_OK = 0;
        const R_OK = 4;
        const W_OK = 2;
        const X_OK = 1;
    }
}

pub fn access(pathname: CStr, mode: Mode) -> Result<(), Error> {
    unsafe { syscall!(ACCESS, pathname.as_ptr(), mode.bits()) }.null_result()
}

pub fn pipe() -> Result<[c_int; 2], Error> {
    let mut pipefd: [c_int; 2] = [0, 0];
    unsafe { syscall!(PIPE, pipefd.as_mut_ptr()) }.to_result_with(pipefd)
}

pub fn select(
    nfds: c_int,
    readfds: &mut libc::fd_set,
    writefds: &mut libc::fd_set,
    exceptfds: &mut libc::fd_set,
    timeout: &mut libc::timespec,
) -> Result<c_int, Error> {
    unsafe {
        syscall!(
            SELECT,
            nfds,
            readfds as *mut libc::fd_set,
            writefds as *mut libc::fd_set,
            exceptfds as *mut libc::fd_set,
            timeout as *mut libc::timespec
        )
    }
    .to_result_and(|n| n as c_int)
}

pub fn sched_yield() -> Result<(), Error> {
    unsafe { syscall!(SCHED_YIELD) }.null_result()
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

bitflags::bitflags! {
    pub struct MSync: c_int {
        const ASYNC = libc::MS_ASYNC;
        const SYNC = libc::MS_SYNC;
        const INVALIDATE = libc::MS_INVALIDATE;
    }
}

pub fn msync(memory: &[u8], flags: MSync) -> Result<(), Error> {
    unsafe { syscall!(MSYNC, memory.as_ptr(), memory.len(), flags.bits()) }.null_result()
}

pub fn mincore(memory: &[u8], status: &mut [u8]) -> Result<(), Error> {
    if status.len() < (memory.len() + 4096 - 1) / 4096 {
        return Err(Error(libc::EINVAL as isize));
    }
    unsafe { syscall!(MINCORE, memory.as_ptr(), memory.len(), status.as_mut_ptr()) }.null_result()
}

bitflags::bitflags! {
    pub struct Advice: c_int {
        const NORMAL = libc::MADV_NORMAL;
        const RANDOM = libc::MADV_RANDOM;
        const SEQUENTIAL = libc::MADV_SEQUENTIAL;
        const WILLNEED = libc::MADV_WILLNEED;
        const DONTNEED = libc::MADV_DONTNEED;
        const REMOVE = libc::MADV_REMOVE;
        const DONTFORM = libc::MADV_DONTFORK;
        const DOFORK = libc::MADV_DOFORK;
        const HWPOISON = libc::MADV_HWPOISON;
        const MERGEABLE = libc::MADV_MERGEABLE;
        const UNMERGEABLE = libc::MADV_UNMERGEABLE;
        const SOFT_OFFLINE = libc::MADV_SOFT_OFFLINE;
        const HUGEPAGE = libc::MADV_HUGEPAGE;
        const NOHUGEPAGE = libc::MADV_NOHUGEPAGE;
        const DONTDUMP = libc::MADV_DONTDUMP;
        const DODUMP = libc::MADV_DODUMP;
        const FREE = libc::MADV_FREE;
        //const WIPEONFORK = libc::MADV_WIPEONFORK;
        //const KEEPONFORK = libc::MADV_KEEPONFORK;
    }
}

pub fn madvise(memory: &[u8], advice: Advice) -> Result<(), Error> {
    unsafe { syscall!(MADVISE, memory.as_ptr(), memory.len(), advice.bits()) }.null_result()
}

bitflags::bitflags! {
    pub struct ShmFlags: c_int {
        const CREAT = libc::IPC_CREAT;
        const EXCL = libc::IPC_EXCL;
        const HUGETLB = libc::SHM_HUGETLB;
        const HUGE_2MB = libc::MAP_HUGE_2MB;
        const HUGE_1GB = libc::MAP_HUGE_1GB;
        const NORESERVE = libc::SHM_NORESERVE;
    }
}

pub fn shmget(key: libc::key_t, size: usize, shmflg: ShmFlags) -> Result<libc::key_t, Error> {
    unsafe { syscall!(SHMGET, key, size, shmflg.bits()) }.to_result_and(|key| key as libc::key_t)
}

// shmctl
//
// dup
//
// dup2
//
// pause
//
// nanosleep
//
// getitimer
//
// alarm
//
// setitimer
//
// getpid
//
// sendfile
//
// socket
//
// connect
//
// accept
//
// sendto
//
// recvfrom
//
// sendmsg
//
// recvmsg
//
// shutdown
//
// bind
//
// listen
//
// getsockname
//
// getpeername
//
// socketpair
//
// setsockopt
//
// getsockopt
//
// clone
//
// fork
//
// execve

pub fn exit(error_code: c_int) {
    unsafe {
        syscall!(EXIT, error_code);
    }
}

// wait4

// Require that it is non-negative
pub struct Pid(libc::pid_t);

pub enum SignalWhere {
    Exactly(usize),
    CurrentGroup,
    AllValid,
    Group(usize),
}
pub fn kill(pid: usize, signal: i32) -> Result<(), Error> {
    unsafe { syscall!(KILL, pid, signal) }.null_result()
}

// uname

pub fn fstatat(fd: c_int, name: CStr) -> Result<libc::stat64, Error> {
    unsafe {
        let mut stats = mem::zeroed();
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
        let mut stats = mem::zeroed();
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
    unsafe { syscall!(FACCESSAT, fd, name.as_ptr(), mode) }.null_result()
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
        let mut winsize: libc::winsize = mem::zeroed();
        syscall!(
            IOCTL,
            libc::STDOUT_FILENO,
            libc::TIOCGWINSZ,
            &mut winsize as *mut libc::winsize
        )
        .to_result_with(winsize)
    }
}

trait SyscallRet: Sized {
    fn to_result_with<T>(self, t: T) -> Result<T, Error>;
    fn to_result_and<T, F>(self, f: F) -> Result<T, Error>
    where
        F: FnOnce(Self) -> T,
        Self: Sized;

    fn usize_result(self) -> Result<usize, Error>;

    fn null_result(self) -> Result<(), Error> {
        self.to_result_with(())
    }
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

    fn usize_result(self) -> Result<usize, Error> {
        self.to_result_and(|n| n)
    }
}

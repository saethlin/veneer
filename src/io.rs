pub use crate::Error;

pub type Result<T> = core::result::Result<T, Error>;

pub trait Read {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize>;
}

pub trait Write {
    fn write(&mut self, buf: &[u8]) -> Result<usize>;

    #[inline]
    fn write_all(&mut self, mut buf: &[u8]) -> Result<()> {
        while !buf.is_empty() {
            match self.write(buf) {
                Ok(0) => {
                    return Err(Error(libc::EBADF));
                }
                Ok(n) => buf = &buf[n..],
                Err(Error(libc::EAGAIN)) => {}
                Err(e) => return Err(e),
            }
        }
        Ok(())
    }
}

pub struct Stdout;

impl Write for Stdout {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        crate::syscalls::write(libc::STDOUT_FILENO, buf)
    }
}

impl core::fmt::Write for Stdout {
    #[inline]
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.write(s.as_bytes()).unwrap();
        Ok(())
    }
}

pub struct Stderr;

impl Write for Stderr {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        crate::syscalls::write(libc::STDERR_FILENO, buf)
    }
}

impl core::fmt::Write for Stderr {
    #[inline]
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.write(s.as_bytes()).unwrap();
        Ok(())
    }
}

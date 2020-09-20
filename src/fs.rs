use crate::{
    io::{Read, Write},
    syscalls,
    syscalls::{OpenFlags, OpenMode},
    CStr, Error,
};
use alloc::{vec, vec::Vec};
use libc::c_int;

#[derive(Debug)]
pub struct File(c_int);

impl File {
    #[inline]
    pub fn open(path: &[u8]) -> Result<Self, Error> {
        Ok(Self(syscalls::open(
            CStr::from_bytes(path),
            OpenFlags::RDONLY | OpenFlags::CLOEXEC,
            OpenMode::empty(),
        )?))
    }

    #[inline]
    pub fn create(path: &[u8]) -> Result<Self, Error> {
        Ok(Self(syscalls::open(
            CStr::from_bytes(path),
            OpenFlags::RDWR | OpenFlags::CREAT | OpenFlags::CLOEXEC,
            OpenMode::RUSR
                | OpenMode::WUSR
                | OpenMode::RGRP
                | OpenMode::WGRP
                | OpenMode::ROTH
                | OpenMode::WOTH,
        )?))
    }
}

impl Drop for File {
    #[inline]
    fn drop(&mut self) {
        let _ = syscalls::close(self.0);
    }
}

impl Read for File {
    #[inline]
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Error> {
        syscalls::read(self.0, buf)
    }
}

impl Write for File {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> Result<usize, Error> {
        syscalls::write(self.0, buf)
    }
}

#[inline]
pub fn read(path: &[u8]) -> Result<Vec<u8>, Error> {
    let file_len = syscalls::stat(CStr::from_bytes(path)).map(|stat| stat.st_size)?;
    let mut file = File::open(path)?;
    let mut bytes = vec![0; file_len as usize];
    let mut buf = &mut bytes[..];
    while !buf.is_empty() {
        match file.read(buf) {
            Ok(0) => break,
            Ok(n) => buf = &mut buf[n..],
            Err(Error(libc::EAGAIN)) => {}
            Err(e) => return Err(e),
        }
    }
    Ok(bytes)
}

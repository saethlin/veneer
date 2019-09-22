use crate::{syscalls, CStr};
use alloc::{vec, vec::Vec};
use libc::c_int;

pub struct Directory {
    fd: c_int,
}

impl Directory {
    pub fn raw_fd(&self) -> c_int {
        self.fd
    }
}

impl Drop for Directory {
    fn drop(&mut self) {
        let _ = syscalls::close(self.fd);
    }
}

pub struct DirectoryContents {
    dirents: Vec<u8>,
}

impl<'a> Directory {
    pub fn open(path: CStr) -> Result<Self, crate::Error> {
        let fd = syscalls::open_dir(path)?;

        Ok(Self { fd })
    }

    pub fn read(&self) -> Result<DirectoryContents, crate::Error> {
        let mut dirents = vec![0u8; 4096];
        let mut bytes_read = syscalls::getdents64(self.fd, &mut dirents[..])?;
        let mut bytes_used = bytes_read;

        while bytes_read > 0 {
            dirents.extend(core::iter::repeat(0).take(4096));
            bytes_read = syscalls::getdents64(self.fd, &mut dirents[bytes_used..])?;
            bytes_used += bytes_read;
        }

        dirents.truncate(bytes_read);

        Ok(DirectoryContents { dirents })
    }
}

impl DirectoryContents {
    pub fn iter(&self) -> IterDir {
        IterDir {
            contents: self.dirents.as_slice(),
            offset: 0,
        }
    }
}

pub struct IterDir<'a> {
    contents: &'a [u8],
    offset: isize,
}

impl<'a> Iterator for IterDir<'a> {
    type Item = DirEntry<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            if self.offset < self.contents.len() as isize {
                let raw_dirent =
                    &*(self.contents.as_ptr().offset(self.offset) as *const libc::dirent64);

                self.offset += raw_dirent.d_reclen as isize;

                Some(DirEntry { raw_dirent })
            } else {
                None
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (
            self.contents.len() / core::mem::size_of::<libc::dirent64>(),
            Some(self.contents.len() / (core::mem::size_of::<libc::dirent64>() - 256)),
        )
    }
}

// Storing just a reference in here instead of a CStr and a d_type makes this
// struct smaller and prevents calling strlen if the name is never required.
#[derive(Clone)]
pub struct DirEntry<'a> {
    raw_dirent: &'a libc::dirent64,
}

impl<'a> DirEntry<'a> {
    pub fn name(&self) -> CStr {
        unsafe { CStr::from_ptr(self.raw_dirent.d_name.as_ptr()) }
    }

    pub fn d_type(&self) -> DType {
        match self.raw_dirent.d_type {
            0 => DType::UNKNOWN,
            1 => DType::FIFO,
            2 => DType::CHR,
            4 => DType::DIR,
            6 => DType::BLK,
            8 => DType::REG,
            10 => DType::LNK,
            12 => DType::SOCK,
            _ => DType::UNKNOWN,
        }
    }
}

pub enum DType {
    UNKNOWN = 0,
    FIFO = 1,
    CHR = 2,
    DIR = 4,
    BLK = 6,
    REG = 8,
    LNK = 10,
    SOCK = 12,
}

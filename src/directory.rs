use crate::{syscalls, CStr};
use libc::c_int;

use alloc::{vec, vec::Vec};

pub struct Directory {
    fd: c_int,
}

impl<'a> Directory {
    pub fn open(path: CStr) -> Result<Self, crate::Error> {
        Ok(Self {
            fd: syscalls::open_dir(path)?,
        })
    }

    pub fn raw_fd(&self) -> c_int {
        self.fd
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

        dirents.truncate(bytes_used);

        Ok(DirectoryContents { dirents })
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

// Storing just a reference in here instead of an inode, d_type, and CStr makes
// this struct smaller and prevents calling strlen if the name is never required.
#[derive(Clone)]
pub struct DirEntry<'a> {
    raw_dirent: &'a libc::dirent64,
}

impl<'a> DirEntry<'a> {
    pub fn inode(&self) -> libc::c_ulong {
        self.raw_dirent.d_ino
    }

    pub fn name(&self) -> CStr {
        unsafe { CStr::from_ptr(self.raw_dirent.d_name.as_ptr()) }
    }

    pub fn name_ptr(&self) -> *const libc::c_char {
        self.raw_dirent.d_name.as_ptr()
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

#[derive(Clone, Copy, Debug)]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_cwd() {
        let dir = Directory::open(CStr::from_bytes(b".\0")).unwrap();
        let contents = dir.read().unwrap();

        let mut libc_dirents = Vec::new();
        unsafe {
            let dirp = libc::opendir(b".\0".as_ptr() as *const libc::c_char);
            let mut entry = libc::readdir64(dirp);
            while !entry.is_null() {
                libc_dirents.push(*entry);
                entry = libc::readdir64(dirp);
            }
        }

        for (libc, ven) in libc_dirents.iter().zip(contents.iter()) {
            unsafe {
                assert_eq!(CStr::from_ptr(libc.d_name.as_ptr()), ven.name());
            }
            assert_eq!(libc.d_ino, ven.inode());
            assert_eq!(libc.d_type, ven.d_type() as u8);
        }
    }
}

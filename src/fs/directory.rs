use crate::{
    syscalls,
    syscalls::{OpenFlags, OpenMode},
    CStr, Error,
};
use alloc::{vec, vec::Vec};
use core::convert::TryInto;
use libc::c_int;

pub struct Directory {
    fd: c_int,
}

impl Directory {
    #[inline]
    pub fn open(path: CStr) -> Result<Self, Error> {
        Ok(Self {
            fd: syscalls::openat(
                libc::AT_FDCWD,
                path,
                OpenFlags::RDONLY | OpenFlags::DIRECTORY | OpenFlags::CLOEXEC,
                OpenMode::empty(),
            )?,
        })
    }

    #[inline]
    pub fn raw_fd(&self) -> c_int {
        self.fd
    }

    #[inline]
    pub fn read(&self) -> Result<DirectoryContents, Error> {
        let mut contents = vec![0u8; 4096];

        // First, read using the first half of the allocation
        let mut previous_bytes_used = syscalls::getdents64(self.fd, &mut contents[..2048])?;
        let mut bytes_used = previous_bytes_used;

        // If we read something, try using the rest of the allocation
        if previous_bytes_used > 0 {
            bytes_used += syscalls::getdents64(self.fd, &mut contents[previous_bytes_used..])?;
        }
        // Then, if we read something on the second time, start reallocating.

        // Must run this loop until getdents64 returns no new entries
        // Yes, even if there is plenty of unused space. Some filesystems (at least sshfs) rely on this behavior
        while bytes_used != previous_bytes_used {
            previous_bytes_used = bytes_used;
            contents.extend(core::iter::repeat(0).take(contents.capacity()));
            bytes_used += syscalls::getdents64(self.fd, &mut contents[previous_bytes_used..])?;
        }

        contents.truncate(bytes_used);

        Ok(DirectoryContents { contents })
    }
}

impl Drop for Directory {
    #[inline]
    fn drop(&mut self) {
        let _ = syscalls::close(self.fd);
    }
}

pub struct DirectoryContents {
    contents: Vec<u8>,
}

impl DirectoryContents {
    #[inline]
    pub fn iter(&self) -> IterDir {
        IterDir {
            remaining: &self.contents[..],
        }
    }
}

pub struct IterDir<'a> {
    remaining: &'a [u8],
}

impl<'a> Iterator for IterDir<'a> {
    type Item = DirEntry<'a>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining.is_empty() {
            return None;
        }

        let inode = u64::from_ne_bytes(self.remaining[..8].try_into().unwrap());
        // We don't need to read the offset member
        let reclen = u16::from_ne_bytes(self.remaining[16..18].try_into().unwrap());
        let d_type = self.remaining[18];

        let mut end = 19;
        while self.remaining[end] != 0 {
            end += 1;
        }
        let name = CStr::from_bytes(&self.remaining[19..end + 1]);

        self.remaining = &self.remaining[reclen as usize..];

        Some(DirEntry {
            inode,
            name,
            d_type: match d_type {
                0 => DType::UNKNOWN,
                1 => DType::FIFO,
                2 => DType::CHR,
                4 => DType::DIR,
                6 => DType::BLK,
                8 => DType::REG,
                10 => DType::LNK,
                12 => DType::SOCK,
                _ => DType::UNKNOWN,
            },
        })
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (
            self.remaining.len() / core::mem::size_of::<libc::dirent64>(),
            Some(self.remaining.len() / (core::mem::size_of::<libc::dirent64>() - 256)),
        )
    }
}

#[derive(Clone)]
pub struct DirEntry<'a> {
    inode: libc::c_ulong,
    name: CStr<'a>,
    d_type: DType,
}

impl<'a> DirEntry<'a> {
    #[inline]
    pub fn name(&self) -> CStr<'a> {
        self.name
    }

    #[inline]
    pub fn inode(&self) -> libc::c_ulong {
        self.inode
    }

    #[inline]
    pub fn d_type(&self) -> DType {
        self.d_type
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(clippy::upper_case_acronyms)]
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
    use alloc::vec::Vec;

    // TODO: Get a directory with more file types

    #[test]
    fn read_cwd() {
        let dir = Directory::open(CStr::from_bytes(b"/dev\0")).unwrap();
        let contents = dir.read().unwrap();

        let mut libc_dirents = Vec::new();
        unsafe {
            let dirp = libc::opendir(b"/dev\0".as_ptr() as *const libc::c_char);
            let mut entry = libc::readdir64(dirp);
            while !entry.is_null() {
                libc_dirents.push(*entry);
                entry = libc::readdir64(dirp);
            }
            libc::closedir(dirp);
        }

        for (libc, ven) in libc_dirents.iter().zip(contents.iter()) {
            unsafe {
                assert_eq!(CStr::from_ptr(libc.d_name.as_ptr().cast()), ven.name);
            }
            assert_eq!(libc.d_ino, ven.inode);
            assert_eq!(libc.d_type, ven.d_type as u8);
        }
    }
}

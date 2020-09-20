use crate::{
    syscalls,
    syscalls::{OpenFlags, OpenMode},
    CStr,
};
use libc::c_int;

pub struct Directory {
    fd: c_int,
}

impl<'a> Directory {
    #[inline]
    pub fn open(path: CStr) -> Result<Self, crate::Error> {
        Ok(Self {
            fd: syscalls::open(
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
    pub fn read(&self) -> Result<DirectoryContents, crate::Error> {
        use alloc::alloc::{alloc, realloc, Layout};
        unsafe {
            let chunk_size = 32768;

            let mut layout = Layout::from_size_align_unchecked(
                chunk_size,
                core::mem::align_of::<libc::dirent64>(),
            );
            let mut ptr = alloc(layout);
            let mut bytes_used =
                syscalls::getdents64(self.fd, core::slice::from_raw_parts_mut(ptr, layout.size()))?;
            let mut previous_bytes_used = 0;

            // Must run this loop until getdents64 returns no new entries
            // Yes, it looks silly but some filesystems (at least sshfs) rely on this behavior
            while bytes_used != previous_bytes_used {
                previous_bytes_used = bytes_used;
                ptr = realloc(ptr, layout, layout.size() + chunk_size);
                layout =
                    Layout::from_size_align_unchecked(layout.size() + chunk_size, layout.align());
                bytes_used += syscalls::getdents64(
                    self.fd,
                    core::slice::from_raw_parts_mut(
                        ptr.add(bytes_used),
                        layout.size() - bytes_used,
                    ),
                )?;
            }

            Ok(DirectoryContents {
                ptr,
                len: bytes_used,
                layout,
            })
        }
    }
}

impl Drop for Directory {
    #[inline]
    fn drop(&mut self) {
        let _ = syscalls::close(self.fd);
    }
}

pub struct DirectoryContents {
    ptr: *const u8,
    len: usize,
    layout: alloc::alloc::Layout,
}

impl DirectoryContents {
    #[inline]
    pub fn iter(&self) -> IterDir {
        IterDir {
            contents: unsafe { core::slice::from_raw_parts(self.ptr, self.len) },
            offset: 0,
        }
    }
}

impl Drop for DirectoryContents {
    #[inline]
    fn drop(&mut self) {
        unsafe {
            alloc::alloc::dealloc(self.ptr as *mut u8, self.layout);
        }
    }
}

pub struct IterDir<'a> {
    contents: &'a [u8],
    offset: isize,
}

impl<'a> Iterator for IterDir<'a> {
    type Item = DirEntry<'a>;

    #[inline]
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

    #[inline]
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
    #[inline]
    pub fn inode(&self) -> libc::c_ulong {
        self.raw_dirent.d_ino
    }

    #[inline]
    pub fn name(&self) -> CStr {
        unsafe { CStr::from_ptr(self.raw_dirent.d_name.as_ptr()) }
    }

    #[inline]
    pub fn name_ptr(&self) -> *const libc::c_char {
        self.raw_dirent.d_name.as_ptr()
    }

    #[inline]
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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

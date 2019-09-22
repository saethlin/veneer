use core::{fmt, str};

#[derive(Clone, Copy, PartialEq)]
pub struct CStr<'a> {
    bytes: &'a [u8],
}

impl<'a> CStr<'a> {
    /// Requires that the passed-in pointer be a valid pointer to a null-terminated array of c_char
    pub unsafe fn from_ptr(ptr: *const libc::c_char) -> CStr<'a> {
        CStr {
            bytes: core::slice::from_raw_parts(ptr as *const u8, libc::strlen(ptr) + 1),
        }
    }

    pub fn from_bytes(bytes: &'a [u8]) -> CStr<'a> {
        assert!(
            bytes.last() == Some(&0),
            "attempted to construct a CStr from a slice without a null terminator"
        );
        CStr { bytes }
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes[..self.bytes.len() - 1]
    }

    pub fn get(&self, i: usize) -> Option<u8> {
        self.bytes.get(i).cloned()
    }

    pub fn as_ptr(&self) -> *const libc::c_char {
        self.bytes.as_ptr() as *const libc::c_char
    }
}

impl<'a> PartialEq<&[u8]> for CStr<'a> {
    fn eq(&self, bytes: &&[u8]) -> bool {
        if bytes.last() == Some(&0) {
            self.bytes == *bytes
        } else {
            &self.bytes[..self.bytes.len() - 1] == *bytes
        }
    }
}

impl<'a> fmt::Debug for CStr<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match str::from_utf8(self.as_bytes()) {
            Ok(s) => s.fmt(f),
            Err(e) => str::from_utf8(&self.as_bytes()[..e.valid_up_to()])
                .unwrap()
                .fmt(f),
        }
    }
}

impl<'a> fmt::Display for CStr<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match str::from_utf8(self.as_bytes()) {
            Ok(s) => s.fmt(f),
            Err(e) => str::from_utf8(&self.as_bytes()[..e.valid_up_to()])
                .unwrap()
                .fmt(f),
        }
    }
}

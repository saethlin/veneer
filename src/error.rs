pub struct Error(pub isize);

impl Error {
    pub fn msg(&self) -> &'static str {
        unsafe {
            let msg_ptr = libc::strerror(self.0 as libc::c_int);
            let bytes = core::slice::from_raw_parts(msg_ptr as *const u8, libc::strlen(msg_ptr));
            core::str::from_utf8_unchecked(bytes)
        }
    }
}

impl PartialEq<i32> for Error {
    fn eq(&self, other: &i32) -> bool {
        self.0 == *other as isize
    }
}

impl core::fmt::Debug for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "{}", self.msg())
    }
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "{}", self.msg())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_messages() {
        for i in 1..41 {
            println!("{} {:?}", i, Error(i).msg());
        }
    }
}

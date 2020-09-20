use crate::Error;

pub trait Read {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Error>;
}

pub trait Write {
    fn write(&mut self, buf: &[u8]) -> Result<usize, Error>;

    #[inline]
    fn write_all(&mut self, mut buf: &[u8]) -> Result<(), Error> {
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

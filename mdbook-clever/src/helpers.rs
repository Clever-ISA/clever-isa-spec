use std::{fmt::Write, io};

pub struct StringAppender<'a>(pub &'a mut String);

impl<'a> io::Write for StringAppender<'a> {
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }

    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match core::str::from_utf8(buf) {
            Ok(st) => {
                self.0.push_str(st);
                Ok(st.len())
            }
            Err(e) => Err(io::Error::new(io::ErrorKind::InvalidData, e)),
        }
    }

    fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
        match core::str::from_utf8(buf) {
            Ok(st) => Ok(self.0.push_str(st)),
            Err(e) => Err(io::Error::new(io::ErrorKind::InvalidData, e)),
        }
    }

    fn write_fmt(&mut self, fmt: std::fmt::Arguments<'_>) -> io::Result<()> {
        self.0
            .write_fmt(fmt)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))
    }
}

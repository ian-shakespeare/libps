use crate::Peek;
use std::io;

pub struct StringReadPeeker<'a> {
    value: &'a str,
    position: usize,
}

impl<'a> From<&'a str> for StringReadPeeker<'a> {
    fn from(value: &'a str) -> Self {
        Self { position: 0, value }
    }
}

impl Peek for StringReadPeeker<'_> {
    fn peek(&self, buf: &mut [u8]) -> io::Result<usize> {
        let mut cursor = 0;
        for ch in self.value.bytes().skip(self.position) {
            if cursor >= buf.len() {
                break;
            }

            buf[cursor] = ch;

            cursor += 1;
        }
        let write_count = cursor;

        if write_count < buf.len() {
            Err(io::Error::from(io::ErrorKind::UnexpectedEof))
        } else {
            Ok(write_count)
        }
    }
}

impl io::Read for StringReadPeeker<'_> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let write_count = self.peek(buf)?;
        self.position += write_count;
        Ok(write_count)
    }
}

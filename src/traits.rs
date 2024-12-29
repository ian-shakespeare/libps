use std::io::{Error, Read};

pub struct StringReader<'a> {
    s: &'a str,
    position: usize,
}

impl<'a> From<&'a str> for StringReader<'a> {
    fn from(value: &'a str) -> Self {
        Self {
            s: value,
            position: 0,
        }
    }
}

impl Read for StringReader<'_> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        for i in 0..buf.len() {
            match self.s.bytes().skip(self.position + i).next() {
                Some(ch) => buf[i] = ch,
                None => {
                    return Err(Error::new(
                        std::io::ErrorKind::UnexpectedEof,
                        "end of string",
                    ))
                }
            }

            self.position += 1;
        }

        Ok(buf.len())
    }
}

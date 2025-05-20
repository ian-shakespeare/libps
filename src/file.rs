use std::io;

use crate::object::Mode;

#[derive(Clone)]
pub struct FileObject {
    cursor: usize,
    inner: Vec<u8>,
    pub(crate) mode: Mode,
}

impl io::Read for FileObject {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let mut count = 0;

        for (ch, slot) in self.inner.iter().skip(self.cursor).zip(buf) {
            *slot = *ch;
            count += 1;
        }

        self.cursor += count;
        Ok(count)
    }
}

impl io::Seek for FileObject {
    fn seek(&mut self, pos: io::SeekFrom) -> io::Result<u64> {
        match pos {
            io::SeekFrom::Current(offset) => {
                let index = self
                    .cursor
                    .checked_add_signed(offset as isize)
                    .ok_or(io::Error::from(io::ErrorKind::NotSeekable))?;

                if index >= self.inner.len() {
                    return Err(io::Error::from(io::ErrorKind::NotSeekable));
                }
                self.cursor = index;

                Ok(self.cursor as u64)
            },
            io::SeekFrom::End(offset) => {
                let index = self
                    .inner
                    .len()
                    .checked_add_signed(offset as isize)
                    .ok_or(io::Error::from(io::ErrorKind::NotSeekable))?;

                if index >= self.inner.len() {
                    return Err(io::Error::from(io::ErrorKind::NotSeekable));
                }
                self.cursor = index;

                Ok(self.cursor as u64)
            },
            io::SeekFrom::Start(offset) => {
                let index = offset as usize;

                if index >= self.inner.len() {
                    return Err(io::Error::from(io::ErrorKind::NotSeekable));
                }
                self.cursor = index;

                Ok(offset)
            },
        }
    }
}

impl io::Write for FileObject {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.inner.extend_from_slice(buf);

        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl From<String> for FileObject {
    fn from(value: String) -> Self {
        Self {
            cursor: 0,
            inner: value.bytes().collect(),
            mode: Mode::default(),
        }
    }
}

impl From<Vec<u8>> for FileObject {
    fn from(value: Vec<u8>) -> Self {
        Self {
            cursor: 0,
            inner: value,
            mode: Mode::default(),
        }
    }
}

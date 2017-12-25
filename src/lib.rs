extern crate memchr;

use std::io::{Read, Result};
use std::cmp::min;
use memchr::memchr;

static DEFAULT_BUF_SIZE: usize = 8 * 1024;

#[derive(Debug, Eq, PartialEq, PartialOrd)]
pub enum Line {
    Return(usize),
    More(usize),
}

impl Line {
    pub fn amount(&self) -> usize {
        match *self {
            Line::Return(n) => n,
            Line::More(n) => n,
        }
    }
}

pub struct LineReader<R> {
    inner: R,
    buf: Box<[u8]>,
    pos: usize,
    cap: usize,
}

impl<R: Read> LineReader<R> {
    pub fn new(inner: R) -> LineReader<R> {
        LineReader::with_capacity(DEFAULT_BUF_SIZE, inner)
    }

    pub fn with_capacity(capacity: usize, inner: R) -> LineReader<R> {
        unsafe {
            let mut buf = Vec::with_capacity(capacity);
            buf.set_len(capacity);
            LineReader {
                inner,
                buf: buf.into_boxed_slice(),
                pos: 0,
                cap: 0,
            }
        }
    }

    fn fill_buf(&mut self) -> Result<&[u8]> {
        if self.pos >= self.cap {
            self.cap = self.inner.read(&mut self.buf)?;
            self.pos = 0;
        }
        Ok(&mut self.buf[self.pos..self.cap])
    }

    fn read_buf(&mut self, buf: &mut [u8]) -> Result<Line> {
        let line = {
            self.fill_buf()?;
            if self.pos == self.cap {
                return Ok(Line::Return(0));
            }

            let mut rem = self.fill_buf()?;

            if buf.len() < rem.len() {
                rem = &rem[..buf.len()];
            }

            match memchr(b'\n', rem) {
                Some(n) => Line::Return((&rem[..n + 1]).read(buf)?),
                None => Line::More(rem.read(buf)?),
            }
        };

        self.consume(line.amount());
        Ok(line)
    }

    pub fn try_read_line(&mut self, buf: &mut [u8]) -> Result<Line> {
        let mut amt: usize = 0;
        let cap = buf.len();

        while amt < cap {
            match self.read_buf(&mut buf[amt..])? {
                Line::Return(n) => {
                    return Ok(Line::Return(amt + n));
                }
                Line::More(n) => {
                    amt += n;
                }
            }
        }
        Ok(Line::More(amt))
    }

    fn consume(&mut self, n: usize) {
        self.pos = min(self.pos + n, self.cap);
    }
}



#[cfg(test)]
mod tests {
    use Line;
    use LineReader;
    use std::io::Cursor;

    #[test]
    fn try_read_line() {
        let cur = Cursor::new(b"12\n345");
        let mut reader = LineReader::new(cur.clone());
        let mut buf = vec![0; 1];
        assert_eq!(reader.try_read_line(&mut buf).ok(), Some(Line::More(1)));
        assert_eq!(reader.try_read_line(&mut buf).ok(), Some(Line::More(1)));
        assert_eq!(reader.try_read_line(&mut buf).ok(), Some(Line::Return(1)));
        assert_eq!(reader.try_read_line(&mut buf).ok(), Some(Line::More(1)));
        assert_eq!(reader.try_read_line(&mut buf).ok(), Some(Line::More(1)));
        assert_eq!(reader.try_read_line(&mut buf).ok(), Some(Line::More(1)));
        assert_eq!(reader.try_read_line(&mut buf).ok(), Some(Line::Return(0)));

        let mut reader = LineReader::new(cur.clone());
        let mut buf = vec![0; 2];
        assert_eq!(reader.try_read_line(&mut buf).ok(), Some(Line::More(2)));
        assert_eq!(reader.try_read_line(&mut buf).ok(), Some(Line::Return(1)));
        assert_eq!(reader.try_read_line(&mut buf).ok(), Some(Line::More(2)));
        assert_eq!(reader.try_read_line(&mut buf).ok(), Some(Line::Return(1)));

        let mut reader = LineReader::new(cur.clone());
        let mut buf = vec![0; 4];
        assert_eq!(reader.try_read_line(&mut buf).ok(), Some(Line::Return(3)));
        assert_eq!(reader.try_read_line(&mut buf).ok(), Some(Line::Return(3)));
    }
}

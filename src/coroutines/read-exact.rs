//! Module dedicated to the [`Read`] I/O-free coroutine.

use log::debug;

use crate::Io;

use super::Read;

/// I/O-free coroutine for reading bytes into a buffer until it
/// reaches EOF.
#[derive(Debug)]
pub struct ReadExact {
    read: Read,
    count: usize,
    buffer: Option<Vec<u8>>,
}

impl ReadExact {
    pub fn new(bytes_count: usize) -> Self {
        Self {
            read: Read::with_capacity(Read::DEFAULT_CAPACITY.min(bytes_count)),
            count: bytes_count,
            buffer: Some(Vec::new()),
        }
    }

    /// Creates a new read coroutine with the given buffer capacity.
    pub fn with_capacity(capacity: usize, bytes_count: usize) -> Self {
        Self {
            read: Read::with_capacity(capacity),
            count: bytes_count,
            buffer: Some(Vec::new()),
        }
    }

    pub fn extend(&mut self, bytes: impl IntoIterator<Item = u8>) {
        let bytes: Vec<u8> = bytes.into_iter().collect();
        self.count -= bytes.len().min(self.count);

        let Some(buffer) = &mut self.buffer else {
            self.buffer.replace(bytes);
            return;
        };

        buffer.extend(bytes);
    }

    /// Makes the read progress.
    pub fn resume(&mut self, mut arg: Option<Io>) -> Result<Vec<u8>, Io> {
        loop {
            let Some(buffer) = &mut self.buffer else {
                return Err(Io::err("read exact buffer not ready"));
            };

            if self.count == 0 {
                break Ok(self.buffer.take().unwrap());
            }

            if self.count < self.read.capacity() {
                self.read = Read::with_capacity(self.count);
            }

            let output = self.read.resume(arg.take())?;

            if output.bytes_count == 0 {
                debug!("expected {} more bytes, got unexpected EOF", self.count);
                break Err(Io::err("read 0 bytes, unexpected EOF?"));
            }

            buffer.extend(output.bytes());
            self.count -= self.read.capacity();
            self.read.replace(output.buffer);
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io::{BufReader, Read as _};

    use crate::{Io, Output};

    use super::ReadExact;

    #[test]
    fn read_exact_smaller_capacity() {
        let mut reader = BufReader::new("abcdef".as_bytes());

        let mut read = ReadExact::with_capacity(3, 4);
        let mut arg = None;

        let output = loop {
            match read.resume(arg.take()) {
                Ok(output) => break output,
                Err(Io::Read(Err(mut buffer))) => {
                    let bytes_count = reader.read(&mut buffer).unwrap();
                    let output = Output {
                        buffer,
                        bytes_count,
                    };
                    arg = Some(Io::Read(Ok(output)))
                }
                Err(io) => unreachable!("unexpected I/O: {io:?}"),
            }
        };

        assert_eq!(output, b"abcd");

        let mut remaining = vec![0; 4];
        let bytes_count = reader.read(&mut remaining).unwrap();

        assert_eq!(bytes_count, 2);
        assert_eq!(&remaining[..bytes_count], b"ef");
    }

    #[test]
    fn read_exact_bigger_capacity() {
        let mut reader = BufReader::new("abcdef".as_bytes());

        let mut read = ReadExact::with_capacity(5, 4);
        let mut arg = None;

        let output = loop {
            match read.resume(arg.take()) {
                Ok(output) => break output,
                Err(Io::Read(Err(mut buffer))) => {
                    let bytes_count = reader.read(&mut buffer).unwrap();
                    let output = Output {
                        buffer,
                        bytes_count,
                    };
                    arg = Some(Io::Read(Ok(output)))
                }
                Err(io) => unreachable!("unexpected I/O: {io:?}"),
            }
        };

        assert_eq!(output, b"abcd");

        let mut remaining = vec![0; 4];
        let bytes_count = reader.read(&mut remaining).unwrap();

        assert_eq!(bytes_count, 2);
        assert_eq!(&remaining[..bytes_count], b"ef");
    }
}

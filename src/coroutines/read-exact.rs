use log::{debug, trace};

use crate::Io;

use super::Read;

/// I/O-free coroutine for reading bytes into a buffer until it
/// reaches a given amount of bytes.
#[derive(Debug)]
pub struct ReadExact {
    /// The inner read coroutine.
    read: Read,

    /// The exact amount of bytes to read.
    max: usize,

    /// The buffer containing the read bytes.
    buffer: Option<Vec<u8>>,
}

impl ReadExact {
    /// Creates a new coroutine to read bytes using a buffer with
    /// [`Read::DEFAULT_CAPACITY`] capacity.
    ///
    /// See [`Self::with_capacity`] for a custom buffer capacity.
    pub fn new(max: usize) -> Self {
        Self::with_capacity(Read::DEFAULT_CAPACITY, max)
    }

    /// Creates a new coroutine to read bytes using a buffer with the
    /// given capacity.
    pub fn with_capacity(capacity: usize, max: usize) -> Self {
        Self {
            read: Read::with_capacity(capacity.min(max)),
            max,
            buffer: Some(Vec::with_capacity(max)),
        }
    }

    /// Adds the given bytes the to inner buffer.
    pub fn extend(&mut self, bytes: impl IntoIterator<Item = u8>) {
        let Some(buffer) = &mut self.buffer else {
            self.buffer.replace(bytes.into_iter().collect());
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

            if buffer.len() >= self.max {
                // SAFETY: buffer exists due to check above
                break Ok(self.buffer.take().unwrap());
            }

            let remaining = self.max - buffer.len();
            trace!("{remaining} remaining bytes to read");

            if remaining < self.read.capacity() {
                self.read = Read::with_capacity(remaining);
            }

            let output = self.read.resume(arg.take())?;

            if output.bytes_count == 0 {
                debug!("expected {remaining} more bytes, got unexpected EOF");
                break Err(Io::err("Read 0 bytes, unexpected EOF?"));
            }

            buffer.extend(output.bytes());
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
        let _ = env_logger::try_init();

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
        let _ = env_logger::try_init();

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

    #[test]
    fn read_exact_0() {
        let _ = env_logger::try_init();

        let mut reader = BufReader::new("abcdef".as_bytes());

        let mut read = ReadExact::with_capacity(5, 0);
        read.extend(b"123".iter().cloned());

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

        assert_eq!(output, b"123");
    }
}

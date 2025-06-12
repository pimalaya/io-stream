use std::mem;

use log::{debug, trace};
use thiserror::Error;

use crate::{coroutines::read::ReadResult, io::StreamIo};

use super::read::{Read, ReadError};

#[derive(Clone, Debug, Error)]
pub enum ReadExactError {
    #[error("Unexpected EOF, expected to read {0}/{1} more bytes")]
    UnexpectedEof(usize, usize, Vec<u8>),
    #[error(transparent)]
    Read(#[from] ReadError),
}

#[derive(Clone, Debug)]
pub enum ReadExactResult {
    Ok(Vec<u8>),
    Err(ReadExactError),
    Io(StreamIo),
}

/// I/O-free coroutine for reading bytes into a buffer until it
/// reaches a given amount of bytes.
#[derive(Debug)]
pub struct ReadExact {
    /// The inner read coroutine.
    read: Read,

    /// The buffer containing the final read bytes.
    buffer: Vec<u8>,

    /// The exact amount of bytes to read.
    max: usize,
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
        trace!("init coroutine for reading exactly {max} bytes (capacity: {capacity})");
        let read = Read::with_capacity(capacity.min(max));
        let buffer = Vec::with_capacity(max);
        Self { read, buffer, max }
    }

    /// Extends the inner buffer with the given bytes slice.
    pub fn extend(&mut self, bytes: impl IntoIterator<Item = u8>) {
        self.buffer.extend(bytes);
    }

    pub fn resume(&mut self, mut arg: Option<StreamIo>) -> ReadExactResult {
        loop {
            if self.buffer.len() >= self.max {
                let buffer = mem::take(&mut self.buffer);
                break ReadExactResult::Ok(buffer);
            }

            let remaining = self.max - self.buffer.len();
            debug!("{remaining} remaining bytes to read");

            if remaining < self.read.capacity() {
                self.read.truncate(remaining);
            }

            let output = match self.read.resume(arg.take()) {
                ReadResult::Ok(output) => output,
                ReadResult::Err(err) => break ReadExactResult::Err(err.into()),
                ReadResult::Io(io) => break ReadExactResult::Io(io),
                ReadResult::Eof => {
                    let buffer = mem::take(&mut self.buffer);
                    let err = ReadExactError::UnexpectedEof(remaining, self.max, buffer);
                    break ReadExactResult::Err(err);
                }
            };

            self.buffer.extend(output.bytes());
            self.read.replace(output.buffer);
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io::{BufReader, Read as _};

    use crate::{
        coroutines::read_exact::{ReadExactError, ReadExactResult},
        io::{StreamIo, StreamOutput},
    };

    use super::ReadExact;

    #[test]
    fn read_exact_smaller_capacity() {
        let _ = env_logger::try_init();

        let mut reader = BufReader::new("abcdef".as_bytes());

        let mut read = ReadExact::with_capacity(3, 4);
        let mut arg = None;

        let output = loop {
            match read.resume(arg.take()) {
                ReadExactResult::Ok(output) => break output,
                ReadExactResult::Io(StreamIo::Read(Err(mut buffer))) => {
                    let bytes_count = reader.read(&mut buffer).unwrap();
                    let output = StreamOutput {
                        buffer,
                        bytes_count,
                    };
                    arg = Some(StreamIo::Read(Ok(output)))
                }
                other => unreachable!("Unexpected result: {other:?}"),
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
                ReadExactResult::Ok(output) => break output,
                ReadExactResult::Io(StreamIo::Read(Err(mut buffer))) => {
                    let bytes_count = reader.read(&mut buffer).unwrap();
                    let output = StreamOutput {
                        buffer,
                        bytes_count,
                    };
                    arg = Some(StreamIo::Read(Ok(output)))
                }
                other => unreachable!("Unexpected result: {other:?}"),
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
        read.extend("123".as_bytes().to_vec());

        let mut arg = None;

        let output = loop {
            match read.resume(arg.take()) {
                ReadExactResult::Ok(output) => break output,
                ReadExactResult::Io(StreamIo::Read(Err(mut buffer))) => {
                    let bytes_count = reader.read(&mut buffer).unwrap();
                    let output = StreamOutput {
                        buffer,
                        bytes_count,
                    };
                    arg = Some(StreamIo::Read(Ok(output)))
                }
                other => unreachable!("Unexpected result: {other:?}"),
            }
        };

        assert_eq!(output, b"123");
    }

    #[test]
    fn read_eof() {
        let _ = env_logger::try_init();

        let mut reader = BufReader::new("abcdef".as_bytes());

        let mut read = ReadExact::new(8);
        let mut arg = None;

        loop {
            match read.resume(arg.take()) {
                ReadExactResult::Err(ReadExactError::UnexpectedEof(2, 8, output)) => {
                    break assert_eq!(output, b"abcdef");
                }
                ReadExactResult::Io(StreamIo::Read(Err(mut buffer))) => {
                    let bytes_count = reader.read(&mut buffer).unwrap();
                    let output = StreamOutput {
                        buffer,
                        bytes_count,
                    };
                    arg = Some(StreamIo::Read(Ok(output)))
                }
                other => unreachable!("Unexpected result: {other:?}"),
            }
        }
    }
}

use std::mem;

use log::{debug, trace};
use thiserror::Error;

use crate::io::{StreamIo, StreamOutput};

#[derive(Clone, Debug, Error)]
pub enum ReadError {
    #[error("Invalid argument: expected {0}, got {1:?}")]
    InvalidArgument(&'static str, StreamIo),
}

#[derive(Clone, Debug)]
pub enum ReadResult {
    Ok(StreamOutput),
    Err(ReadError),
    Io(StreamIo),
    Eof,
}

/// I/O-free coroutine for reading bytes into a buffer.
#[derive(Debug)]
pub struct Read {
    buffer: Vec<u8>,
}

impl Read {
    /// The default read buffer capacity.
    pub const DEFAULT_CAPACITY: usize = 8 * 1024;

    /// Creates a new coroutine to read bytes using a buffer with
    /// [`Self::DEFAULT_CAPACITY`] capacity.
    ///
    /// See [`Self::with_capacity`] for a custom buffer capacity.
    pub fn new() -> Self {
        Self::with_capacity(Self::DEFAULT_CAPACITY)
    }

    /// Creates a new coroutine to read bytes using a buffer with the
    /// given capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        trace!("init coroutine for reading bytes (capacity: {capacity})");
        let buffer = vec![0; capacity];
        Self { buffer }
    }

    /// Returns the buffer capacity.
    ///
    /// This function does not return directly the capacity of the
    /// buffer, it returns instead the initial capacity the coroutine
    /// was created with.
    pub fn capacity(&self) -> usize {
        self.buffer.capacity()
    }

    /// Shortens the buffer to the given length.
    pub fn truncate(&mut self, len: usize) {
        self.buffer.truncate(len);
        self.buffer.shrink_to(len);
    }

    /// Replaces the inner buffer with the given one.
    pub fn replace(&mut self, mut buffer: Vec<u8>) {
        buffer.fill(0);
        self.buffer = buffer;
    }

    /// Makes the read progress.
    pub fn resume(&mut self, arg: Option<StreamIo>) -> ReadResult {
        let Some(arg) = arg else {
            let mut buffer = vec![0; self.buffer.capacity()];
            mem::swap(&mut buffer, &mut self.buffer);
            trace!("wants I/O to read bytes");
            return ReadResult::Io(StreamIo::Read(Err(buffer)));
        };

        trace!("resume after reading bytes");

        let StreamIo::Read(io) = arg else {
            return ReadResult::Err(ReadError::InvalidArgument("read output", arg));
        };

        let output = match io {
            Ok(output) => output,
            Err(buffer) => return ReadResult::Io(StreamIo::Read(Err(buffer))),
        };

        match output.bytes_count {
            0 => ReadResult::Eof,
            n => {
                debug!("read {n}/{} bytes", output.buffer.capacity());
                ReadResult::Ok(output)
            }
        }
    }
}

impl Default for Read {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use std::io::{BufReader, Read as _};

    use crate::{
        coroutines::read::ReadResult,
        io::{StreamIo, StreamOutput},
    };

    use super::Read;

    #[test]
    fn read() {
        let _ = env_logger::try_init();

        let mut reader = BufReader::new("abcdef".as_bytes());

        let mut read = Read::with_capacity(4);
        let mut arg = None;

        let output = loop {
            match read.resume(arg.take()) {
                ReadResult::Ok(output) => break output,
                ReadResult::Io(StreamIo::Read(Err(mut buffer))) => {
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

        assert_eq!(output.bytes(), b"abcd");

        read.replace(output.buffer);

        let output = loop {
            match read.resume(arg.take()) {
                ReadResult::Ok(output) => break output,
                ReadResult::Io(StreamIo::Read(Err(mut buffer))) => {
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

        assert_eq!(output.bytes(), b"ef");

        read.replace(output.buffer);

        loop {
            match read.resume(arg.take()) {
                ReadResult::Eof => break,
                ReadResult::Io(StreamIo::Read(Err(mut buffer))) => {
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

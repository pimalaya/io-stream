//! I/O-free coroutine to read bytes into a buffer.

use std::mem;

use log::{debug, trace};
use thiserror::Error;

use crate::io::{StreamIo, StreamOutput};

/// Errors that can occur during the coroutine progression.
#[derive(Clone, Debug, Error)]
pub enum ReadStreamError {
    /// The coroutine received an invalid argument.
    ///
    /// Occurs when the coroutine receives an I/O response from
    /// another coroutine, which should not happen if the runtime maps
    /// correctly the arguments.
    #[error("Invalid argument: expected {0}, got {1:?}")]
    InvalidArgument(&'static str, StreamIo),
}

/// Output emitted after a coroutine finishes its progression.
#[derive(Clone, Debug)]
pub enum ReadStreamResult {
    /// The coroutine has successfully terminated its progression.
    Ok(StreamOutput),

    /// A stream I/O needs to be performed to make the coroutine
    /// progress.
    Io(StreamIo),

    /// The coroutine reached the End Of File.
    ///
    /// Only the consumer can determine if its an error or not.
    Eof,

    /// An error occured during the coroutine progression.
    Err(ReadStreamError),
}

/// I/O-free coroutine to read bytes into a buffer.
#[derive(Debug)]
pub struct ReadStream {
    buffer: Vec<u8>,
}

impl ReadStream {
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
        trace!("init coroutine to read bytes (capacity: {capacity})");
        let buffer = vec![0; capacity];
        Self { buffer }
    }

    /// Returns the buffer capacity.
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
    pub fn resume(&mut self, arg: Option<StreamIo>) -> ReadStreamResult {
        let Some(arg) = arg else {
            let mut buffer = vec![0; self.buffer.capacity()];
            mem::swap(&mut buffer, &mut self.buffer);
            trace!("wants I/O to read bytes");
            return ReadStreamResult::Io(StreamIo::Read(Err(buffer)));
        };

        trace!("resume after reading bytes");

        let StreamIo::Read(io) = arg else {
            return ReadStreamResult::Err(ReadStreamError::InvalidArgument("read output", arg));
        };

        let output = match io {
            Ok(output) => output,
            Err(buffer) => return ReadStreamResult::Io(StreamIo::Read(Err(buffer))),
        };

        match output.bytes_count {
            0 => ReadStreamResult::Eof,
            n => {
                debug!("read {n}/{} bytes", output.buffer.capacity());
                ReadStreamResult::Ok(output)
            }
        }
    }
}

impl Default for ReadStream {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use std::io::{BufReader, Read as _};

    use crate::{
        coroutines::read::ReadStreamResult,
        io::{StreamIo, StreamOutput},
    };

    use super::ReadStream;

    #[test]
    fn read() {
        let _ = env_logger::try_init();

        let mut reader = BufReader::new("abcdef".as_bytes());

        let mut read = ReadStream::with_capacity(4);
        let mut arg = None;

        let output = loop {
            match read.resume(arg.take()) {
                ReadStreamResult::Ok(output) => break output,
                ReadStreamResult::Io(StreamIo::Read(Err(mut buffer))) => {
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
                ReadStreamResult::Ok(output) => break output,
                ReadStreamResult::Io(StreamIo::Read(Err(mut buffer))) => {
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
                ReadStreamResult::Eof => break,
                ReadStreamResult::Io(StreamIo::Read(Err(mut buffer))) => {
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

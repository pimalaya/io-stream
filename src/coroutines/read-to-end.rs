//! I/O-free coroutine to read bytes into a buffer until it reaches
//! EOF.

use std::mem;

use log::trace;
use thiserror::Error;

use crate::io::StreamIo;

use super::read::{ReadStream, ReadStreamError, ReadStreamResult};

/// Errors that can occur during the coroutine progression.
#[derive(Clone, Debug, Error)]
pub enum ReadStreamToEndError {
    /// Error from the [`Read`] coroutine.
    #[error(transparent)]
    Read(#[from] ReadStreamError),
}

/// Output emitted after a coroutine finishes its progression.
#[derive(Clone, Debug)]
pub enum ReadStreamToEndResult {
    /// The coroutine has successfully terminated its progression.
    Ok(Vec<u8>),

    /// A stream I/O needs to be performed to make the coroutine
    /// progress.
    Io(StreamIo),

    /// An error occured during the coroutine progression.
    Err(ReadStreamToEndError),
}

/// I/O-free coroutine to read bytes into a buffer until it reaches
/// EOF.
#[derive(Debug)]
pub struct ReadStreamToEnd {
    /// The inner read coroutine.
    read: ReadStream,

    /// The buffer containing the read bytes.
    buffer: Vec<u8>,
}

impl ReadStreamToEnd {
    /// Creates a new coroutine to read bytes using a buffer with
    /// [`Read::DEFAULT_CAPACITY`] capacity.
    ///
    /// See [`Self::with_capacity`] for a custom buffer capacity.
    pub fn new() -> Self {
        Self::with_capacity(ReadStream::DEFAULT_CAPACITY)
    }

    /// Creates a new coroutine to read bytes using a buffer with the
    /// given capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        trace!("init coroutine to read until EOF (capacity: {capacity})");
        let read = ReadStream::with_capacity(capacity);
        let buffer = Vec::with_capacity(capacity);
        Self { read, buffer }
    }

    /// Extends the inner buffer with the given bytes slice.
    pub fn extend(&mut self, bytes: impl IntoIterator<Item = u8>) {
        self.buffer.extend(bytes);
    }

    /// Makes the coroutine progress.
    pub fn resume(&mut self, mut arg: Option<StreamIo>) -> ReadStreamToEndResult {
        loop {
            let output = match self.read.resume(arg.take()) {
                ReadStreamResult::Ok(output) => output,
                ReadStreamResult::Err(err) => break ReadStreamToEndResult::Err(err.into()),
                ReadStreamResult::Io(io) => break ReadStreamToEndResult::Io(io),
                ReadStreamResult::Eof => {
                    let buffer = mem::take(&mut self.buffer);
                    break ReadStreamToEndResult::Ok(buffer);
                }
            };

            self.buffer.extend(output.bytes());
            self.read.replace(output.buffer);
        }
    }
}

impl Default for ReadStreamToEnd {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use std::io::{BufReader, Read as _};

    use crate::{
        coroutines::read_to_end::ReadStreamToEndResult,
        io::{StreamIo, StreamOutput},
    };

    use super::ReadStreamToEnd;

    #[test]
    fn read_to_end() {
        let _ = env_logger::try_init();

        let mut reader = BufReader::new("abcdef".as_bytes());

        let mut read = ReadStreamToEnd::with_capacity(4);
        let mut arg = None;

        let output = loop {
            match read.resume(arg.take()) {
                ReadStreamToEndResult::Ok(output) => break output,
                ReadStreamToEndResult::Io(StreamIo::Read(Err(mut buffer))) => {
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

        assert_eq!(output, b"abcdef");
    }
}

use std::mem;

use log::trace;

use crate::io::StreamIo;

use super::read::{Read, ReadError, ReadResult};

#[derive(Clone, Debug)]
pub enum ReadToEndResult {
    Ok(Vec<u8>),
    Err(ReadError),
    Io(StreamIo),
}

/// I/O-free coroutine for reading bytes into a buffer until it
/// reaches EOF.
#[derive(Debug)]
pub struct ReadToEnd {
    /// The inner read coroutine.
    read: Read,

    /// The buffer containing the read bytes.
    buffer: Vec<u8>,
}

impl ReadToEnd {
    /// Creates a new coroutine to read bytes using a buffer with
    /// [`Read::DEFAULT_CAPACITY`] capacity.
    ///
    /// See [`Self::with_capacity`] for a custom buffer capacity.
    pub fn new() -> Self {
        Self::with_capacity(Read::DEFAULT_CAPACITY)
    }

    /// Creates a new coroutine to read bytes using a buffer with the
    /// given capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        trace!("init coroutine for reading until EOF (capacity: {capacity})");
        let read = Read::with_capacity(capacity);
        let buffer = Vec::with_capacity(capacity);
        Self { read, buffer }
    }

    /// Extends the inner buffer with the given bytes slice.
    pub fn extend(&mut self, bytes: impl IntoIterator<Item = u8>) {
        self.buffer.extend(bytes);
    }

    pub fn resume(&mut self, mut arg: Option<StreamIo>) -> ReadToEndResult {
        loop {
            let output = match self.read.resume(arg.take()) {
                ReadResult::Ok(output) => output,
                ReadResult::Err(err) => break ReadToEndResult::Err(err.into()),
                ReadResult::Io(io) => break ReadToEndResult::Io(io),
                ReadResult::Eof => {
                    let buffer = mem::take(&mut self.buffer);
                    break ReadToEndResult::Ok(buffer);
                }
            };

            self.buffer.extend(output.bytes());
            self.read.replace(output.buffer);
        }
    }
}

impl Default for ReadToEnd {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use std::io::{BufReader, Read as _};

    use crate::{
        coroutines::read_to_end::ReadToEndResult,
        io::{StreamIo, StreamOutput},
    };

    use super::ReadToEnd;

    #[test]
    fn read_to_end() {
        let _ = env_logger::try_init();

        let mut reader = BufReader::new("abcdef".as_bytes());

        let mut read = ReadToEnd::with_capacity(4);
        let mut arg = None;

        let output = loop {
            match read.resume(arg.take()) {
                ReadToEndResult::Ok(output) => break output,
                ReadToEndResult::Io(StreamIo::Read(Err(mut buffer))) => {
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

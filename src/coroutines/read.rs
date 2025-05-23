//! Module dedicated to the [`Read`] I/O-free coroutine.

use log::debug;

use crate::{Io, Output};

/// I/O-free coroutine for reading bytes into a buffer.
#[derive(Debug)]
pub struct Read {
    capacity: usize,
    buffer: Option<Vec<u8>>,
}

impl Read {
    pub const DEFAULT_CAPACITY: usize = 1024;

    /// Creates a new coroutine to read bytes with a buffer capacity
    /// of 1024.
    ///
    /// See [`Read::with_capacity`] for a different buffer capacity.
    pub fn new() -> Self {
        Self::with_capacity(Self::DEFAULT_CAPACITY)
    }

    /// Creates a new coroutine to read bytes with the given buffer
    /// capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        debug!("create read buffer of {capacity} capacity");
        let buffer = Some(vec![0; capacity]);
        Self { capacity, buffer }
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Replaces the inner buffer with the given one.
    pub fn take(&mut self) -> Option<Vec<u8>> {
        self.buffer.take()
    }

    /// Replaces the inner buffer with the given one.
    pub fn replace(&mut self, buffer: Vec<u8>) {
        let capacity = buffer.capacity();
        debug!("replace read buffer with {capacity} capacity");
        self.buffer.replace(buffer);
        self.capacity = capacity;
    }

    /// Makes the read progress.
    pub fn resume(&mut self, arg: Option<Io>) -> Result<Output, Io> {
        let Some(arg) = arg else {
            let Some(buffer) = self.buffer.take() else {
                return Err(Io::err("read buffer not initialized"));
            };

            debug!("break: need I/O to read bytes");
            return Err(Io::Read(Err(buffer)));
        };

        debug!("resume after reading bytes");

        let Io::Read(io) = arg else {
            let err = format!("expected read output, got {arg:?}");
            return Err(Io::err(err));
        };

        let output = match io {
            Ok(output) => output,
            Err(buffer) => return Err(Io::Read(Err(buffer))),
        };

        let n = output.bytes_count;
        let capacity = output.buffer.capacity();
        debug!("read {n}/{capacity} bytes");

        Ok(output)
    }
}

impl Default for Read {
    fn default() -> Self {
        Self::new()
    }
}

impl From<Vec<u8>> for Read {
    fn from(buffer: Vec<u8>) -> Self {
        Self {
            capacity: buffer.capacity(),
            buffer: Some(buffer),
        }
    }
}

impl From<Option<Vec<u8>>> for Read {
    fn from(buffer: Option<Vec<u8>>) -> Self {
        let Some(buffer) = buffer else {
            return Self {
                capacity: 0,
                buffer: None,
            };
        };

        Self::from(buffer)
    }
}

#[cfg(test)]
mod tests {
    use std::io::{BufReader, Read as _};

    use crate::{Io, Output};

    use super::Read;

    #[test]
    fn read() {
        let mut reader = BufReader::new("abcdef".as_bytes());

        let mut read = Read::with_capacity(4);
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

        assert_eq!(output.bytes(), b"abcd");

        read.replace(output.buffer);

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

        assert_eq!(output.bytes(), b"ef");

        read.replace(output.buffer);

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

        assert_eq!(output.bytes_count, 0);
    }
}

use log::{debug, trace};

use crate::{Io, Output};

/// I/O-free coroutine for reading bytes into a buffer.
#[derive(Debug)]
pub struct Read {
    capacity: usize,
    buffer: Option<Vec<u8>>,
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
        debug!("create read buffer of {capacity} capacity");
        let buffer = Some(vec![0; capacity]);
        Self { capacity, buffer }
    }

    /// Returns the buffer capacity.
    ///
    /// This function does not return directly the capacity of the
    /// buffer, it returns instead the initial capacity the coroutine
    /// was created with.
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Shrinks the buffer to the given capacity.
    pub fn shrink_to(&mut self, min_capacity: usize) {
        if let Some(buffer) = &mut self.buffer {
            buffer.shrink_to(min_capacity);
            self.capacity = buffer.capacity();
        } else {
            self.capacity = min_capacity;
        }
    }

    /// Replaces the inner buffer with the given one.
    pub fn replace(&mut self, mut buffer: Vec<u8>) {
        let capacity = buffer.capacity();
        trace!("replace read buffer with {capacity} capacity");
        buffer.fill(0);
        self.buffer.replace(buffer);
        self.capacity = capacity;
    }

    /// Makes the read progress.
    pub fn resume(&mut self, arg: Option<Io>) -> Result<Output, Io> {
        let Some(arg) = arg else {
            let Some(buffer) = self.buffer.take() else {
                return Err(Io::err("Read buffer not ready"));
            };

            trace!("break: need I/O to read bytes");
            return Err(Io::Read(Err(buffer)));
        };

        trace!("resume after reading bytes");

        let Io::Read(io) = arg else {
            let err = format!("Expected read output, got {arg:?}");
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

#[cfg(test)]
mod tests {
    use std::io::{BufReader, Read as _};

    use crate::{Io, Output};

    use super::Read;

    #[test]
    fn read() {
        let _ = env_logger::try_init();

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

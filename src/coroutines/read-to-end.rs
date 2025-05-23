use crate::Io;

use super::read::Read;

/// I/O-free coroutine for reading bytes into a buffer until it
/// reaches EOF.
#[derive(Debug)]
pub struct ReadToEnd {
    /// The inner read coroutine.
    read: Read,

    /// The buffer containing the read bytes.
    buffer: Option<Vec<u8>>,
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
        let read = Read::with_capacity(capacity);
        let buffer = Some(Vec::with_capacity(capacity));
        Self { read, buffer }
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
            let output = self.read.resume(arg.take())?;

            let Some(buffer) = &mut self.buffer else {
                break Err(Io::err("read to end buffer not ready"));
            };

            if output.bytes_count == 0 {
                // SAFETY: buffer exists due to check above
                break Ok(self.buffer.take().unwrap());
            }

            buffer.extend(output.bytes());
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

    use crate::{Io, Output};

    use super::ReadToEnd;

    #[test]
    fn read_to_end() {
        let _ = env_logger::try_init();

        let mut reader = BufReader::new("abcdef".as_bytes());

        let mut read = ReadToEnd::with_capacity(4);
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

        assert_eq!(output, b"abcdef");
    }
}

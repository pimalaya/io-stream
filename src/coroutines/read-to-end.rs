//! Module dedicated to the [`Read`] I/O-free coroutine.

use crate::Io;

use super::read::Read;

/// I/O-free coroutine for reading bytes into a buffer until it
/// reaches EOF.
#[derive(Debug)]
pub struct ReadToEnd {
    read: Read,
    buffer: Option<Vec<u8>>,
}

impl ReadToEnd {
    pub fn new() -> Self {
        Self {
            read: Read::new(),
            buffer: Some(Vec::new()),
        }
    }

    /// Creates a new read coroutine with the given buffer capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            read: Read::with_capacity(capacity),
            buffer: Some(Vec::new()),
        }
    }

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

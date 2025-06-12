use log::{debug, trace};
use thiserror::Error;

use crate::io::{StreamIo, StreamOutput};

#[derive(Clone, Debug, Error)]
pub enum WriteError {
    #[error("Invalid argument: expected {0}, got {1:?}")]
    InvalidArgument(&'static str, StreamIo),
}

#[derive(Clone, Debug)]
pub enum WriteResult {
    Ok(StreamOutput),
    Err(WriteError),
    Io(StreamIo),
    Eof,
}

/// I/O-free coroutine for writing bytes into a stream.
#[derive(Debug, Default)]
pub struct Write {
    bytes: Vec<u8>,
}

impl Write {
    /// Creates a new coroutine to write the given bytes.
    pub fn new(bytes: Vec<u8>) -> Self {
        trace!("init coroutine for writing {} bytes", bytes.len());
        Self { bytes }
    }

    // /// Replaces the inner bytes with the given one.
    // pub fn replace(&mut self, bytes: impl IntoIterator<Item = u8>) {
    //     *self = Self::new(bytes.into_iter()collect());
    // }

    // /// Adds the given bytes the to inner buffer.
    // pub fn extend(&mut self, more_bytes: impl IntoIterator<Item = u8>) {
    //     match &mut self.bytes {
    //         Some(bytes) => {
    //             let prev_len = bytes.len();
    //             bytes.extend(more_bytes);
    //             let next_len = bytes.len();
    //             let n = next_len - prev_len;
    //             trace!("prepare {prev_len}+{n} additional bytes to be written");
    //         }
    //         None => self.replace(more_bytes),
    //     }
    // }

    /// Makes the write progress.
    pub fn resume(&mut self, arg: Option<StreamIo>) -> WriteResult {
        let Some(arg) = arg else {
            let bytes = self.bytes.drain(..).collect();
            trace!("wants I/O to write bytes");
            return WriteResult::Io(StreamIo::Write(Err(bytes)));
        };

        trace!("resume after writing bytes");

        let StreamIo::Write(io) = arg else {
            return WriteResult::Err(WriteError::InvalidArgument("write output", arg));
        };

        let output = match io {
            Ok(output) => output,
            Err(bytes) => return WriteResult::Io(StreamIo::Write(Err(bytes))),
        };

        match output.bytes_count {
            0 => WriteResult::Eof,
            n => {
                debug!("wrote {n} bytes");
                WriteResult::Ok(output)
            }
        }
    }
}

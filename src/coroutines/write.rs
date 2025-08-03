//! I/O-free coroutine to write bytes into a stream.

use log::{debug, trace};
use thiserror::Error;

use crate::io::{StreamIo, StreamOutput};

/// Errors that can occur during the coroutine progression.
#[derive(Clone, Debug, Error)]
pub enum WriteStreamError {
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
pub enum WriteStreamResult {
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
    Err(WriteStreamError),
}

/// I/O-free coroutine to write bytes into a stream.
#[derive(Debug, Default)]
pub struct WriteStream {
    bytes: Vec<u8>,
}

impl WriteStream {
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
    pub fn resume(&mut self, arg: Option<StreamIo>) -> WriteStreamResult {
        let Some(arg) = arg else {
            let bytes = self.bytes.drain(..).collect();
            trace!("wants I/O to write bytes");
            return WriteStreamResult::Io(StreamIo::Write(Err(bytes)));
        };

        trace!("resume after writing bytes");

        let StreamIo::Write(io) = arg else {
            return WriteStreamResult::Err(WriteStreamError::InvalidArgument("write output", arg));
        };

        let output = match io {
            Ok(output) => output,
            Err(bytes) => return WriteStreamResult::Io(StreamIo::Write(Err(bytes))),
        };

        match output.bytes_count {
            0 => WriteStreamResult::Eof,
            n => {
                debug!("wrote {n} bytes");
                WriteStreamResult::Ok(output)
            }
        }
    }
}

use std::fmt;

/// The streams I/O request enum, emitted by [coroutines] and
/// processed by [runtimes].
///
/// Represents all the possible I/O requests that a stream coroutine
/// can emit. Runtimes should be able to handle all variants.
///
/// [coroutines]: crate::coroutines
/// [runtimes]: crate::runtimes
#[derive(Clone, Eq, PartialEq)]
pub enum StreamIo {
    /// I/O for reading bytes.
    Read(Result<StreamOutput, Vec<u8>>),

    /// I/O for writing bytes.
    Write(Result<StreamOutput, Vec<u8>>),
}

impl fmt::Debug for StreamIo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Read(Ok(_)) => f.write_str("read output"),
            Self::Read(Err(_)) => f.write_str("read input"),
            Self::Write(Ok(_)) => f.write_str("write output"),
            Self::Write(Err(_)) => f.write_str("write input"),
        }
    }
}

/// Output returned by both read and write coroutines.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StreamOutput {
    /// The inner buffer.
    pub buffer: Vec<u8>,

    /// The amount of bytes that have been read/written.
    pub bytes_count: usize,
}

impl StreamOutput {
    pub fn bytes(&self) -> &[u8] {
        &self.buffer[..self.bytes_count]
    }
}

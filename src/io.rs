use std::fmt;

/// The streams I/O request enum, emitted by [coroutines] and
/// processed by [runtimes].
///
/// Represents all the possible I/O requests that a stream coroutine
/// can emit. Runtimes should be able to handle all variants.
///
/// [coroutines]: crate::coroutines
/// [runtimes]: crate::runtimes
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Io {
    /// Generic error related to coroutine progression.
    Error(String),

    /// I/O for reading bytes.
    Read(Result<Output, Vec<u8>>),

    /// I/O for writing bytes.
    Write(Result<Output, Vec<u8>>),
}

impl Io {
    pub fn err(msg: impl fmt::Display) -> Io {
        let msg = format!("Stream error: {msg}");
        Io::Error(msg)
    }
}

/// Output returned by both read and write coroutines.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Output {
    /// The inner buffer.
    pub buffer: Vec<u8>,

    /// The amount of bytes that have been read/written.
    pub bytes_count: usize,
}

impl Output {
    pub fn bytes(&self) -> &[u8] {
        &self.buffer[..self.bytes_count]
    }
}

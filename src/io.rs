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
    Error(String),
    Read(Result<Output, Vec<u8>>),
    Write(Result<Output, Vec<u8>>),
}

impl Io {
    pub fn err(message: impl fmt::Display) -> Io {
        let message = format!("stream error: {message}");
        Io::Error(message.to_string())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Output {
    pub buffer: Vec<u8>,
    pub bytes_count: usize,
}

impl Output {
    pub fn bytes(&self) -> &[u8] {
        &self.buffer[..self.bytes_count]
    }
}

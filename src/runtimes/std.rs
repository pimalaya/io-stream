//! The standard, blocking stream runtime.

use std::io::{self, Read, Write};

use log::trace;

use crate::io::{StreamIo, StreamOutput};

/// The standard, blocking filesystem runtime handler.
///
/// This handler makes use of standard modules [`std::io`] to process
/// [`StreamIo`].
pub fn handle(stream: impl Read + Write, io: StreamIo) -> io::Result<StreamIo> {
    match io {
        StreamIo::Read(io) => read(stream, io),
        StreamIo::Write(io) => write(stream, io),
    }
}

pub fn read(mut stream: impl Read, input: Result<StreamOutput, Vec<u8>>) -> io::Result<StreamIo> {
    let mut buffer = match input {
        Ok(output) => return Ok(StreamIo::Read(Ok(output))),
        Err(buffer) => buffer,
    };

    trace!("reading bytes synchronously");
    let bytes_count = stream.read(&mut buffer)?;

    let output = StreamOutput {
        buffer,
        bytes_count,
    };

    Ok(StreamIo::Read(Ok(output)))
}

pub fn write(mut stream: impl Write, input: Result<StreamOutput, Vec<u8>>) -> io::Result<StreamIo> {
    let bytes = match input {
        Ok(output) => return Ok(StreamIo::Write(Ok(output))),
        Err(bytes) => bytes,
    };

    trace!("writing bytes synchronously");
    let bytes_count = stream.write(&bytes)?;

    let output = StreamOutput {
        buffer: bytes,
        bytes_count,
    };

    Ok(StreamIo::Write(Ok(output)))
}

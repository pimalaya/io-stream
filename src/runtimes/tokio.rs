//! The Tokio-based, async stream runtime.

use std::io;

use log::trace;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

use crate::io::{StreamIo, StreamOutput};

/// The Tokio-based, async stream runtime handler.
///
/// This handler makes use of standard module [`std::io`] and Tokio
/// module [`tokio::io`] to process [`StreamIo`].
pub async fn handle(
    stream: impl AsyncRead + AsyncWrite + Unpin,
    io: StreamIo,
) -> io::Result<StreamIo> {
    match io {
        StreamIo::Read(io) => read(stream, io).await,
        StreamIo::Write(io) => write(stream, io).await,
    }
}

pub async fn read(
    mut stream: impl AsyncRead + Unpin,
    input: Result<StreamOutput, Vec<u8>>,
) -> io::Result<StreamIo> {
    let mut buffer = match input {
        Ok(output) => return Ok(StreamIo::Read(Ok(output))),
        Err(buffer) => buffer,
    };

    trace!("reading bytes asynchronously");
    let bytes_count = stream.read(&mut buffer).await?;

    let output = StreamOutput {
        buffer,
        bytes_count,
    };

    Ok(StreamIo::Read(Ok(output)))
}

pub async fn write(
    mut stream: impl AsyncWrite + Unpin,
    input: Result<StreamOutput, Vec<u8>>,
) -> io::Result<StreamIo> {
    let bytes = match input {
        Ok(output) => return Ok(StreamIo::Write(Ok(output))),
        Err(bytes) => bytes,
    };

    trace!("writing bytes asynchronously");
    let bytes_count = stream.write(&bytes).await?;

    let output = StreamOutput {
        buffer: bytes,
        bytes_count,
    };

    Ok(StreamIo::Write(Ok(output)))
}

//! Module dedicated to the Tokio-based, async runtime.

use std::io;

use log::trace;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

use crate::{Io, Output};

/// The main runtime I/O handler.
///
/// This handler makes use of the [`tokio::io`] module as well as
/// standard module [`std::io`] to process stream [`Io`].
pub async fn handle(stream: impl AsyncRead + AsyncWrite + Unpin, io: Io) -> io::Result<Io> {
    match io {
        Io::Error(err) => Err(io::Error::new(io::ErrorKind::Other, err)),
        Io::Read(io) => read(stream, io).await,
        Io::Write(io) => write(stream, io).await,
    }
}

pub async fn read(
    mut stream: impl AsyncRead + Unpin,
    input: Result<Output, Vec<u8>>,
) -> io::Result<Io> {
    let Err(mut buffer) = input else {
        let kind = io::ErrorKind::InvalidInput;
        return Err(io::Error::new(kind, "Missing read buffer"));
    };

    trace!("reading bytes asynchronously");
    let bytes_count = stream.read(&mut buffer).await?;

    let output = Output {
        buffer,
        bytes_count,
    };

    Ok(Io::Read(Ok(output)))
}

pub async fn write(
    mut stream: impl AsyncWrite + Unpin,
    input: Result<Output, Vec<u8>>,
) -> io::Result<Io> {
    let Err(buffer) = input else {
        let kind = io::ErrorKind::InvalidInput;
        return Err(io::Error::new(kind, "Missing write bytes"));
    };

    trace!("writing bytes asynchronously");
    let bytes_count = stream.write(&buffer).await?;

    let output = Output {
        buffer,
        bytes_count,
    };

    Ok(Io::Write(Ok(output)))
}

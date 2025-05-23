//! Module dedicated to the standard, blocking runtime.

use std::io::{self, Read, Write};

use log::trace;

use crate::{Io, Output};

/// The main runtime I/O handler.
///
/// This handler makes use of standard modules [`std::io`] to process
/// stream [`Io`].
pub fn handle(stream: impl Read + Write, io: Io) -> io::Result<Io> {
    match io {
        Io::Error(err) => Err(io::Error::new(io::ErrorKind::Other, err)),
        Io::Read(io) => read(stream, io),
        Io::Write(io) => write(stream, io),
    }
}

pub fn read(mut stream: impl Read, input: Result<Output, Vec<u8>>) -> io::Result<Io> {
    let Err(mut buffer) = input else {
        let kind = io::ErrorKind::InvalidInput;
        return Err(io::Error::new(kind, "Missing read buffer"));
    };

    trace!("reading bytes synchronously");
    let bytes_count = stream.read(&mut buffer)?;

    let output = Output {
        buffer,
        bytes_count,
    };

    Ok(Io::Read(Ok(output)))
}

pub fn write(mut stream: impl Write, input: Result<Output, Vec<u8>>) -> io::Result<Io> {
    let Err(buffer) = input else {
        let kind = io::ErrorKind::InvalidInput;
        return Err(io::Error::new(kind, "Missing write bytes"));
    };

    trace!("writing bytes synchronously");
    let bytes_count = stream.write(&buffer)?;

    let output = Output {
        buffer,
        bytes_count,
    };

    Ok(Io::Write(Ok(output)))
}

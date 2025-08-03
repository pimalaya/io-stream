#![cfg(feature = "tokio")]

use std::env;

use io_stream::{
    coroutines::{
        read::{ReadStream, ReadStreamResult},
        write::{WriteStream, WriteStreamResult},
    },
    runtimes::tokio::handle,
};
use tokio::{
    io::{stdin, stdout, AsyncBufReadExt, AsyncWriteExt, BufReader, Stdout},
    net::TcpStream,
};

#[tokio::main]
async fn main() {
    env_logger::init();

    let mut stdout = stdout();

    let host = match env::var("HOST") {
        Ok(host) => host,
        Err(_) => prompt(&mut stdout, "TCP server host?").await,
    };

    let port: u16 = match env::var("PORT") {
        Ok(port) => port.parse().unwrap(),
        Err(_) => prompt(&mut stdout, "TCP server port?")
            .await
            .parse()
            .unwrap(),
    };

    let mut tcp = TcpStream::connect((host.as_str(), port)).await.unwrap();

    stdout.write(b"\nReceived greeting:\n").await.unwrap();

    let mut arg = None;
    let mut read = ReadStream::new();

    let greeting = loop {
        match read.resume(arg) {
            ReadStreamResult::Ok(output) => break output,
            ReadStreamResult::Err(err) => panic!("{err}"),
            ReadStreamResult::Eof => panic!("reached unexpected EOF"),
            ReadStreamResult::Io(io) => arg = Some(handle(&mut tcp, io).await.unwrap()),
        }
    };

    let mut lines = greeting.bytes().lines();
    while let Ok(Some(line)) = lines.next_line().await {
        stdout
            .write(format!("S: {line}\n").as_bytes())
            .await
            .unwrap();
    }

    loop {
        stdout.write(b"\n").await.unwrap();

        let mut data = prompt(&mut stdout, "C:").await;
        data.push_str("\r\n");

        let mut arg = None;
        let mut write = WriteStream::new(data.into_bytes());

        loop {
            match write.resume(arg) {
                WriteStreamResult::Ok(_) => break,
                WriteStreamResult::Err(err) => panic!("{err}"),
                WriteStreamResult::Eof => panic!("reached unexpected EOF"),
                WriteStreamResult::Io(io) => arg = Some(handle(&mut tcp, io).await.unwrap()),
            }
        }

        let mut arg = None;
        let mut read = ReadStream::new();

        let response = loop {
            match read.resume(arg) {
                ReadStreamResult::Ok(output) => break output,
                ReadStreamResult::Err(err) => panic!("{err}"),
                ReadStreamResult::Eof => panic!("reached unexpected EOF"),
                ReadStreamResult::Io(io) => arg = Some(handle(&mut tcp, io).await.unwrap()),
            }
        };

        let mut lines = response.bytes().lines();
        while let Ok(Some(line)) = lines.next_line().await {
            stdout
                .write(format!("S: {line}\n").as_bytes())
                .await
                .unwrap();
        }
    }
}

async fn prompt(stdout: &mut Stdout, message: &str) -> String {
    stdout
        .write(format!("{message} ").as_bytes())
        .await
        .unwrap();

    stdout.flush().await.unwrap();

    let mut line = String::new();
    BufReader::new(stdin()).read_line(&mut line).await.unwrap();

    line.trim().to_owned()
}

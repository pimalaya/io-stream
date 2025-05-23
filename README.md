# I/O Stream [![Documentation](https://img.shields.io/docsrs/io-stream)](https://docs.rs/io-stream/latest/io_stream) [![Matrix](https://img.shields.io/matrix/pimalaya:matrix.org?color=success&label=chat)](https://matrix.to/#/#pimalaya:matrix.org)

Set of **I/O-free** Rust coroutines and runtimes to manage streams.

This library allows you to manage streams using an I/O-agnostic approach, based on 3 concepts:

### Coroutine

A coroutine is an *I/O-free*, *resumable* and *composable* state machine that **emits I/O requests**. A coroutine is considered *terminated* when it does not emit I/O requests anymore.

*See available coroutines at [./src/coroutines](https://github.com/pimalaya/io-stream/tree/master/src/coroutines).*

### Runtime

A runtime contains all the I/O logic, and is responsible for **processing I/O requests** emitted by coroutines.

*See available runtimes at [./src/runtimes](https://github.com/pimalaya/io-stream/tree/master/src/runtimes).*

### Loop

The loop is the glue between coroutines and runtimes. It makes the coroutine progress while allowing runtime to process I/O.

## Examples

*See complete examples at [./examples](https://github.com/pimalaya/io-stream/blob/master/examples).*

### Read stdin synchronously

```rust,ignore
use io_stream::{coroutines::Read, runtimes::std::handle};

let mut stdin = std::io::stdin();

let mut arg = None;
let mut read = Read::new();

let output = loop {
    match read.resume(arg) {
        Ok(output) => break output,
        Err(io) => arg = Some(handle(&mut stdin, io).unwrap()),
    }
};

// same as &output.buffer[..output.bytes_count]
let bytes = output.bytes();
```

### Write TCP stream asynchronously

```rust,ignore
use tokio::net::TcpStream;
use io_stream::{coroutines::Write, runtimes::tokio::handle};

let mut tcp = TcpStream::connect("127.0.0.1:1234").await.unwrap();

let mut arg = None;
let mut write = Write::new(b"bytes".iter().cloned());

while let Err(io) = write.resume(arg) {
    arg = Some(handle(&mut tcp, io).await.unwrap());
}
```

### More examples

Have a look at projects built on the top of this library:

- [io-starttls](https://github.com/pimalaya/io-starttls): I/O-free coroutine to upgrade any plain stream to a secure one
- [io-timer](https://github.com/pimalaya/io-timer): Set of I/O-free coroutines to manage timers
- [io-http](https://github.com/pimalaya/io-http): Set of I/O-free Rust coroutines to manage HTTP streams
- [io-oauth](https://github.com/pimalaya/io-oauth): Set of I/O-free Rust coroutines to manage OAuth flows
- [io-addressbook](https://github.com/pimalaya/io-addressbook): Set of I/O-free coroutines to manage contacts
- [comodoro](https://github.com/pimalaya/comodoro): CLI to manage timers
- [cardamum](https://github.com/pimalaya/cardamum): CLI to manage contacts
- [ortie](https://github.com/pimalaya/ortie): CLI to manage OAuth access tokens

## Sponsoring

[![nlnet](https://nlnet.nl/logo/banner-160x60.png)](https://nlnet.nl/)

Special thanks to the [NLnet foundation](https://nlnet.nl/) and the [European Commission](https://www.ngi.eu/) that helped the project to receive financial support from various programs:

- [NGI Assure](https://nlnet.nl/project/Himalaya/) in 2022
- [NGI Zero Entrust](https://nlnet.nl/project/Pimalaya/) in 2023
- [NGI Zero Core](https://nlnet.nl/project/Pimalaya-PIM/) in 2024 *(still ongoing)*

If you appreciate the project, feel free to donate using one of the following providers:

[![GitHub](https://img.shields.io/badge/-GitHub%20Sponsors-fafbfc?logo=GitHub%20Sponsors)](https://github.com/sponsors/soywod)
[![Ko-fi](https://img.shields.io/badge/-Ko--fi-ff5e5a?logo=Ko-fi&logoColor=ffffff)](https://ko-fi.com/soywod)
[![Buy Me a Coffee](https://img.shields.io/badge/-Buy%20Me%20a%20Coffee-ffdd00?logo=Buy%20Me%20A%20Coffee&logoColor=000000)](https://www.buymeacoffee.com/soywod)
[![Liberapay](https://img.shields.io/badge/-Liberapay-f6c915?logo=Liberapay&logoColor=222222)](https://liberapay.com/soywod)
[![thanks.dev](https://img.shields.io/badge/-thanks.dev-000000?logo=data:image/svg+xml;base64,PHN2ZyB3aWR0aD0iMjQuMDk3IiBoZWlnaHQ9IjE3LjU5NyIgY2xhc3M9InctMzYgbWwtMiBsZzpteC0wIHByaW50Om14LTAgcHJpbnQ6aW52ZXJ0IiB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciPjxwYXRoIGQ9Ik05Ljc4MyAxNy41OTdINy4zOThjLTEuMTY4IDAtMi4wOTItLjI5Ny0yLjc3My0uODktLjY4LS41OTMtMS4wMi0xLjQ2Mi0xLjAyLTIuNjA2di0xLjM0NmMwLTEuMDE4LS4yMjctMS43NS0uNjc4LTIuMTk1LS40NTItLjQ0Ni0xLjIzMi0uNjY5LTIuMzQtLjY2OUgwVjcuNzA1aC41ODdjMS4xMDggMCAxLjg4OC0uMjIyIDIuMzQtLjY2OC40NTEtLjQ0Ni42NzctMS4xNzcuNjc3LTIuMTk1VjMuNDk2YzAtMS4xNDQuMzQtMi4wMTMgMS4wMjEtMi42MDZDNS4zMDUuMjk3IDYuMjMgMCA3LjM5OCAwaDIuMzg1djEuOTg3aC0uOTg1Yy0uMzYxIDAtLjY4OC4wMjctLjk4LjA4MmExLjcxOSAxLjcxOSAwIDAgMC0uNzM2LjMwN2MtLjIwNS4xNTYtLjM1OC4zODQtLjQ2LjY4Mi0uMTAzLjI5OC0uMTU0LjY4Mi0uMTU0IDEuMTUxVjUuMjNjMCAuODY3LS4yNDkgMS41ODYtLjc0NSAyLjE1NS0uNDk3LjU2OS0xLjE1OCAxLjAwNC0xLjk4MyAxLjMwNXYuMjE3Yy44MjUuMyAxLjQ4Ni43MzYgMS45ODMgMS4zMDUuNDk2LjU3Ljc0NSAxLjI4Ny43NDUgMi4xNTR2MS4wMjFjMCAuNDcuMDUxLjg1NC4xNTMgMS4xNTIuMTAzLjI5OC4yNTYuNTI1LjQ2MS42ODIuMTkzLjE1Ny40MzcuMjYuNzMyLjMxMi4yOTUuMDUuNjIzLjA3Ni45ODQuMDc2aC45ODVabTE0LjMxNC03LjcwNmgtLjU4OGMtMS4xMDggMC0xLjg4OC4yMjMtMi4zNC42NjktLjQ1LjQ0NS0uNjc3IDEuMTc3LS42NzcgMi4xOTVWMTQuMWMwIDEuMTQ0LS4zNCAyLjAxMy0xLjAyIDIuNjA2LS42OC41OTMtMS42MDUuODktMi43NzQuODloLTIuMzg0di0xLjk4OGguOTg0Yy4zNjIgMCAuNjg4LS4wMjcuOTgtLjA4LjI5Mi0uMDU1LjUzOC0uMTU3LjczNy0uMzA4LjIwNC0uMTU3LjM1OC0uMzg0LjQ2LS42ODIuMTAzLS4yOTguMTU0LS42ODIuMTU0LTEuMTUydi0xLjAyYzAtLjg2OC4yNDgtMS41ODYuNzQ1LTIuMTU1LjQ5Ny0uNTcgMS4xNTgtMS4wMDQgMS45ODMtMS4zMDV2LS4yMTdjLS44MjUtLjMwMS0xLjQ4Ni0uNzM2LTEuOTgzLTEuMzA1LS40OTctLjU3LS43NDUtMS4yODgtLjc0NS0yLjE1NXYtMS4wMmMwLS40Ny0uMDUxLS44NTQtLjE1NC0xLjE1Mi0uMTAyLS4yOTgtLjI1Ni0uNTI2LS40Ni0uNjgyYTEuNzE5IDEuNzE5IDAgMCAwLS43MzctLjMwNyA1LjM5NSA1LjM5NSAwIDAgMC0uOTgtLjA4MmgtLjk4NFYwaDIuMzg0YzEuMTY5IDAgMi4wOTMuMjk3IDIuNzc0Ljg5LjY4LjU5MyAxLjAyIDEuNDYyIDEuMDIgMi42MDZ2MS4zNDZjMCAxLjAxOC4yMjYgMS43NS42NzggMi4xOTUuNDUxLjQ0NiAxLjIzMS42NjggMi4zNC42NjhoLjU4N3oiIGZpbGw9IiNmZmYiLz48L3N2Zz4=)](https://thanks.dev/soywod)
[![PayPal](https://img.shields.io/badge/-PayPal-0079c1?logo=PayPal&logoColor=ffffff)](https://www.paypal.com/paypalme/soywod)

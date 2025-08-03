//! Collection of I/O-free, resumable and composable stream state
//! machines.
//!
//! Coroutines emit [I/O] requests that need to be processed by
//! [runtimes] in order to continue their progression.
//!
//! [I/O]: crate::io::StreamIo
//! [runtimes]: crate::runtimes

pub mod read;
#[path = "read-exact.rs"]
pub mod read_exact;
#[path = "read-to-end.rs"]
pub mod read_to_end;
pub mod write;

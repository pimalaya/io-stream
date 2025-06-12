//! Collection of I/O-free, resumable and composable stream state
//! machines.
//!
//! Coroutines emit [`Io`] requests that need to be processed by
//! [runtimes] in order to continue their progression.
//!
//! [`Io`]: crate::Io
//! [runtimes]: crate::runtimes

pub mod read;
#[path = "read-exact.rs"]
pub mod read_exact;
#[path = "read-to-end.rs"]
pub mod read_to_end;
pub mod write;

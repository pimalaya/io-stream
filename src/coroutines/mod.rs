//! Collection of I/O-free, resumable and composable stream state
//! machines.
//!
//! Coroutines emit [`Io`] requests that need to be processed by
//! [runtimes] in order to continue their progression.
//!
//! [`Io`]: crate::Io
//! [runtimes]: crate::runtimes

mod read;
#[path = "read-exact.rs"]
mod read_exact;
#[path = "read-to-end.rs"]
mod read_to_end;
mod write;

#[doc(inline)]
pub use self::{read::Read, read_exact::ReadExact, read_to_end::ReadToEnd, write::Write};

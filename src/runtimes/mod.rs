//! Collection of stream runtimes.
//!
//! A runtime contains all the I/O logic, and is responsible for
//! processing [I/O] requests emitted by [coroutines].
//!
//! If you miss a runtime matching your requirements, you can easily
//! implement your own by taking example on the existing ones. PRs are
//! welcomed!
//!
//! [I/O]: crate::io::Io
//! [coroutines]: crate::coroutines

#[cfg(feature = "std")]
pub mod std;
#[cfg(feature = "tokio")]
pub mod tokio;

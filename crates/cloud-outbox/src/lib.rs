//! Pure outbox state machine for cloud file synchronization.

pub mod backoff;
mod batch_adjust;
pub mod codec;
pub mod enqueue;
pub mod history;
pub mod merge;
pub mod scheduling;
pub mod summary;
pub mod transitions;
pub mod types;

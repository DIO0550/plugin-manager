//! Sync endpoint sub-parent.
//!
//! Groups source/destination wrappers around a Target environment used by
//! the sync orchestrator. The leaves keep test form A
//! (`#[path = "*_test.rs"] mod tests;` at the bottom of each leaf file), so
//! no test declarations are needed at this sub-parent.

mod destination;
mod source;

pub use self::destination::SyncDestination;
pub use self::source::SyncSource;

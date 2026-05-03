//! Sync value-object sub-parent.
//!
//! Groups action / options / placed / result value objects under a single
//! namespace so cross-subgroup references inside `sync/endpoint/` can use
//! `super::super::model::*` paths. The leaves keep test form A
//! (`#[path = "*_test.rs"] mod tests;` at the bottom of each leaf file), so
//! no test declarations are needed at this sub-parent.

mod action;
mod options;
mod placed;
mod result;

pub use self::action::SyncAction;
pub use self::options::{SyncOptions, SyncableKind};
pub use self::placed::{PlacedComponent, PlacedRef};
pub use self::result::{SyncFailure, SyncResult};

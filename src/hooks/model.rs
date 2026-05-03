//! Hook value-object sub-parent.
//!
//! Groups hook value objects (`hook_definition`, `name`, `script_path`) under
//! a single namespace so cross-subgroup references inside `hooks/converter/`
//! can use `super::super::model::*` paths. Children are declared as
//! `pub(crate) mod` (rustc E0365 — re-exporting a private module is rejected,
//! so the parent `hooks.rs` line `pub(crate) use model::name;` requires at
//! least crate-internal visibility). `pub mod` is forbidden because it would
//! introduce a new external public path `crate::hooks::model::*` and violate
//! the DoD; `pub(crate) mod` is crate-internal only and adds no external path.

pub(crate) mod hook_definition;
pub(crate) mod name;
pub(crate) mod script_path;

pub(crate) use self::hook_definition::{CommandHook, HookDefinition, HttpHook, StubHook};

#[cfg(test)]
mod name_test;
#[cfg(test)]
mod script_path_test;

//! Hooks module for hook configuration conversion.
//!
//! Provides polymorphic conversion layers for Claude Code hooks
//! to various target formats (Copilot CLI, Codex, etc.).

pub mod converter;
pub(crate) mod event;
pub(crate) mod hook_definition;
pub(crate) mod name;
pub(crate) mod script_path;
pub(crate) mod tool;

#[cfg(test)]
mod name_test;
#[cfg(test)]
mod script_path_test;

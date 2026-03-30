//! Hooks module for hook configuration conversion.
//!
//! Provides polymorphic conversion layers for Claude Code hooks
//! to various target formats (Copilot CLI, Codex, etc.).

pub(crate) mod codex;
pub mod converter;
pub(crate) mod copilot;
pub(crate) mod event;
pub(crate) mod hook_definition;
pub(crate) mod name;
pub(crate) mod tool;

#[cfg(test)]
mod codex_test;
#[cfg(test)]
mod converter_test;
#[cfg(test)]
mod copilot_test;
#[cfg(test)]
mod name_test;

//! Hooks module for Claude Code <-> Copilot CLI conversion.
//!
//! Provides event name and tool name mapping functions
//! for the Hooks stdin/stdout context.

pub mod converter;
pub mod event_map;

#[cfg(test)]
mod converter_test;
#[cfg(test)]
mod event_map_test;

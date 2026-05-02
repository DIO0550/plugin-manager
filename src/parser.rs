//! Parser module for command/prompt/agent files.

mod claude_code;
mod codex;
pub mod convert;
mod copilot;
mod frontmatter;

pub use claude_code::{ClaudeCodeAgent, ClaudeCodeCommand};
pub use convert::TargetType;

#[cfg(test)]
mod convert_test;
#[cfg(test)]
mod frontmatter_test;

//! Parser module for command/prompt/agent files.

mod claude_code;
mod claude_code_agent;
mod codex;
mod codex_agent;
pub mod convert;
mod copilot;
mod copilot_agent;
mod frontmatter;

pub use claude_code::ClaudeCodeCommand;
pub use convert::TargetType;

#[cfg(test)]
mod claude_code_agent_test;
#[cfg(test)]
mod claude_code_test;
#[cfg(test)]
mod codex_agent_test;
#[cfg(test)]
mod codex_test;
#[cfg(test)]
mod convert_test;
#[cfg(test)]
mod copilot_agent_test;
#[cfg(test)]
mod copilot_test;
#[cfg(test)]
mod frontmatter_test;

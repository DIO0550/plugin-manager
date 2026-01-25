//! Parser module for command/prompt files.

mod claude_code;
mod codex;
mod copilot;
mod frontmatter;

pub use claude_code::ClaudeCodeCommand;
pub use codex::CodexPrompt;
pub use copilot::CopilotPrompt;
pub use frontmatter::{parse_frontmatter, ParsedDocument};

#[cfg(test)]
mod claude_code_test;
#[cfg(test)]
mod codex_test;
#[cfg(test)]
mod copilot_test;
#[cfg(test)]
mod frontmatter_test;

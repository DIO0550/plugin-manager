//! Codex parser subgroup (prompts and agents).

mod agent;
mod prompt;

pub(crate) use agent::CodexAgent;
pub(crate) use prompt::CodexPrompt;

#[cfg(test)]
mod agent_test;
#[cfg(test)]
mod prompt_test;

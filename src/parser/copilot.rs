//! Copilot parser subgroup (prompts and agents).

mod agent;
mod prompt;

pub(crate) use agent::CopilotAgent;
pub(crate) use prompt::CopilotPrompt;

#[cfg(test)]
mod agent_test;
#[cfg(test)]
mod prompt_test;

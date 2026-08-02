//! ターゲット環境（実装系）モジュール集約
//!
//! antigravity / codex / copilot / cursor / gemini_cli / opencode の各 `Target` 実装を集約する。

mod antigravity;
mod codex;
mod copilot;
mod cursor;
mod gemini_cli;
mod opencode;

pub use antigravity::AntigravityTarget;
pub use codex::{CodexTarget, FeatureFlagOutcome};
pub use copilot::CopilotTarget;
pub use cursor::CursorTarget;
pub use gemini_cli::GeminiCliTarget;
pub use opencode::OpenCodeTarget;

pub(crate) use opencode::personal_root_from_env;

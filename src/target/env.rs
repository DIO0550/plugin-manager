//! ターゲット環境（実装系）モジュール集約
//!
//! antigravity / codex / copilot / cursor / gemini_cli の各 `Target` 実装を集約する。

mod antigravity;
mod codex;
mod copilot;
mod cursor;
mod gemini_cli;

pub use antigravity::AntigravityTarget;
pub use codex::{apply_codex_hooks_flag, CodexTarget, FeatureFlagOutcome};
pub use copilot::CopilotTarget;
pub use cursor::CursorTarget;
pub use gemini_cli::GeminiCliTarget;

//! ターゲット環境（実装系）モジュール集約
//!
//! antigravity / codex / copilot / gemini_cli の各 `Target` 実装を集約する。

mod antigravity;
mod codex;
mod copilot;
mod gemini_cli;

pub use antigravity::AntigravityTarget;
pub use codex::CodexTarget;
pub use copilot::CopilotTarget;
pub use gemini_cli::GeminiCliTarget;

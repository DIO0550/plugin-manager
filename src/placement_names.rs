//! 配置パス断片の単一真実源（葉モジュール）
//!
//! - ターゲット非依存の名前は [`crate::component::ComponentKind`] を正とする
//! - 本モジュールはターゲット依存の文字列定数（環境ルート・instruction ファイル名）を保持する
//! - `target` / `scan` / `cleanup` がここを消費し、`target` ↔ `scan` 循環を避ける

// ---------------------------------------------------------------------------
// Instruction ファイル名
// ---------------------------------------------------------------------------

pub const INSTRUCTION_AGENTS: &str = "AGENTS.md";
pub const INSTRUCTION_COPILOT: &str = "copilot-instructions.md";
pub const INSTRUCTION_GEMINI: &str = "GEMINI.md";

/// 全ターゲットの Instruction ファイル名（重複なし）。
///
/// 各ターゲットの `instruction_filename` と一致すること（`placement_names` / `target` のテストで固定）。
pub const ALL_INSTRUCTION_FILENAMES: &[&str] =
    &[INSTRUCTION_AGENTS, INSTRUCTION_COPILOT, INSTRUCTION_GEMINI];

// ---------------------------------------------------------------------------
// 環境ルート（単一セグメント）
// ---------------------------------------------------------------------------

pub const CODEX_SUBDIR: &str = ".codex";
pub const COPILOT_PERSONAL_SUBDIR: &str = ".copilot";
pub const COPILOT_PROJECT_SUBDIR: &str = ".github";
pub const ANTIGRAVITY_PERSONAL_PARENT: &str = ".gemini";
pub const ANTIGRAVITY_PERSONAL_CHILD: &str = "antigravity";
pub const ANTIGRAVITY_PROJECT_SUBDIR: &str = ".agent";
pub const GEMINI_SUBDIR: &str = ".gemini";
pub const CURSOR_SUBDIR: &str = ".cursor";

/// Copilot Command の実配置ディレクトリ（表示用 `plural()` の `"commands"` とは異なる）。
pub const COPILOT_COMMAND_SUBDIR: &str = "prompts";

#[cfg(test)]
#[path = "placement_names_test.rs"]
mod tests;

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
/// Antigravity Hooks の Global（Personal）配置: `~/.gemini/config/hooks.json`
pub const ANTIGRAVITY_HOOKS_PERSONAL_CHILD: &str = "config";
/// Antigravity Hooks の Project 配置ディレクトリ（Skills の `.agent` とは別）
pub const ANTIGRAVITY_HOOKS_PROJECT_SUBDIR: &str = ".agents";
pub const ANTIGRAVITY_HOOKS_FILE: &str = "hooks.json";
pub const GEMINI_SUBDIR: &str = ".gemini";
pub const CURSOR_SUBDIR: &str = ".cursor";

/// OpenCode の Personal 配置ルート親（`~/.config`）。`$XDG_CONFIG_HOME` 設定時は使わない。
pub const OPENCODE_PERSONAL_PARENT: &str = ".config";
/// OpenCode の Personal 配置ルート子（`opencode`）。`$XDG_CONFIG_HOME/opencode` にも使う。
pub const OPENCODE_PERSONAL_CHILD: &str = "opencode";
/// OpenCode の Project 配置ルート
pub const OPENCODE_PROJECT_SUBDIR: &str = ".opencode";

/// Copilot Command の実配置ディレクトリ（表示用 `plural()` の `"commands"` とは異なる）。
pub const COPILOT_COMMAND_SUBDIR: &str = "prompts";

// ---------------------------------------------------------------------------
// Plugin リソース
// ---------------------------------------------------------------------------

/// ターゲット上のプラグインリソース親ディレクトリ（`<base>/plugins/<plugin>/...`）。
pub const PLUGIN_RESOURCES_SUBDIR: &str = "plugins";

/// Claude Code Plugin マニフェスト用ディレクトリ。
pub const CLAUDE_PLUGIN_DIR: &str = ".claude-plugin";

/// プラグインルート直下の `plugin.json`。
pub const PLUGIN_JSON_FILE: &str = "plugin.json";

/// PLM 管理メタデータファイル。
pub const PLM_META_FILE: &str = ".plm-meta.json";

/// プラグインルート直下でリソースからも除外する VCS / OS メタ名。
pub const PLUGIN_RESOURCE_VCS_NAMES: &[&str] = &[
    ".git",
    ".gitignore",
    ".gitattributes",
    ".github",
    ".DS_Store",
];

#[cfg(test)]
#[path = "placement_names_test.rs"]
mod tests;

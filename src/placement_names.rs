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

/// Copilot Command の実配置ディレクトリ（表示用 `plural()` の `"commands"` とは異なる）。
pub const COPILOT_COMMAND_SUBDIR: &str = "prompts";

// ---------------------------------------------------------------------------
// Plugin 付属リソース除外
// ---------------------------------------------------------------------------

/// プラグインマニフェスト用の予約ディレクトリ。
pub const PLUGIN_MANIFEST_DIR: &str = ".claude-plugin";

/// プラグインルート直下の `plugin.json`（マニフェスト）。
pub const PLUGIN_MANIFEST_FILE: &str = "plugin.json";

/// PLM 管理メタデータファイル。
pub const PLM_META_FILE: &str = ".plm-meta.json";

/// プラグインパッケージ内のデフォルト instruction ファイル名。
pub const DEFAULT_INSTRUCTIONS_FILE: &str = "instructions.md";

/// Plugin 付属リソース検出時に除外する正確なベースネーム（VCS / CI / OS / 予約）。
pub const ATTACHED_EXACT_EXCLUSIONS: &[&str] = &[
    PLUGIN_MANIFEST_DIR,
    PLUGIN_MANIFEST_FILE,
    PLM_META_FILE,
    DEFAULT_INSTRUCTIONS_FILE,
    ".git",
    ".gitignore",
    ".gitattributes",
    ".github",
    ".DS_Store",
    COPILOT_COMMAND_SUBDIR,
];

/// Plugin 付属リソース検出時に除外するプレフィックス（大文字小文字無視）。
///
/// `README` / `LICENSE` / `CHANGELOG` / `CONTRIBUTING` で始まる名前
/// （拡張子付き含む）を対象とする。
pub const ATTACHED_PREFIX_EXCLUSIONS: &[&str] = &["README", "LICENSE", "CHANGELOG", "CONTRIBUTING"];

/// Plugin 付属リソースの合計サイズ上限（バイト）。超過時は配置せず警告する。
pub const ATTACHED_RESOURCES_MAX_BYTES: u64 = 10 * 1024 * 1024;

#[cfg(test)]
#[path = "placement_names_test.rs"]
mod tests;

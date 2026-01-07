//! コンポーネントスキャン共通関数
//!
//! SKILL.md を持つサブディレクトリの列挙など、複数箇所で使用される
//! スキャンロジックを提供する。
//!
//! ## 主要API
//!
//! - [`scan_components`]: プラグインディレクトリからコンポーネントをスキャン（統一API）
//! - [`ComponentScan`]: スキャン結果（名前のみ）
//!
//! ## 低レベル関数
//!
//! - [`list_skill_names`], [`list_agent_names`], etc.: 個別コンポーネントのスキャン

mod components;
mod constants;

use crate::plugin::PluginManifest;
use std::path::Path;

// Re-exports
pub use components::{
    file_stem_name, list_agent_names, list_command_names, list_hook_names, list_markdown_names,
    list_skill_names,
};
pub use constants::{
    AGENT_SUFFIX, DEFAULT_AGENTS_DIR, DEFAULT_COMMANDS_DIR, DEFAULT_HOOKS_DIR,
    DEFAULT_INSTRUCTIONS_DIR, DEFAULT_INSTRUCTIONS_FILE, DEFAULT_SKILLS_DIR, MARKDOWN_SUFFIX,
    PROMPT_SUFFIX, SKILL_MANIFEST,
};

// ============================================================================
// 統一スキャンAPI
// ============================================================================

/// コンポーネントスキャン結果
///
/// プラグインディレクトリから検出されたコンポーネントの名前一覧。
/// パスは含まず、名前のみを保持する（パス解決は呼び出し側の責務）。
#[derive(Debug, Clone, Default)]
pub struct ComponentScan {
    /// スキル名一覧
    pub skills: Vec<String>,
    /// エージェント名一覧
    pub agents: Vec<String>,
    /// コマンド名一覧
    pub commands: Vec<String>,
    /// インストラクション名一覧
    pub instructions: Vec<String>,
    /// フック名一覧
    pub hooks: Vec<String>,
}

impl ComponentScan {
    /// コンポーネントの総数を取得
    pub fn total_count(&self) -> usize {
        self.skills.len()
            + self.agents.len()
            + self.commands.len()
            + self.instructions.len()
            + self.hooks.len()
    }

    /// コンポーネントが存在するか
    pub fn has_components(&self) -> bool {
        self.total_count() > 0
    }
}

/// プラグインディレクトリからコンポーネントをスキャン
///
/// マニフェストのカスタムパス定義を尊重し、全コンポーネント種別を一括スキャン。
/// これが唯一の統一スキャンAPIであり、Application層はこれを使用すべき。
///
/// # Arguments
/// * `plugin_path` - プラグインのルートディレクトリ
/// * `manifest` - プラグインマニフェスト
///
/// # Returns
/// 検出されたコンポーネント名の一覧
pub fn scan_components(plugin_path: &Path, manifest: &PluginManifest) -> ComponentScan {
    ComponentScan {
        skills: scan_skills_internal(plugin_path, manifest),
        agents: scan_agents_internal(plugin_path, manifest),
        commands: scan_commands_internal(plugin_path, manifest),
        instructions: scan_instructions_internal(plugin_path, manifest),
        hooks: scan_hooks_internal(plugin_path, manifest),
    }
}

// ============================================================================
// 内部スキャン関数（scan_components から呼ばれる）
// ============================================================================

/// Skills をスキャン（内部用）
fn scan_skills_internal(plugin_path: &Path, manifest: &PluginManifest) -> Vec<String> {
    let skills_dir = manifest.skills_dir(plugin_path);
    list_skill_names(&skills_dir)
}

/// Agents をスキャン（内部用）
fn scan_agents_internal(plugin_path: &Path, manifest: &PluginManifest) -> Vec<String> {
    let agents_path = manifest.agents_dir(plugin_path);
    list_agent_names(&agents_path)
}

/// Commands をスキャン（内部用）
fn scan_commands_internal(plugin_path: &Path, manifest: &PluginManifest) -> Vec<String> {
    let commands_dir = manifest.commands_dir(plugin_path);
    list_command_names(&commands_dir)
}

/// Instructions をスキャン（内部用）
///
/// マニフェスト指定時: ファイルまたはディレクトリを走査。
/// 未指定時: instructions/ を走査 + ルートの AGENTS.md を追加。
fn scan_instructions_internal(plugin_path: &Path, manifest: &PluginManifest) -> Vec<String> {
    if let Some(path_str) = &manifest.instructions {
        // マニフェストで指定された場合
        let path = plugin_path.join(path_str);

        if path.is_file() {
            return file_stem_name(&path)
                .map(|name| vec![name])
                .unwrap_or_default();
        }

        if path.is_dir() {
            return list_markdown_names(&path);
        }

        return Vec::new();
    }

    // デフォルト: instructions/ ディレクトリ + AGENTS.md
    let instructions_dir = manifest.instructions_dir(plugin_path);
    let mut instructions = list_markdown_names(&instructions_dir);

    // ルートの AGENTS.md もチェック
    if plugin_path.join("AGENTS.md").exists() {
        instructions.push("AGENTS".to_string());
    }

    instructions
}

/// Hooks をスキャン（内部用）
fn scan_hooks_internal(plugin_path: &Path, manifest: &PluginManifest) -> Vec<String> {
    let hooks_dir = manifest.hooks_dir(plugin_path);
    list_hook_names(&hooks_dir)
}

//! コンポーネントスキャン共通関数
//!
//! ドメイン非依存のスキャンロジックを提供する。
//! Path と String のみに依存し、ドメイン型への変換はユースケース層で行う。
//!
//! ## 主要API
//!
//! - `scan_from_paths`: ドメイン非依存のスキャン（推奨）
//! - `scan_components`: プラグインディレクトリからコンポーネントをスキャン（レガシー）
//! - `ComponentScan`: スキャン結果（名前のみ）
//!
//! ## 配置スキャン
//!
//! - `list_placed_plugins`: 配置済みアイテムからプラグインを抽出
//! - `parse_placement`: 配置済みアイテム文字列をパース
//!
//! ## 低レベル関数
//!
//! - [`list_skill_names`], [`list_agent_names`], etc.: 個別コンポーネントのスキャン

mod components;
mod constants;
mod placement;

use crate::plugin::PluginManifest;
use std::path::{Path, PathBuf};

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
pub use placement::{list_placed_plugins, parse_placement};

// ============================================================================
// ドメイン非依存スキャン型
// ============================================================================

/// スキャン対象パス（ドメイン非依存）
///
/// PluginManifest 等のドメイン型からの変換はユースケース層で行う。
///
/// ## Note
///
/// `Default` は実装しない。空パスはカレントディレクトリを走査するリスクがあるため、
/// 常に明示的に構築すること。
#[derive(Debug, Clone)]
pub struct ScanPaths {
    /// スキルディレクトリ
    pub skills_dir: PathBuf,
    /// エージェントパス（ファイル or ディレクトリ）
    pub agents_path: PathBuf,
    /// コマンドディレクトリ
    pub commands_dir: PathBuf,
    /// インストラクション設定
    pub instructions: InstructionPath,
    /// フックディレクトリ
    pub hooks_dir: PathBuf,
}

/// インストラクションパス設定
///
/// ## Note
///
/// `Default` は実装しない。空パスはカレントディレクトリを走査するリスクがあるため、
/// 常に明示的に構築すること。
#[derive(Debug, Clone)]
pub enum InstructionPath {
    /// 単一ファイル
    File(PathBuf),
    /// ディレクトリ
    Dir(PathBuf),
    /// デフォルト（instructions/ + AGENTS.md）
    Default {
        instructions_dir: PathBuf,
        root_agents_md: PathBuf,
    },
}

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

/// コンポーネントをスキャン（ドメイン非依存）
///
/// 推奨API。ScanPaths を受け取り、ドメイン型への依存がない。
/// `PluginManifest` からの変換は `PluginManifest::to_scan_paths()` を使用。
///
/// # Arguments
/// * `paths` - スキャン対象パス
///
/// # Returns
/// 検出されたコンポーネント名の一覧
pub fn scan_from_paths(paths: &ScanPaths) -> ComponentScan {
    ComponentScan {
        skills: list_skill_names(&paths.skills_dir),
        agents: list_agent_names(&paths.agents_path),
        commands: list_command_names(&paths.commands_dir),
        instructions: scan_instructions(&paths.instructions),
        hooks: list_hook_names(&paths.hooks_dir),
    }
}

/// インストラクションをスキャン
fn scan_instructions(path: &InstructionPath) -> Vec<String> {
    match path {
        InstructionPath::File(p) => file_stem_name(p).map(|n| vec![n]).unwrap_or_default(),
        InstructionPath::Dir(p) => list_markdown_names(p),
        InstructionPath::Default {
            instructions_dir,
            root_agents_md,
        } => {
            let mut result = list_markdown_names(instructions_dir);
            if root_agents_md.exists() {
                result.push("AGENTS".to_string());
            }
            result
        }
    }
}

/// プラグインディレクトリからコンポーネントをスキャン
///
/// マニフェストのカスタムパス定義を尊重し、全コンポーネント種別を一括スキャン。
///
/// **Note**: 新規コードでは `scan_from_paths()` の使用を推奨。
/// このAPIは後方互換のために維持されている。
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

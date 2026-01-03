//! プラグインカタログ
//!
//! インストール済みプラグインの一覧取得ユースケースを提供する。

use crate::error::Result;
use crate::plugin::{has_manifest, PluginCache, PluginManifest};
use crate::scan::{
    file_stem_name, list_agent_names, list_command_names, list_hook_names, list_markdown_names,
    list_skill_names,
};
use std::path::Path;

/// プラグイン情報のサマリ（DTO）
#[derive(Debug, Clone)]
pub struct PluginSummary {
    /// プラグイン名
    pub name: String,
    /// マーケットプレイス名（マーケットプレイス経由の場合）
    pub marketplace: Option<String>,
    /// バージョン
    pub version: String,
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

impl PluginSummary {
    /// コンポーネントの総数を取得
    pub fn component_count(&self) -> usize {
        self.skills.len()
            + self.agents.len()
            + self.commands.len()
            + self.instructions.len()
            + self.hooks.len()
    }

    /// コンポーネントが存在するか
    pub fn has_components(&self) -> bool {
        self.component_count() > 0
    }
}

/// インストール済みプラグインの一覧を取得
///
/// キャッシュディレクトリをスキャンし、有効なプラグインの一覧を返す。
pub fn list_installed_plugins() -> Result<Vec<PluginSummary>> {
    let cache = PluginCache::new()?;
    let plugin_list = cache.list()?;

    let mut plugins = Vec::new();

    for (marketplace, name) in plugin_list {
        // 隠しディレクトリやメタデータディレクトリは除外
        if name.starts_with('.') {
            continue;
        }

        let plugin_path = cache.plugin_path(marketplace.as_deref(), &name);

        // plugin.json が存在するもののみをプラグインとして扱う
        if !has_manifest(&plugin_path) {
            continue;
        }

        let manifest = match cache.load_manifest(marketplace.as_deref(), &name) {
            Ok(m) => m,
            Err(_) => continue,
        };

        plugins.push(build_summary(name, marketplace, &plugin_path, &manifest));
    }

    Ok(plugins)
}

/// PluginSummary を構築
fn build_summary(
    name: String,
    marketplace: Option<String>,
    plugin_path: &Path,
    manifest: &PluginManifest,
) -> PluginSummary {
    let scan = scan_components(plugin_path, manifest);

    PluginSummary {
        name,
        marketplace,
        version: manifest.version.clone(),
        skills: scan.skills,
        agents: scan.agents,
        commands: scan.commands,
        instructions: scan.instructions,
        hooks: scan.hooks,
    }
}

/// スキャン結果（内部用）
struct ScanResult {
    skills: Vec<String>,
    agents: Vec<String>,
    instructions: Vec<String>,
    commands: Vec<String>,
    hooks: Vec<String>,
}

// ============================================================================
// コンポーネント別スキャン関数
// ============================================================================

/// Skills をスキャン
///
/// SKILL.md を持つサブディレクトリのみ抽出する。
fn scan_skills(plugin_path: &Path, manifest: &PluginManifest) -> Vec<String> {
    let skills_dir = manifest.skills_dir(plugin_path);
    list_skill_names(&skills_dir)
}

/// Agents をスキャン
///
/// 単一ファイルまたはディレクトリ内の .agent.md / .md ファイルを抽出する。
fn scan_agents(plugin_path: &Path, manifest: &PluginManifest) -> Vec<String> {
    let agents_path = manifest.agents_dir(plugin_path);
    list_agent_names(&agents_path)
}

/// Commands をスキャン
///
/// .prompt.md / .md ファイルを抽出する。
fn scan_commands(plugin_path: &Path, manifest: &PluginManifest) -> Vec<String> {
    let commands_dir = manifest.commands_dir(plugin_path);
    list_command_names(&commands_dir)
}

/// Instructions をスキャン
///
/// マニフェスト指定時: ファイルまたはディレクトリを走査。
/// 未指定時: instructions/ を走査 + ルートの AGENTS.md を追加。
fn scan_instructions(plugin_path: &Path, manifest: &PluginManifest) -> Vec<String> {
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

/// Hooks をスキャン
///
/// ファイル名から拡張子を除去して抽出する。
fn scan_hooks(plugin_path: &Path, manifest: &PluginManifest) -> Vec<String> {
    let hooks_dir = manifest.hooks_dir(plugin_path);
    list_hook_names(&hooks_dir)
}

// ============================================================================
// メイン関数
// ============================================================================

/// プラグインディレクトリからコンポーネントをスキャン
///
/// マニフェストのカスタムパス定義を尊重する。
fn scan_components(plugin_path: &Path, manifest: &PluginManifest) -> ScanResult {
    ScanResult {
        skills: scan_skills(plugin_path, manifest),
        agents: scan_agents(plugin_path, manifest),
        commands: scan_commands(plugin_path, manifest),
        instructions: scan_instructions(plugin_path, manifest),
        hooks: scan_hooks(plugin_path, manifest),
    }
}

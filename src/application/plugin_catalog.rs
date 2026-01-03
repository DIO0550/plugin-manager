//! プラグインカタログ
//!
//! インストール済みプラグインの一覧取得ユースケースを提供する。

use crate::error::Result;
use crate::path_ext::PathExt;
use crate::plugin::{has_manifest, PluginCache, PluginManifest};
use crate::scan::list_skill_names;
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
/// 読み取り専用操作のため、キャッシュディレクトリの作成は行わない。
pub fn list_installed_plugins() -> Result<Vec<PluginSummary>> {
    let cache = PluginCache::for_reading()?;
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

    // 単一ファイルの場合
    if agents_path.is_file() {
        return agents_path
            .file_stem()
            .and_then(|s| s.to_str())
            .map(|name| vec![name.to_string()])
            .unwrap_or_default();
    }

    if !agents_path.is_dir() {
        return Vec::new();
    }

    agents_path
        .read_dir_entries()
        .into_iter()
        .filter(|path| path.is_file())
        .filter_map(|path| {
            let name = path.file_name()?.to_str()?;
            if name.ends_with(".agent.md") {
                Some(name.trim_end_matches(".agent.md").to_string())
            } else if name.ends_with(".md") {
                Some(name.trim_end_matches(".md").to_string())
            } else {
                None
            }
        })
        .collect()
}

/// Commands をスキャン
///
/// .prompt.md / .md ファイルを抽出する。
fn scan_commands(plugin_path: &Path, manifest: &PluginManifest) -> Vec<String> {
    let commands_dir = manifest.commands_dir(plugin_path);

    if !commands_dir.is_dir() {
        return Vec::new();
    }

    commands_dir
        .read_dir_entries()
        .into_iter()
        .filter(|path| path.is_file())
        .filter_map(|path| {
            let name = path.file_name()?.to_str()?;
            if name.ends_with(".prompt.md") {
                Some(name.trim_end_matches(".prompt.md").to_string())
            } else if name.ends_with(".md") {
                Some(name.trim_end_matches(".md").to_string())
            } else {
                None
            }
        })
        .collect()
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
            return path
                .file_stem()
                .and_then(|s| s.to_str())
                .map(|name| vec![name.to_string()])
                .unwrap_or_default();
        }

        if path.is_dir() {
            return path
                .read_dir_entries()
                .into_iter()
                .filter(|p| p.is_file())
                .filter_map(|p| {
                    let name = p.file_name()?.to_str()?;
                    if name.ends_with(".md") {
                        Some(name.trim_end_matches(".md").to_string())
                    } else {
                        None
                    }
                })
                .collect();
        }

        return Vec::new();
    }

    // デフォルト: instructions/ ディレクトリ + AGENTS.md
    let mut instructions = Vec::new();

    let instructions_dir = manifest.instructions_dir(plugin_path);
    if instructions_dir.is_dir() {
        instructions.extend(
            instructions_dir
                .read_dir_entries()
                .into_iter()
                .filter(|p| p.is_file())
                .filter_map(|p| {
                    let name = p.file_name()?.to_str()?;
                    if name.ends_with(".md") {
                        Some(name.trim_end_matches(".md").to_string())
                    } else {
                        None
                    }
                }),
        );
    }

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

    if !hooks_dir.is_dir() {
        return Vec::new();
    }

    hooks_dir
        .read_dir_entries()
        .into_iter()
        .filter(|path| path.is_file())
        .filter_map(|path| {
            let name = path.file_name()?.to_str()?;
            let hook_name = name
                .rsplit_once('.')
                .map(|(n, _)| n.to_string())
                .unwrap_or_else(|| name.to_string());
            Some(hook_name)
        })
        .collect()
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

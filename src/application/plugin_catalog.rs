//! プラグインカタログ
//!
//! インストール済みプラグインの一覧取得ユースケースを提供する。

use crate::error::Result;
use crate::plugin::{has_manifest, PluginCache, PluginManifest};
use std::collections::HashSet;
use std::path::Path;

/// デフォルトのコンポーネントディレクトリパス
const DEFAULT_SKILLS_DIR: &str = "skills";
const DEFAULT_AGENTS_DIR: &str = "agents";
const DEFAULT_COMMANDS_DIR: &str = "commands";
const DEFAULT_INSTRUCTIONS_DIR: &str = "instructions";
const DEFAULT_HOOKS_DIR: &str = "hooks";

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
    let mut seen_marketplaces = HashSet::new();

    for (marketplace, name) in plugin_list {
        // 隠しディレクトリやメタデータディレクトリは除外
        if name.starts_with('.') {
            // ただし、マーケットプレイスディレクトリ自体がプラグインの場合をチェック
            if let Some(mp) = &marketplace {
                if !seen_marketplaces.contains(mp) {
                    seen_marketplaces.insert(mp.clone());
                    // マーケットプレイスルートが直接プラグインかチェック
                    let mp_path = cache.plugin_path(Some(mp), "").parent().unwrap().to_path_buf();
                    if has_manifest(&mp_path) {
                        let manifest = cache.load_manifest(Some(mp), "").ok();
                        let version = manifest
                            .as_ref()
                            .map(|m| m.version.clone())
                            .unwrap_or_else(|| "unknown".to_string());
                        let scan = scan_components(&mp_path, manifest.as_ref());
                        plugins.push(PluginSummary {
                            name: mp.clone(),
                            marketplace: Some(mp.clone()),
                            version,
                            skills: scan.skills,
                            agents: scan.agents,
                            commands: scan.commands,
                            instructions: scan.instructions,
                            hooks: scan.hooks,
                        });
                    }
                }
            }
            continue;
        }

        let plugin_path = cache.plugin_path(marketplace.as_deref(), &name);

        // plugin.json が存在するもののみをプラグインとして扱う
        if !has_manifest(&plugin_path) {
            continue;
        }

        let manifest = cache.load_manifest(marketplace.as_deref(), &name).ok();
        let version = manifest
            .as_ref()
            .map(|m| m.version.clone())
            .unwrap_or_else(|| "unknown".to_string());

        // コンポーネント情報を取得（マニフェストのカスタムパスを尊重）
        let scan = scan_components(&plugin_path, manifest.as_ref());

        plugins.push(PluginSummary {
            name,
            marketplace,
            version,
            skills: scan.skills,
            agents: scan.agents,
            commands: scan.commands,
            instructions: scan.instructions,
            hooks: scan.hooks,
        });
    }

    Ok(plugins)
}

/// スキャン結果（内部用）
struct ScanResult {
    skills: Vec<String>,
    agents: Vec<String>,
    instructions: Vec<String>,
    commands: Vec<String>,
    hooks: Vec<String>,
}

/// プラグインディレクトリからコンポーネントをスキャン
///
/// マニフェストが提供された場合、カスタムパス定義を尊重する。
/// マニフェストがない場合はデフォルトパスを使用する。
fn scan_components(plugin_path: &Path, manifest: Option<&PluginManifest>) -> ScanResult {
    let mut skills = Vec::new();
    let mut agents = Vec::new();
    let mut commands = Vec::new();
    let mut instructions = Vec::new();
    let mut hooks = Vec::new();

    // Skills: マニフェスト指定パス or skills/ ディレクトリ内の SKILL.md を持つディレクトリ
    let skills_dir = manifest
        .and_then(|m| m.skills.as_ref())
        .map(|p| plugin_path.join(p))
        .unwrap_or_else(|| plugin_path.join(DEFAULT_SKILLS_DIR));
    if skills_dir.is_dir() {
        if let Ok(entries) = std::fs::read_dir(&skills_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() && path.join("SKILL.md").exists() {
                    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                        skills.push(name.to_string());
                    }
                }
            }
        }
    }

    // Agents: マニフェスト指定パス or agents/ ディレクトリ内の .agent.md または .md ファイル
    let agents_path = manifest
        .and_then(|m| m.agents.as_ref())
        .map(|p| plugin_path.join(p))
        .unwrap_or_else(|| plugin_path.join(DEFAULT_AGENTS_DIR));
    // 単一ファイルの場合
    if agents_path.is_file() {
        if let Some(name) = agents_path.file_stem().and_then(|s| s.to_str()) {
            agents.push(name.to_string());
        }
    } else if agents_path.is_dir() {
        if let Ok(entries) = std::fs::read_dir(&agents_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                        if name.ends_with(".agent.md") {
                            agents.push(name.trim_end_matches(".agent.md").to_string());
                        } else if name.ends_with(".md") {
                            agents.push(name.trim_end_matches(".md").to_string());
                        }
                    }
                }
            }
        }
    }

    // Commands: マニフェスト指定パス or commands/ ディレクトリ内の .prompt.md または .md ファイル
    let commands_dir = manifest
        .and_then(|m| m.commands.as_ref())
        .map(|p| plugin_path.join(p))
        .unwrap_or_else(|| plugin_path.join(DEFAULT_COMMANDS_DIR));
    if commands_dir.is_dir() {
        if let Ok(entries) = std::fs::read_dir(&commands_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                        if name.ends_with(".prompt.md") {
                            commands.push(name.trim_end_matches(".prompt.md").to_string());
                        } else if name.ends_with(".md") {
                            commands.push(name.trim_end_matches(".md").to_string());
                        }
                    }
                }
            }
        }
    }

    // Instructions: マニフェスト指定パス or instructions/ ディレクトリ内のファイル、またはルートの AGENTS.md
    let instructions_path = manifest
        .and_then(|m| m.instructions.as_ref())
        .map(|p| plugin_path.join(p));

    if let Some(path) = instructions_path {
        // マニフェストで指定された場合、単一ファイルまたはディレクトリ
        if path.is_file() {
            if let Some(name) = path.file_stem().and_then(|s| s.to_str()) {
                instructions.push(name.to_string());
            }
        } else if path.is_dir() {
            if let Ok(entries) = std::fs::read_dir(&path) {
                for entry in entries.flatten() {
                    let entry_path = entry.path();
                    if entry_path.is_file() {
                        if let Some(name) = entry_path.file_name().and_then(|n| n.to_str()) {
                            if name.ends_with(".md") {
                                instructions.push(name.trim_end_matches(".md").to_string());
                            }
                        }
                    }
                }
            }
        }
    } else {
        // デフォルト: instructions/ ディレクトリ
        let instructions_dir = plugin_path.join(DEFAULT_INSTRUCTIONS_DIR);
        if instructions_dir.is_dir() {
            if let Ok(entries) = std::fs::read_dir(&instructions_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_file() {
                        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                            if name.ends_with(".md") {
                                instructions.push(name.trim_end_matches(".md").to_string());
                            }
                        }
                    }
                }
            }
        }
        // ルートの AGENTS.md もチェック
        if plugin_path.join("AGENTS.md").exists() {
            instructions.push("AGENTS".to_string());
        }
    }

    // Hooks: マニフェスト指定パス or hooks/ ディレクトリ内のファイル
    let hooks_dir = manifest
        .and_then(|m| m.hooks.as_ref())
        .map(|p| plugin_path.join(p))
        .unwrap_or_else(|| plugin_path.join(DEFAULT_HOOKS_DIR));
    if hooks_dir.is_dir() {
        if let Ok(entries) = std::fs::read_dir(&hooks_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                        // 拡張子を除いた名前を取得
                        let hook_name = name
                            .rsplit_once('.')
                            .map(|(n, _)| n.to_string())
                            .unwrap_or_else(|| name.to_string());
                        hooks.push(hook_name);
                    }
                }
            }
        }
    }

    ScanResult {
        skills,
        agents,
        instructions,
        commands,
        hooks,
    }
}

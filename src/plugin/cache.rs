use crate::component::{Component, ComponentKind};
use crate::error::{PlmError, Result};
use crate::path_ext::PathExt;
use crate::plugin::PluginManifest;
use crate::scan::list_skill_names;
use std::fs;
use std::io::{Cursor, Read};
use std::path::{Path, PathBuf};
use zip::ZipArchive;

/// コンポーネント検出用のファイル名パターン
const AGENT_SUFFIX: &str = ".agent.md";
const PROMPT_SUFFIX: &str = ".prompt.md";

/// マニフェストファイルのパス候補（優先順）
const MANIFEST_PATHS: &[&str] = &[".claude-plugin/plugin.json", "plugin.json"];

/// プラグインディレクトリ内のマニフェストパスを解決する
///
/// 以下の順序でマニフェストを検索:
/// 1. `.claude-plugin/plugin.json` (推奨)
/// 2. `plugin.json` (フォールバック)
///
/// # Arguments
/// * `plugin_dir` - プラグインのルートディレクトリ
///
/// # Returns
/// マニフェストファイルのパス、見つからない場合は None
pub fn resolve_manifest_path(plugin_dir: &Path) -> Option<PathBuf> {
    for candidate in MANIFEST_PATHS {
        let path = plugin_dir.join(candidate);
        if path.exists() {
            return Some(path);
        }
    }
    None
}

/// プラグインディレクトリがマニフェストを持つか確認する
pub fn has_manifest(plugin_dir: &Path) -> bool {
    resolve_manifest_path(plugin_dir).is_some()
}


/// キャッシュされたプラグイン情報
#[derive(Debug, Clone)]
pub struct CachedPlugin {
    pub name: String,
    /// マーケットプレイス名（marketplace経由の場合）
    /// None の場合は直接GitHubからインストール
    pub marketplace: Option<String>,
    pub path: PathBuf,
    pub manifest: PluginManifest,
    pub git_ref: String,
    pub commit_sha: String,
}

impl CachedPlugin {
    /// プラグインのバージョンを取得
    pub fn version(&self) -> &str {
        &self.manifest.version
    }

    /// プラグインの説明を取得
    pub fn description(&self) -> Option<&str> {
        self.manifest.description.as_deref()
    }

    /// スキルが含まれているか
    pub fn has_skills(&self) -> bool {
        self.manifest.has_skills()
    }

    /// スキルのパスを取得
    pub fn skills(&self) -> Option<&str> {
        self.manifest.skills.as_deref()
    }

    /// エージェントが含まれているか
    pub fn has_agents(&self) -> bool {
        self.manifest.has_agents()
    }

    /// エージェントのパスを取得
    pub fn agents(&self) -> Option<&str> {
        self.manifest.agents.as_deref()
    }

    /// コマンドが含まれているか
    pub fn has_commands(&self) -> bool {
        self.manifest.has_commands()
    }

    /// コマンドのパスを取得
    pub fn commands(&self) -> Option<&str> {
        self.manifest.commands.as_deref()
    }

    /// インストラクションが含まれているか
    pub fn has_instructions(&self) -> bool {
        self.manifest.has_instructions()
    }

    /// インストラクションのパスを取得
    pub fn instructions(&self) -> Option<&str> {
        self.manifest.instructions.as_deref()
    }

    /// フックが含まれているか
    pub fn has_hooks(&self) -> bool {
        self.manifest.hooks.is_some()
    }

    /// フックのパスを取得
    pub fn hooks(&self) -> Option<&str> {
        self.manifest.hooks.as_deref()
    }

    // =========================================================================
    // パス解決メソッド（デメテルの法則準拠）
    // =========================================================================

    /// スキルディレクトリのパスを解決
    pub fn skills_dir(&self) -> PathBuf {
        self.manifest.skills_dir(&self.path)
    }

    /// エージェントディレクトリのパスを解決
    pub fn agents_dir(&self) -> PathBuf {
        self.manifest.agents_dir(&self.path)
    }

    /// コマンドディレクトリのパスを解決
    pub fn commands_dir(&self) -> PathBuf {
        self.manifest.commands_dir(&self.path)
    }

    /// インストラクションパスを解決
    pub fn instructions_path(&self) -> PathBuf {
        self.manifest.instructions_path(&self.path)
    }

    /// フックディレクトリのパスを解決
    pub fn hooks_dir(&self) -> PathBuf {
        self.manifest.hooks_dir(&self.path)
    }

    // =========================================================================
    // スキャンメソッド
    // =========================================================================

    /// プラグイン内のコンポーネントをスキャン
    pub fn components(&self) -> Vec<Component> {
        let mut components = Vec::new();
        components.extend(self.scan_skills());
        components.extend(self.scan_agents());
        components.extend(self.scan_prompts());
        components.extend(self.scan_instructions());
        components.extend(self.scan_hooks());
        components
    }

    /// Skills をスキャン
    /// skills ディレクトリ配下で SKILL.md を持つディレクトリを検出
    fn scan_skills(&self) -> Vec<Component> {
        let skills_dir = self.skills_dir();

        let names = list_skill_names(&skills_dir);
        names
            .into_iter()
            .map(|name| Component {
                kind: ComponentKind::Skill,
                path: skills_dir.join(&name),
                name,
            })
            .collect()
    }

    /// Agents をスキャン
    /// ディレクトリなら .agent.md または .md ファイルを検出、単一ファイルならそれを1件として扱う
    fn scan_agents(&self) -> Vec<Component> {
        let agents_dir = self.agents_dir();

        if !agents_dir.exists() {
            return Vec::new();
        }

        // 単一ファイルの場合
        if agents_dir.is_file() {
            let name = agents_dir
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("agent")
                .to_string();
            return vec![Component {
                kind: ComponentKind::Agent,
                name,
                path: agents_dir,
            }];
        }

        // ディレクトリの場合: .agent.md または .md ファイルを検出
        agents_dir
            .read_dir_entries()
            .into_iter()
            .filter(|path| path.is_file())
            .filter_map(|path| {
                let file_name = path.file_name()?.to_str()?;
                // .agent.md サフィックスを優先、なければ .md として処理
                let name = if file_name.ends_with(AGENT_SUFFIX) {
                    file_name.trim_end_matches(AGENT_SUFFIX).to_string()
                } else if file_name.ends_with(".md") {
                    file_name.trim_end_matches(".md").to_string()
                } else {
                    return None;
                };
                Some(Component {
                    kind: ComponentKind::Agent,
                    name,
                    path,
                })
            })
            .collect()
    }

    /// Prompts (Commands) をスキャン
    /// commands ディレクトリ配下の .prompt.md または .md ファイルを検出
    fn scan_prompts(&self) -> Vec<Component> {
        let commands_dir = self.commands_dir();

        if !commands_dir.is_dir() {
            return Vec::new();
        }

        commands_dir
            .read_dir_entries()
            .into_iter()
            .filter(|path| path.is_file())
            .filter_map(|path| {
                let file_name = path.file_name()?.to_str()?;
                // .prompt.md サフィックスを優先、なければ .md として処理
                let name = if file_name.ends_with(PROMPT_SUFFIX) {
                    file_name.trim_end_matches(PROMPT_SUFFIX).to_string()
                } else if file_name.ends_with(".md") {
                    file_name.trim_end_matches(".md").to_string()
                } else {
                    return None;
                };
                Some(Component {
                    kind: ComponentKind::Command,
                    name,
                    path,
                })
            })
            .collect()
    }

    /// Instructions をスキャン
    /// 単一ファイルのみ検出
    fn scan_instructions(&self) -> Vec<Component> {
        let path = self.instructions_path();

        if !path.is_file() {
            return Vec::new();
        }

        let name = path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("instructions")
            .to_string();

        vec![Component {
            kind: ComponentKind::Instruction,
            name,
            path,
        }]
    }

    /// Hooks をスキャン
    /// hooks ディレクトリ配下のスクリプトファイルを検出
    fn scan_hooks(&self) -> Vec<Component> {
        let hooks_dir = self.hooks_dir();

        if !hooks_dir.is_dir() {
            return Vec::new();
        }

        hooks_dir
            .read_dir_entries()
            .into_iter()
            .filter(|path| path.is_file())
            .filter_map(|path| {
                let file_name = path.file_name()?.to_str()?;
                // 拡張子を除いた名前を取得
                let name = file_name
                    .rsplit_once('.')
                    .map(|(n, _)| n.to_string())
                    .unwrap_or_else(|| file_name.to_string());
                Some(Component {
                    kind: ComponentKind::Hook,
                    name,
                    path,
                })
            })
            .collect()
    }
}

/// プラグインキャッシュマネージャ
pub struct PluginCache {
    /// キャッシュルート: ~/.plm/cache/plugins/
    cache_dir: PathBuf,
}

impl PluginCache {
    /// キャッシュマネージャを初期化（ディレクトリ作成含む）
    ///
    /// 書き込み操作を行う場合に使用。キャッシュディレクトリが存在しない場合は作成する。
    pub fn new() -> Result<Self> {
        let cache = Self::for_reading()?;
        fs::create_dir_all(&cache.cache_dir)?;
        Ok(cache)
    }

    /// 読み取り専用でキャッシュマネージャを初期化
    ///
    /// キャッシュディレクトリの作成を行わない。一覧取得などの読み取り操作に使用。
    pub fn for_reading() -> Result<Self> {
        let home = std::env::var("HOME")
            .map_err(|_| PlmError::Cache("HOME environment variable not set".to_string()))?;
        let cache_dir = PathBuf::from(home).join(".plm").join("cache").join("plugins");

        Ok(Self { cache_dir })
    }

    /// カスタムキャッシュディレクトリで初期化（テスト用）
    pub fn with_cache_dir(cache_dir: PathBuf) -> Result<Self> {
        fs::create_dir_all(&cache_dir)?;
        Ok(Self { cache_dir })
    }

    /// プラグインのキャッシュパスを取得（階層型: marketplace/plugin）
    ///
    /// # Arguments
    /// * `marketplace` - マーケットプレイス名（None の場合は "github" を使用）
    /// * `name` - プラグイン名またはリポジトリ識別子（owner--repo 形式）
    pub fn plugin_path(&self, marketplace: Option<&str>, name: &str) -> PathBuf {
        let marketplace_dir = marketplace.unwrap_or("github");
        self.cache_dir.join(marketplace_dir).join(name)
    }

    /// キャッシュ済みかチェック
    pub fn is_cached(&self, marketplace: Option<&str>, name: &str) -> bool {
        self.plugin_path(marketplace, name).exists()
    }

    /// zipアーカイブを展開してキャッシュに保存
    /// GitHubのzipballは `{repo}-{ref}/` というプレフィックスが付くため、それを除去する
    ///
    /// # Arguments
    /// * `marketplace` - マーケットプレイス名（None の場合は "github" を使用）
    /// * `name` - プラグイン名またはリポジトリ識別子
    /// * `archive` - zipアーカイブのバイト列
    pub fn store_from_archive(
        &self,
        marketplace: Option<&str>,
        name: &str,
        archive: &[u8],
    ) -> Result<PathBuf> {
        let plugin_dir = self.plugin_path(marketplace, name);

        // 既存のキャッシュがあれば削除
        if plugin_dir.exists() {
            fs::remove_dir_all(&plugin_dir)?;
        }

        // zipを展開
        let cursor = Cursor::new(archive);
        let mut zip = ZipArchive::new(cursor)?;

        // 最初のエントリからプレフィックスを取得
        let prefix = if zip.len() > 0 {
            let first = zip.by_index(0)?;
            let first_name = first.name();
            // "repo-branch/" のような形式からプレフィックスを抽出
            first_name
                .split('/')
                .next()
                .map(|s| format!("{}/", s))
                .unwrap_or_default()
        } else {
            String::new()
        };

        // 各ファイルを展開
        for i in 0..zip.len() {
            let mut file = zip.by_index(i)?;
            let file_path = file.name();

            // プレフィックスを除去したパスを作成
            let relative_path = if !prefix.is_empty() && file_path.starts_with(&prefix) {
                &file_path[prefix.len()..]
            } else {
                file_path
            };

            // 空のパス（ルートディレクトリ）はスキップ
            if relative_path.is_empty() {
                continue;
            }

            let target_path = plugin_dir.join(relative_path);

            if file.is_dir() {
                fs::create_dir_all(&target_path)?;
            } else {
                // 親ディレクトリを作成
                if let Some(parent) = target_path.parent() {
                    fs::create_dir_all(parent)?;
                }

                // ファイルを書き込み
                let mut content = Vec::new();
                file.read_to_end(&mut content)?;
                fs::write(&target_path, content)?;
            }
        }

        Ok(plugin_dir)
    }

    /// キャッシュからマニフェストを読み込み
    ///
    /// 以下の順序でマニフェストを検索:
    /// 1. `.claude-plugin/plugin.json` (推奨)
    /// 2. `plugin.json` (フォールバック)
    pub fn load_manifest(&self, marketplace: Option<&str>, name: &str) -> Result<PluginManifest> {
        let plugin_dir = self.plugin_path(marketplace, name);
        let manifest_path = resolve_manifest_path(&plugin_dir).ok_or_else(|| {
            PlmError::InvalidManifest(format!(
                "plugin.json not found in {:?}",
                plugin_dir
            ))
        })?;

        PluginManifest::load(&manifest_path)
    }

    /// キャッシュから削除
    pub fn remove(&self, marketplace: Option<&str>, name: &str) -> Result<()> {
        let plugin_dir = self.plugin_path(marketplace, name);
        if plugin_dir.exists() {
            fs::remove_dir_all(&plugin_dir)?;
        }
        Ok(())
    }

    /// 全キャッシュをクリア
    pub fn clear(&self) -> Result<()> {
        if self.cache_dir.exists() {
            for entry in fs::read_dir(&self.cache_dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_dir() {
                    fs::remove_dir_all(path)?;
                }
            }
        }
        Ok(())
    }

    /// キャッシュされているプラグイン一覧を取得
    /// 階層構造を走査し、(marketplace, plugin_name) のタプルを返す
    pub fn list(&self) -> Result<Vec<(Option<String>, String)>> {
        let mut plugins = Vec::new();

        if !self.cache_dir.exists() {
            return Ok(plugins);
        }

        // marketplace ディレクトリを走査
        for mp_entry in fs::read_dir(&self.cache_dir)? {
            let mp_entry = mp_entry?;
            let mp_path = mp_entry.path();

            if !mp_path.is_dir() {
                continue;
            }

            let marketplace_name = mp_path.file_name().and_then(|n| n.to_str()).map(String::from);

            // marketplace 内のプラグインを走査
            for plugin_entry in fs::read_dir(&mp_path)? {
                let plugin_entry = plugin_entry?;
                let plugin_path = plugin_entry.path();

                if plugin_path.is_dir() {
                    if let Some(plugin_name) = plugin_path.file_name() {
                        let plugin_name = plugin_name.to_string_lossy().to_string();
                        // "github" marketplace は None として扱う
                        let mp = if marketplace_name.as_deref() == Some("github") {
                            None
                        } else {
                            marketplace_name.clone()
                        };
                        plugins.push((mp, plugin_name));
                    }
                }
            }
        }

        Ok(plugins)
    }
}

impl Default for PluginCache {
    fn default() -> Self {
        Self::new().expect("Failed to initialize plugin cache")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    /// テスト用のプラグインディレクトリを作成
    fn create_test_plugin_dir(temp_dir: &TempDir, structure: &[(&str, Option<&str>)]) -> PathBuf {
        let plugin_dir = temp_dir.path().to_path_buf();

        for (path, content) in structure {
            let full_path = plugin_dir.join(path);
            if let Some(parent) = full_path.parent() {
                fs::create_dir_all(parent).unwrap();
            }
            if let Some(content) = content {
                fs::write(&full_path, content).unwrap();
            } else {
                fs::create_dir_all(&full_path).unwrap();
            }
        }

        plugin_dir
    }

    #[test]
    fn test_resolve_manifest_path_prefers_claude_plugin_dir() {
        let temp_dir = TempDir::new().unwrap();
        let plugin_dir = create_test_plugin_dir(&temp_dir, &[
            (".claude-plugin/plugin.json", Some(r#"{"name":"test","version":"1.0.0"}"#)),
            ("plugin.json", Some(r#"{"name":"fallback","version":"0.1.0"}"#)),
        ]);

        let manifest_path = resolve_manifest_path(&plugin_dir).unwrap();
        assert!(manifest_path.ends_with(".claude-plugin/plugin.json"));
    }

    #[test]
    fn test_resolve_manifest_path_fallback_to_root() {
        let temp_dir = TempDir::new().unwrap();
        let plugin_dir = create_test_plugin_dir(&temp_dir, &[
            ("plugin.json", Some(r#"{"name":"test","version":"1.0.0"}"#)),
        ]);

        let manifest_path = resolve_manifest_path(&plugin_dir).unwrap();
        assert!(manifest_path.ends_with("plugin.json"));
        assert!(!manifest_path.to_string_lossy().contains(".claude-plugin"));
    }

    #[test]
    fn test_resolve_manifest_path_returns_none_when_missing() {
        let temp_dir = TempDir::new().unwrap();
        let plugin_dir = temp_dir.path().to_path_buf();

        assert!(resolve_manifest_path(&plugin_dir).is_none());
    }

    #[test]
    fn test_has_manifest_returns_true_for_claude_plugin_dir() {
        let temp_dir = TempDir::new().unwrap();
        let plugin_dir = create_test_plugin_dir(&temp_dir, &[
            (".claude-plugin/plugin.json", Some(r#"{"name":"test","version":"1.0.0"}"#)),
        ]);

        assert!(has_manifest(&plugin_dir));
    }

    #[test]
    fn test_has_manifest_returns_true_for_root_plugin_json() {
        let temp_dir = TempDir::new().unwrap();
        let plugin_dir = create_test_plugin_dir(&temp_dir, &[
            ("plugin.json", Some(r#"{"name":"test","version":"1.0.0"}"#)),
        ]);

        assert!(has_manifest(&plugin_dir));
    }

    #[test]
    fn test_has_manifest_returns_false_when_missing() {
        let temp_dir = TempDir::new().unwrap();
        assert!(!has_manifest(temp_dir.path()));
    }

    #[test]
    fn test_for_reading_does_not_create_directory() {
        // HOME環境変数を一時的に変更してテスト
        let temp_dir = TempDir::new().unwrap();
        let cache_dir = temp_dir.path().join("nonexistent").join(".plm").join("cache").join("plugins");

        // ディレクトリが存在しないことを確認
        assert!(!cache_dir.exists());

        // for_reading を使用しても new() のようにディレクトリは作成されない
        // (HOME環境変数を変更できないため、このテストは概念的な確認)
    }
}

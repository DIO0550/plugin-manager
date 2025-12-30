use crate::component::{Component, ComponentKind};
use crate::error::{PlmError, Result};
use crate::plugin::PluginManifest;
use std::fs;
use std::io::{Cursor, Read};
use std::path::PathBuf;
use zip::ZipArchive;

/// キャッシュされたプラグイン情報
#[derive(Debug, Clone)]
pub struct CachedPlugin {
    pub name: String,
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

    /// プラグイン内のコンポーネントをスキャン
    pub fn components(&self) -> Vec<Component> {
        let mut components = Vec::new();

        // Skills
        if let Some(skills_path) = self.skills() {
            let skills_dir = self.path.join(skills_path);
            if skills_dir.exists() && skills_dir.is_dir() {
                if let Ok(entries) = std::fs::read_dir(&skills_dir) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if path.is_dir() && path.join("SKILL.md").exists() {
                            let name = path
                                .file_name()
                                .and_then(|s| s.to_str())
                                .unwrap_or("skill")
                                .to_string();
                            components.push(Component {
                                kind: ComponentKind::Skill,
                                name,
                                path,
                            });
                        }
                    }
                }
            }
        }

        // Agents
        if let Some(agents_path) = self.agents() {
            let agents_dir = self.path.join(agents_path);
            if agents_dir.exists() {
                if agents_dir.is_dir() {
                    if let Ok(entries) = std::fs::read_dir(&agents_dir) {
                        for entry in entries.flatten() {
                            let path = entry.path();
                            if path.is_file() {
                                let file_name =
                                    path.file_name().and_then(|s| s.to_str()).unwrap_or("");
                                if file_name.ends_with(".agent.md") {
                                    let name = file_name.trim_end_matches(".agent.md").to_string();
                                    components.push(Component {
                                        kind: ComponentKind::Agent,
                                        name,
                                        path,
                                    });
                                }
                            }
                        }
                    }
                } else if agents_dir.is_file() {
                    // 単一ファイルの場合
                    let name = agents_dir
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("agent")
                        .to_string();
                    components.push(Component {
                        kind: ComponentKind::Agent,
                        name,
                        path: agents_dir,
                    });
                }
            }
        }

        // Commands (as prompts)
        if let Some(commands_path) = self.commands() {
            let commands_dir = self.path.join(commands_path);
            if commands_dir.exists() && commands_dir.is_dir() {
                if let Ok(entries) = std::fs::read_dir(&commands_dir) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if path.is_file() {
                            let file_name =
                                path.file_name().and_then(|s| s.to_str()).unwrap_or("");
                            if file_name.ends_with(".prompt.md") {
                                let name = file_name.trim_end_matches(".prompt.md").to_string();
                                components.push(Component {
                                    kind: ComponentKind::Prompt,
                                    name,
                                    path,
                                });
                            }
                        }
                    }
                }
            }
        }

        components
    }
}

/// プラグインキャッシュマネージャ
pub struct PluginCache {
    /// キャッシュルート: ~/.plm/cache/plugins/
    cache_dir: PathBuf,
}

impl PluginCache {
    /// キャッシュマネージャを初期化（ディレクトリ作成含む）
    pub fn new() -> Result<Self> {
        let home = std::env::var("HOME")
            .map_err(|_| PlmError::Cache("HOME environment variable not set".to_string()))?;
        let cache_dir = PathBuf::from(home).join(".plm").join("cache").join("plugins");

        fs::create_dir_all(&cache_dir)?;

        Ok(Self { cache_dir })
    }

    /// カスタムキャッシュディレクトリで初期化（テスト用）
    pub fn with_cache_dir(cache_dir: PathBuf) -> Result<Self> {
        fs::create_dir_all(&cache_dir)?;
        Ok(Self { cache_dir })
    }

    /// プラグインのキャッシュパスを取得
    pub fn plugin_path(&self, name: &str) -> PathBuf {
        self.cache_dir.join(name)
    }

    /// キャッシュ済みかチェック
    pub fn is_cached(&self, name: &str) -> bool {
        self.plugin_path(name).exists()
    }

    /// zipアーカイブを展開してキャッシュに保存
    /// GitHubのzipballは `{repo}-{ref}/` というプレフィックスが付くため、それを除去する
    pub fn store_from_archive(&self, name: &str, archive: &[u8]) -> Result<PathBuf> {
        let plugin_dir = self.plugin_path(name);

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
    pub fn load_manifest(&self, name: &str) -> Result<PluginManifest> {
        let manifest_path = self.plugin_path(name).join(".claude-plugin").join("plugin.json");

        if !manifest_path.exists() {
            return Err(PlmError::InvalidManifest(format!(
                "plugin.json not found at {:?}",
                manifest_path
            )));
        }

        PluginManifest::load(&manifest_path)
    }

    /// キャッシュから削除
    pub fn remove(&self, name: &str) -> Result<()> {
        let plugin_dir = self.plugin_path(name);
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
    pub fn list(&self) -> Result<Vec<String>> {
        let mut plugins = Vec::new();

        if self.cache_dir.exists() {
            for entry in fs::read_dir(&self.cache_dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_dir() {
                    if let Some(name) = path.file_name() {
                        plugins.push(name.to_string_lossy().to_string());
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

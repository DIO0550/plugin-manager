//! PLM メタデータ管理
//!
//! プラグインのインストール日時などPLM固有のメタデータを `.plm-meta.json` で管理する。
//! `plugin.json` は上流成果物として改変しない設計。

use crate::error::Result;
use crate::fs::{FileSystem, RealFs};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::io::Write;
use std::path::Path;
use tempfile::NamedTempFile;

/// メタデータファイル名
const META_FILE: &str = ".plm-meta.json";

/// PLMが管理するプラグインメタデータ
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PluginMeta {
    /// インストール日時（RFC3339形式）
    /// 欠損時は None として扱う
    #[serde(default, rename = "installedAt")]
    pub installed_at: Option<String>,

    /// ターゲット別ステータス（"enabled" / "disabled"）
    /// 空の場合はシリアライズ時に省略
    #[serde(
        default,
        rename = "statusByTarget",
        skip_serializing_if = "HashMap::is_empty"
    )]
    pub status_by_target: HashMap<String, String>,

    /// Git参照（ブランチ名やタグ）
    #[serde(default, rename = "gitRef", skip_serializing_if = "Option::is_none")]
    pub git_ref: Option<String>,

    /// コミットSHA
    #[serde(default, rename = "commitSha", skip_serializing_if = "Option::is_none")]
    pub commit_sha: Option<String>,

    /// 更新日時（RFC3339形式）
    #[serde(default, rename = "updatedAt", skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<String>,

    /// ソースリポジトリ情報（owner/repo形式）
    #[serde(
        default,
        rename = "sourceRepo",
        skip_serializing_if = "Option::is_none"
    )]
    pub source_repo: Option<String>,

    /// マーケットプレイス（"github" 固定、将来拡張用）
    #[serde(
        default,
        rename = "marketplace",
        skip_serializing_if = "Option::is_none"
    )]
    pub marketplace: Option<String>,
}

impl PluginMeta {
    /// 指定ターゲットのステータスを取得
    ///
    /// # Arguments
    ///
    /// * `target` - Target name (e.g. `"codex"`, `"copilot"`).
    pub fn get_status(&self, target: &str) -> Option<&str> {
        self.status_by_target.get(target).map(|s| s.as_str())
    }

    /// 指定ターゲットのステータスを設定
    ///
    /// # Arguments
    ///
    /// * `target` - Target name.
    /// * `status` - Status string (`"enabled"` or `"disabled"`).
    pub fn set_status(&mut self, target: &str, status: &str) {
        self.status_by_target
            .insert(target.to_string(), status.to_string());
    }

    /// 指定ターゲットが有効化されているか
    ///
    /// # Arguments
    ///
    /// * `target` - Target name.
    pub fn is_enabled(&self, target: &str) -> bool {
        self.get_status(target) == Some("enabled")
    }

    /// いずれかのターゲットが有効化されているか
    pub fn any_enabled(&self) -> bool {
        self.status_by_target.values().any(|s| s == "enabled")
    }

    /// Git参照情報を更新
    ///
    /// # Arguments
    ///
    /// * `git_ref` - Git reference (branch name or tag).
    /// * `commit_sha` - Commit SHA to record.
    pub fn set_git_info(&mut self, git_ref: &str, commit_sha: &str) {
        self.git_ref = Some(git_ref.to_string());
        self.commit_sha = Some(commit_sha.to_string());
        self.updated_at = Some(Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string());
    }

    /// 有効なターゲット一覧を取得
    pub fn enabled_targets(&self) -> Vec<&str> {
        self.status_by_target
            .iter()
            .filter(|(_, status)| *status == "enabled")
            .map(|(target, _)| target.as_str())
            .collect()
    }

    /// ソースリポジトリを設定
    ///
    /// # Arguments
    ///
    /// * `owner` - Repository owner name.
    /// * `repo` - Repository name.
    pub fn set_source_repo(&mut self, owner: &str, repo: &str) {
        self.source_repo = Some(format!("{}/{}", owner, repo));
    }

    /// ソースリポジトリを取得
    pub fn get_source_repo(&self) -> Option<(&str, &str)> {
        self.source_repo.as_ref().and_then(|s| s.split_once('/'))
    }

    /// GitHub プラグインかどうか
    pub fn is_github(&self) -> bool {
        self.marketplace.as_deref() == Some("github") || self.marketplace.is_none()
    }
}

/// installedAt の正規化
/// 空文字/空白のみ → None、それ以外 → Some(trimmed)
///
/// # Arguments
///
/// * `value` - Optional string to normalize (`None` when unset).
fn normalize_installed_at(value: Option<&str>) -> Option<String> {
    value
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(String::from)
}

/// メタデータを書き込む（アトミック書き込み）
///
/// 同一ディレクトリに一時ファイルを作成し、persist() でリネームする。
/// 書き込み失敗時は Err を返す（呼び出し側で警告ログ + 継続を判断）。
///
/// # Arguments
///
/// * `plugin_dir` - Plugin root directory where the metadata file lives.
/// * `meta` - `PluginMeta` value to serialize and persist.
pub fn write_meta(plugin_dir: &Path, meta: &PluginMeta) -> Result<()> {
    let meta_path = plugin_dir.join(META_FILE);

    let mut temp_file = NamedTempFile::new_in(plugin_dir)?;

    let json = serde_json::to_string_pretty(meta)?;
    temp_file.write_all(json.as_bytes())?;
    temp_file.flush()?;

    // Windows では既存ファイルがあると persist が失敗する可能性があるため、
    // エラー時は既存ファイルを削除してから再試行する
    match temp_file.persist(&meta_path) {
        Ok(_) => Ok(()),
        Err(e) => {
            if e.error.kind() == std::io::ErrorKind::AlreadyExists {
                let fs = RealFs;
                let _ = fs.remove_file(&meta_path);
                e.file.persist(&meta_path).map_err(|e| e.error)?;
                Ok(())
            } else {
                Err(e.error.into())
            }
        }
    }
}

/// 現在時刻で installedAt を設定したメタデータを書き込む
///
/// # Arguments
///
/// * `plugin_dir` - Plugin root directory where the metadata file lives.
pub fn write_installed_at(plugin_dir: &Path) -> Result<()> {
    let now = Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
    let meta = PluginMeta {
        installed_at: Some(now),
        ..Default::default()
    };
    write_meta(plugin_dir, &meta)
}

/// メタデータを読み込む
///
/// 欠損時は None、破損時は警告ログを出力して None を返す。
/// 読み取り時は副作用なし（`.bak` 退避などを行わない）。
///
/// # Arguments
///
/// * `plugin_dir` - Plugin root directory where the metadata file lives.
pub fn load_meta(plugin_dir: &Path) -> Option<PluginMeta> {
    let fs = RealFs;
    let meta_path = plugin_dir.join(META_FILE);

    if !fs.exists(&meta_path) {
        return None;
    }

    match fs.read_to_string(&meta_path) {
        Ok(content) => match serde_json::from_str::<PluginMeta>(&content) {
            Ok(meta) => Some(meta),
            Err(e) => {
                eprintln!(
                    "Warning: {} is corrupted ({}); plugin metadata will be unavailable. \
                     It will be regenerated on next install.",
                    META_FILE, e
                );
                None
            }
        },
        Err(e) => {
            eprintln!(
                "Warning: Failed to read {} ({}); plugin metadata will be unavailable.",
                META_FILE, e
            );
            None
        }
    }
}

/// installedAt を `.plm-meta.json` から取得する
///
/// `.plm-meta.json` が存在し `installedAt` が空でない場合のみ値を返す。
/// 旧 `plugin.json` の `installedAt` フィールドは廃止済みのため参照しない。
///
/// # Arguments
///
/// * `plugin_dir` - Plugin root directory where the metadata file lives.
pub fn resolve_installed_at(plugin_dir: &Path) -> Option<String> {
    load_meta(plugin_dir).and_then(|meta| normalize_installed_at(meta.installed_at.as_deref()))
}

/// プラグインが有効かどうかを判定
///
/// 判定ロジック:
/// 1. `.plm-meta.json` の `statusByTarget` を優先参照
///    - ファイルが存在し、statusByTarget が空でない場合: いずれかのターゲットが enabled なら true
/// 2. `.plm-meta.json` が存在しない/読み取りエラー/statusByTarget が空の場合:
///    実デプロイ状態から判定（後方互換）
///
/// 後方互換ロジックは `manifest.name` を `flattened_name` 集合の prefix として
/// マッチさせる。フラット化後の `flattened_name` は `"{plugin_name}_{...}"`
/// 形式のため、境界付き prefix `"{plugin_name}_"` で始まるエントリが
/// 1 件でもあれば「このプラグイン由来のコンポーネントが配置済み」と判定する
/// （末尾 `_` を含めることで `plugin_name` と他プラグイン名の部分一致を防ぐ）。
///
/// # Arguments
///
/// * `cache_path` - プラグインのキャッシュパス
/// * `plugin_name` - プラグイン名（`PluginManifest.name` 相当）
/// * `deployed` - デプロイ済みコンポーネントの `flattened_name` 集合
///
/// # Returns
/// `true` if enabled, `false` otherwise
pub fn is_enabled(cache_path: &Path, plugin_name: &str, deployed: &HashSet<String>) -> bool {
    if let Some(plugin_meta) = load_meta(cache_path) {
        if !plugin_meta.status_by_target.is_empty() {
            return plugin_meta.any_enabled();
        }
    }

    // 後方互換: statusByTarget が無い古いプラグインは flattened_name の prefix
    // マッチで判定する。`flattened_name = "{plugin}_{original}"` なので、
    // `plugin_name` で始まるエントリが 1 件でもあれば enabled と判定する。
    let prefix = format!("{}_", plugin_name);
    deployed.iter().any(|name| name.starts_with(&prefix))
}

/// `is_enabled` の O(1) バリアント。プラグイン一覧を一括判定する経路で使う。
///
/// 事前に [`build_deployed_plugin_set`] で構築した
/// 「配置済みコンポーネントを少なくとも 1 件持つ plugin_name 集合」を渡し、
/// `deployed_plugins.contains(plugin_name)` の定数時間ルックアップに置換する。
/// `.plm-meta.json` の `statusByTarget` を優先する判定ロジックは `is_enabled`
/// と同一。
///
/// # Arguments
///
/// * `cache_path` - プラグインのキャッシュパス
/// * `plugin_name` - `PluginManifest.name`
/// * `deployed_plugins` - 配置済みコンポーネントを持つ plugin_name の集合
pub fn is_enabled_indexed(
    cache_path: &Path,
    plugin_name: &str,
    deployed_plugins: &HashSet<String>,
) -> bool {
    if let Some(plugin_meta) = load_meta(cache_path) {
        if !plugin_meta.status_by_target.is_empty() {
            return plugin_meta.any_enabled();
        }
    }
    deployed_plugins.contains(plugin_name)
}

/// `deployed` の `flattened_name` 集合と既知の `plugin_names` から、
/// 配置済みコンポーネントを少なくとも 1 件持つ plugin_name の集合を構築する。
///
/// `flattened_name = "{plugin}_{original}"` の `plugin` 部分を `_` 区切りで
/// 走査し、`plugin_names` に存在する候補を収集する。`plugin_name` 自体が
/// `_` を含む場合（例 `"my_plugin"`）も全 `_` 位置を試行するため正しく拾える。
///
/// 計算量: `O(sum(|name|))` の文字走査と各 `_` 位置での `HashSet::contains`。
/// プラグイン数 N に対して `is_enabled` を呼ぶ経路では、従来の
/// `O(N × |deployed|)` を `O(sum(|name|) + N)` に改善する。
///
/// # 既知の曖昧さ（プレフィックス衝突）
///
/// `"{plugin}_{original}"` 形式は `_` を含む plugin / original では分解が
/// 一意にならない。例えば `plugin_names = {"foo", "foo_bar"}` で
/// `deployed = {"foo_bar_baz"}` の場合、`"foo"` も `"foo_bar"` もマッチする
/// ため両プラグインが enabled として返る。これは従来の
/// `is_enabled` の `prefix.starts_with` ロジックと同一の挙動で（後方互換）、
/// 「false positive を許容する best-effort なアッパーバウンド」として扱う。
/// 完全な一意性が必要な場合は `flatten_name` の区切り文字再設計
/// （非英数記号への変更や escape）または plugin_name の `_` 禁止が必要。
/// 衝突挙動は `test_build_deployed_plugin_set_prefix_collision` で固定する。
///
/// # Arguments
///
/// * `deployed` - `list_all_placed` が返す `flattened_name` 集合
/// * `plugin_names` - 既知の plugin_name 集合（通常はキャッシュから列挙）
pub fn build_deployed_plugin_set(
    deployed: &HashSet<String>,
    plugin_names: &HashSet<String>,
) -> HashSet<String> {
    deployed
        .iter()
        .flat_map(|name| underscore_prefix_candidates(name))
        .filter(|candidate| plugin_names.contains(*candidate))
        .map(str::to_string)
        .collect()
}

/// `name` 内の各 `_` 位置で区切った prefix 候補を列挙する。
///
/// 例: `"foo_bar_baz"` → `["foo", "foo_bar"]`
fn underscore_prefix_candidates(name: &str) -> impl Iterator<Item = &str> {
    name.char_indices()
        .filter(|(_, ch)| *ch == '_')
        .map(move |(idx, _)| &name[..idx])
}

#[cfg(test)]
#[path = "meta_test.rs"]
mod tests;

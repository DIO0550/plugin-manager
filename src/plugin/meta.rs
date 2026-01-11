//! PLM メタデータ管理
//!
//! プラグインのインストール日時などPLM固有のメタデータを `.plm-meta.json` で管理する。
//! `plugin.json` は上流成果物として改変しない設計。

use super::manifest_resolve::resolve_manifest_path;
use super::PluginManifest;
use crate::error::Result;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::fs;
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
}

/// installedAt の正規化
/// 空文字/空白のみ → None、それ以外 → Some(trimmed)
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
pub fn write_meta(plugin_dir: &Path, meta: &PluginMeta) -> Result<()> {
    let meta_path = plugin_dir.join(META_FILE);

    // 同一ディレクトリに一時ファイルを作成
    let mut temp_file = NamedTempFile::new_in(plugin_dir)?;

    // JSON を書き込み
    let json = serde_json::to_string_pretty(meta)?;
    temp_file.write_all(json.as_bytes())?;
    temp_file.flush()?;

    // persist() で置き換え
    // Windows では既存ファイルがあると失敗する可能性があるため、
    // エラー時は既存ファイルを削除してから再試行
    match temp_file.persist(&meta_path) {
        Ok(_) => Ok(()),
        Err(e) => {
            if e.error.kind() == std::io::ErrorKind::AlreadyExists {
                // 既存ファイルを削除して再試行
                let _ = fs::remove_file(&meta_path);
                e.file.persist(&meta_path).map_err(|e| e.error)?;
                Ok(())
            } else {
                Err(e.error.into())
            }
        }
    }
}

/// 現在時刻で installedAt を設定したメタデータを書き込む
pub fn write_installed_at(plugin_dir: &Path) -> Result<()> {
    let now = Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
    let meta = PluginMeta {
        installed_at: Some(now),
    };
    write_meta(plugin_dir, &meta)
}

/// メタデータを読み込む
///
/// 欠損時は None、破損時は警告ログを出力して None を返す。
/// 読み取り時は副作用なし（`.bak` 退避などを行わない）。
pub fn load_meta(plugin_dir: &Path) -> Option<PluginMeta> {
    let meta_path = plugin_dir.join(META_FILE);

    if !meta_path.exists() {
        return None;
    }

    match fs::read_to_string(&meta_path) {
        Ok(content) => match serde_json::from_str::<PluginMeta>(&content) {
            Ok(meta) => Some(meta),
            Err(e) => {
                eprintln!(
                    "Warning: {} is corrupted ({}), falling back to plugin.json. \
                     It will be regenerated on next install.",
                    META_FILE, e
                );
                None
            }
        },
        Err(e) => {
            eprintln!(
                "Warning: Failed to read {} ({}), falling back to plugin.json.",
                META_FILE, e
            );
            None
        }
    }
}

/// installedAt を取得（.plm-meta.json → plugin.json のフォールバック付き）
///
/// 優先順位:
/// 1. `.plm-meta.json` の `installedAt` を優先
/// 2. `.plm-meta.json` が無いか `installedAt` が欠損 → `plugin.json` の `installedAt` にフォールバック
/// 3. 両方無い → `None`
///
/// manifest が None の場合は plugin.json を読み込んでフォールバック
pub fn resolve_installed_at(plugin_dir: &Path, manifest: Option<&PluginManifest>) -> Option<String> {
    // 1. .plm-meta.json から取得を試みる
    if let Some(meta) = load_meta(plugin_dir) {
        if let Some(installed_at) = normalize_installed_at(meta.installed_at.as_deref()) {
            return Some(installed_at);
        }
    }

    // 2. plugin.json からフォールバック
    if let Some(m) = manifest {
        return normalize_installed_at(m.installed_at.as_deref());
    }

    // manifest が渡されていない場合は読み込みを試みる
    let loaded_manifest = resolve_manifest_path(plugin_dir)
        .and_then(|path| PluginManifest::load(&path).ok());

    if let Some(m) = loaded_manifest {
        return normalize_installed_at(m.installed_at.as_deref());
    }

    None
}

#[cfg(test)]
#[path = "meta_test.rs"]
mod tests;

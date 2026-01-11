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
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_normalize_installed_at_valid() {
        assert_eq!(
            normalize_installed_at(Some("2025-01-15T10:30:00Z")),
            Some("2025-01-15T10:30:00Z".to_string())
        );
    }

    #[test]
    fn test_normalize_installed_at_empty() {
        assert_eq!(normalize_installed_at(Some("")), None);
    }

    #[test]
    fn test_normalize_installed_at_whitespace() {
        assert_eq!(normalize_installed_at(Some("   ")), None);
    }

    #[test]
    fn test_normalize_installed_at_trimmed() {
        assert_eq!(
            normalize_installed_at(Some("  2025-01-15T10:30:00Z  ")),
            Some("2025-01-15T10:30:00Z".to_string())
        );
    }

    #[test]
    fn test_normalize_installed_at_none() {
        assert_eq!(normalize_installed_at(None), None);
    }

    #[test]
    fn test_write_and_load_meta() {
        let temp_dir = TempDir::new().unwrap();
        let plugin_dir = temp_dir.path();

        let meta = PluginMeta {
            installed_at: Some("2025-01-15T10:30:00Z".to_string()),
        };

        write_meta(plugin_dir, &meta).unwrap();

        let loaded = load_meta(plugin_dir).unwrap();
        assert_eq!(loaded.installed_at, Some("2025-01-15T10:30:00Z".to_string()));
    }

    #[test]
    fn test_write_installed_at() {
        let temp_dir = TempDir::new().unwrap();
        let plugin_dir = temp_dir.path();

        write_installed_at(plugin_dir).unwrap();

        let loaded = load_meta(plugin_dir).unwrap();
        assert!(loaded.installed_at.is_some());
        let installed_at = loaded.installed_at.unwrap();
        assert!(installed_at.contains("T"));
        assert!(installed_at.ends_with("Z"));
    }

    #[test]
    fn test_load_meta_not_exists() {
        let temp_dir = TempDir::new().unwrap();
        let loaded = load_meta(temp_dir.path());
        assert!(loaded.is_none());
    }

    #[test]
    fn test_load_meta_corrupted() {
        let temp_dir = TempDir::new().unwrap();
        let meta_path = temp_dir.path().join(META_FILE);

        // 破損したJSONを書き込む
        fs::write(&meta_path, "{ invalid json }").unwrap();

        let loaded = load_meta(temp_dir.path());
        assert!(loaded.is_none());
    }

    #[test]
    fn test_plugin_meta_serde() {
        let meta = PluginMeta {
            installed_at: Some("2025-01-15T10:30:00Z".to_string()),
        };

        let json = serde_json::to_string(&meta).unwrap();
        assert!(json.contains("installedAt"));

        let parsed: PluginMeta = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.installed_at, meta.installed_at);
    }

    #[test]
    fn test_plugin_meta_default() {
        let meta = PluginMeta::default();
        assert!(meta.installed_at.is_none());
    }

    #[test]
    fn test_resolve_installed_at_from_meta() {
        let temp_dir = TempDir::new().unwrap();
        let plugin_dir = temp_dir.path();

        // .plm-meta.json を作成
        let meta = PluginMeta {
            installed_at: Some("2025-01-15T10:30:00Z".to_string()),
        };
        write_meta(plugin_dir, &meta).unwrap();

        let result = resolve_installed_at(plugin_dir, None);
        assert_eq!(result, Some("2025-01-15T10:30:00Z".to_string()));
    }

    #[test]
    fn test_resolve_installed_at_fallback_to_manifest() {
        let temp_dir = TempDir::new().unwrap();
        let plugin_dir = temp_dir.path();

        // plugin.json を作成（.plm-meta.json なし）
        let manifest_content = r#"{"name":"test","version":"1.0.0","installedAt":"2025-01-10T00:00:00Z"}"#;
        fs::write(plugin_dir.join("plugin.json"), manifest_content).unwrap();

        let manifest = PluginManifest::parse(manifest_content).unwrap();
        let result = resolve_installed_at(plugin_dir, Some(&manifest));
        assert_eq!(result, Some("2025-01-10T00:00:00Z".to_string()));
    }

    #[test]
    fn test_resolve_installed_at_meta_priority() {
        let temp_dir = TempDir::new().unwrap();
        let plugin_dir = temp_dir.path();

        // 両方に値がある場合、.plm-meta.json が優先
        let meta = PluginMeta {
            installed_at: Some("2025-01-15T10:30:00Z".to_string()),
        };
        write_meta(plugin_dir, &meta).unwrap();

        let manifest_content = r#"{"name":"test","version":"1.0.0","installedAt":"2025-01-10T00:00:00Z"}"#;
        fs::write(plugin_dir.join("plugin.json"), manifest_content).unwrap();

        let manifest = PluginManifest::parse(manifest_content).unwrap();
        let result = resolve_installed_at(plugin_dir, Some(&manifest));
        assert_eq!(result, Some("2025-01-15T10:30:00Z".to_string()));
    }

    #[test]
    fn test_resolve_installed_at_empty_meta_fallback() {
        let temp_dir = TempDir::new().unwrap();
        let plugin_dir = temp_dir.path();

        // .plm-meta.json は空の installedAt
        let meta = PluginMeta {
            installed_at: Some("".to_string()),
        };
        write_meta(plugin_dir, &meta).unwrap();

        // plugin.json に値あり
        let manifest_content = r#"{"name":"test","version":"1.0.0","installedAt":"2025-01-10T00:00:00Z"}"#;
        fs::write(plugin_dir.join("plugin.json"), manifest_content).unwrap();

        let manifest = PluginManifest::parse(manifest_content).unwrap();
        let result = resolve_installed_at(plugin_dir, Some(&manifest));
        assert_eq!(result, Some("2025-01-10T00:00:00Z".to_string()));
    }

    #[test]
    fn test_resolve_installed_at_both_none() {
        let temp_dir = TempDir::new().unwrap();
        let plugin_dir = temp_dir.path();

        // plugin.json に installedAt なし
        let manifest_content = r#"{"name":"test","version":"1.0.0"}"#;
        fs::write(plugin_dir.join("plugin.json"), manifest_content).unwrap();

        let manifest = PluginManifest::parse(manifest_content).unwrap();
        let result = resolve_installed_at(plugin_dir, Some(&manifest));
        assert!(result.is_none());
    }
}

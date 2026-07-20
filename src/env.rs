/// 環境変数ユーティリティ
pub struct EnvVar;

impl EnvVar {
    /// 環境変数を取得（空文字列はNoneとして扱う）
    ///
    /// # Arguments
    ///
    /// * `key` - Name of the environment variable to read.
    pub fn get(key: &str) -> Option<String> {
        std::env::var(key).ok().filter(|s| !s.is_empty())
    }
}

use crate::error::{PlmError, Result};
use std::path::PathBuf;

/// PLM の状態ルートディレクトリを返す（案 A: HOME 代替セマンティクス）
///
/// 解決順: `PLM_HOME`（有効時 = 非空・非空白・絶対パス）→ `HOME`
/// - 空・空白のみは無効扱い（`EnvVar::get` 互換 + trim）
/// - 相対パスはエラーとして返す
/// - 両方無効なら明確なエラー
pub(crate) fn plm_root() -> Result<PathBuf> {
    let raw = EnvVar::get("PLM_HOME")
        .filter(|s| !s.trim().is_empty())
        .or_else(|| EnvVar::get("HOME").filter(|s| !s.trim().is_empty()))
        .ok_or_else(|| {
            PlmError::General(
                "PLM_HOME and HOME environment variables are not set or empty".to_string(),
            )
        })?;

    let path = PathBuf::from(raw.trim());
    if !path.is_absolute() {
        return Err(PlmError::General(format!(
            "PLM_HOME/HOME must be an absolute path, got: {}",
            path.display()
        )));
    }
    Ok(path)
}

/// PLM 状態ファイル群のパスを集約する値オブジェクト
///
/// `new()` で本番環境変数から構築、`with_root()` でテスト用パスを注入する。
pub(crate) struct PlmPaths {
    root: PathBuf,
}

impl PlmPaths {
    /// 環境変数（PLM_HOME → HOME）からパスを解決して構築する
    pub(crate) fn new() -> Result<Self> {
        Ok(Self { root: plm_root()? })
    }

    /// カスタムルートを指定して構築する（テスト用）
    ///
    /// # Arguments
    ///
    /// * `root` - PLM の状態ルートディレクトリ（`~` 相当）
    pub(crate) fn with_root(root: PathBuf) -> Self {
        Self { root }
    }

    /// PLM の状態ディレクトリ: `{root}/.plm`
    pub(crate) fn plm_dir(&self) -> PathBuf {
        self.root.join(".plm")
    }

    /// ターゲットレジストリのパス: `{plm_dir}/targets.json`
    pub(crate) fn targets_json(&self) -> PathBuf {
        self.plm_dir().join("targets.json")
    }

    /// マーケットプレイス設定のパス: `{plm_dir}/marketplaces.json`
    pub(crate) fn marketplaces_json(&self) -> PathBuf {
        self.plm_dir().join("marketplaces.json")
    }

    /// インポートレジストリのパス: `{plm_dir}/imports.json`
    pub(crate) fn imports_json(&self) -> PathBuf {
        self.plm_dir().join("imports.json")
    }

    /// プラグインキャッシュディレクトリ: `{plm_dir}/cache/plugins`
    pub(crate) fn plugins_cache_dir(&self) -> PathBuf {
        self.plm_dir().join("cache").join("plugins")
    }

    /// マーケットプレイスキャッシュディレクトリ: `{plm_dir}/cache/marketplaces`
    pub(crate) fn marketplaces_cache_dir(&self) -> PathBuf {
        self.plm_dir().join("cache").join("marketplaces")
    }
}

#[cfg(test)]
#[path = "env_test.rs"]
mod tests;

//! プラグインソースパス値オブジェクト
//!
//! marketplace.json の source フィールドで指定されるローカルプラグインへの相対パス。

use crate::error::PlmError;
use crate::marketplace::windows_path::{starts_with_drive_letter, starts_with_unc};
use std::path::{Component, Path};
use std::str::FromStr;

/// marketplace.json の source フィールドで指定される
/// ローカルプラグインへの相対パス（リポジトリルートからの相対）
///
/// # 不変条件
///
/// - 正規化済み（"./", ".", 末尾スラッシュなし）
/// - リポジトリ外への参照なし（".." 禁止）
/// - 相対パスのみ（絶対パス、ドライブレター、UNC禁止）
///
/// # Examples
///
/// ```ignore
/// let path: PluginSourcePath = "./plugins/foo".parse()?;
/// assert_eq!(path.as_str(), "plugins/foo");
///
/// // リポジトリルートを指す場合は空文字列
/// let root: PluginSourcePath = "./".parse()?;
/// assert_eq!(root.as_str(), "");
///
/// assert!("../bad".parse::<PluginSourcePath>().is_err());
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PluginSourcePath(String);

impl PluginSourcePath {
    /// 正規化されたパス文字列を取得
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl FromStr for PluginSourcePath {
    type Err = PlmError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // バックスラッシュをスラッシュに正規化（Windows 由来のパス対応）
        let path = s.replace('\\', "/");

        // Windows ドライブレターを拒否（例: "C:foo", "a:plugins"）
        // クロスプラットフォームの一貫性のため、Unix でも拒否する
        if starts_with_drive_letter(&path) {
            return Err(PlmError::InvalidSource(
                "source_path must be a relative path without drive letters".into(),
            ));
        }

        // Windows UNC パスを拒否（例: "\\server\share" → 正規化後 "//server/share"）
        if starts_with_unc(&path) {
            return Err(PlmError::InvalidSource(
                "source_path must be a relative path without UNC paths".into(),
            ));
        }

        let path_obj = Path::new(&path);

        // 絶対パスを拒否
        if path_obj.is_absolute() {
            return Err(PlmError::InvalidSource(
                "source_path must be relative".into(),
            ));
        }

        // コンポーネントを走査し、Normal のみを収集
        let mut parts: Vec<&str> = Vec::new();
        for component in path_obj.components() {
            match component {
                Component::Normal(s) => match s.to_str() {
                    Some(s) => parts.push(s),
                    None => {
                        return Err(PlmError::InvalidSource(
                            "source_path contains non-UTF-8 characters".into(),
                        ));
                    }
                },
                Component::ParentDir => {
                    return Err(PlmError::InvalidSource("source_path contains '..'".into()));
                }
                Component::Prefix(_) | Component::RootDir => {
                    return Err(PlmError::InvalidSource(
                        "source_path must be a relative path without drive letters or UNC paths"
                            .into(),
                    ));
                }
                Component::CurDir => {
                    // "." は無視して続行
                }
            }
        }

        // "/" 区切りで再構成（OS 非依存）
        // 空の場合はリポジトリルートを表す
        Ok(Self(parts.join("/")))
    }
}

impl AsRef<str> for PluginSourcePath {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl From<PluginSourcePath> for String {
    fn from(path: PluginSourcePath) -> Self {
        path.0
    }
}

#[cfg(test)]
#[path = "plugin_source_path_test.rs"]
mod tests;

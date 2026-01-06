//! プラグインソースパス値オブジェクト
//!
//! marketplace.json の source フィールドで指定されるローカルプラグインへの相対パス。

use crate::error::PlmError;
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
/// assert!("../bad".parse::<PluginSourcePath>().is_err());
/// assert!(".".parse::<PluginSourcePath>().is_err());
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

        // Windows パス形式を拒否（ドライブレター: "C:xxx" など）
        // プラットフォーム間の一貫性のため、単一アルファベット+コロンのパターンは全て拒否
        // 注: Unix では "a:plugins" は合法な相対パスだが、以下の理由で一律拒否:
        //   1. marketplace.json は複数プラットフォームで共有される可能性がある
        //   2. Windows で意図せずドライブレターとして解釈されるリスクを排除
        //   3. コロン含みのディレクトリ名は一般的ではなく、拒否による影響は限定的
        if path.len() >= 2
            && path
                .chars()
                .next()
                .map(|c| c.is_ascii_alphabetic())
                .unwrap_or(false)
            && path.chars().nth(1) == Some(':')
        {
            return Err(PlmError::InvalidSource(
                "subdir must be a relative path without drive letters".into(),
            ));
        }

        // UNC パスを拒否（正規化後は "//" になる）
        if path.starts_with("//") {
            return Err(PlmError::InvalidSource(
                "subdir must be a relative path without UNC paths".into(),
            ));
        }

        let path_obj = Path::new(&path);

        // 絶対パスを拒否
        if path_obj.is_absolute() {
            return Err(PlmError::InvalidSource("subdir must be relative".into()));
        }

        // コンポーネントを走査し、Normal のみを収集
        let mut parts: Vec<&str> = Vec::new();
        for component in path_obj.components() {
            match component {
                Component::Normal(s) => match s.to_str() {
                    Some(s) => parts.push(s),
                    None => {
                        return Err(PlmError::InvalidSource(
                            "subdir contains non-UTF-8 characters".into(),
                        ));
                    }
                },
                Component::ParentDir => {
                    return Err(PlmError::InvalidSource("subdir contains '..'".into()));
                }
                Component::Prefix(_) | Component::RootDir => {
                    return Err(PlmError::InvalidSource(
                        "subdir must be a relative path without drive letters or UNC paths".into(),
                    ));
                }
                Component::CurDir => {
                    // "." は無視して続行
                }
            }
        }

        // 空パスを拒否（仕様通りの明確なメッセージ）
        if parts.is_empty() {
            return Err(PlmError::InvalidSource(
                "Local plugin must specify a subdirectory (e.g., './plugins/my-plugin'). \
                 Use External for root-level plugins."
                    .into(),
            ));
        }

        // "/" 区切りで再構成（OS 非依存）
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
mod tests {
    use super::*;

    // テストケース: "./plugins/foo" → "plugins/foo" に正規化
    #[test]
    fn test_normalize_leading_dot_slash() {
        let path: PluginSourcePath = "./plugins/foo".parse().unwrap();
        assert_eq!(path.as_str(), "plugins/foo");
    }

    // テストケース: "plugins/./foo" → "plugins/foo" に正規化（内部 `.` 除去）
    #[test]
    fn test_normalize_internal_dot() {
        let path: PluginSourcePath = "plugins/./foo".parse().unwrap();
        assert_eq!(path.as_str(), "plugins/foo");
    }

    // テストケース: "plugins/foo/" → "plugins/foo" に正規化（末尾スラッシュ除去）
    #[test]
    fn test_normalize_trailing_slash() {
        let path: PluginSourcePath = "plugins/foo/".parse().unwrap();
        assert_eq!(path.as_str(), "plugins/foo");
    }

    // テストケース: "plugins\\foo" → "plugins/foo" に正規化（バックスラッシュ変換）
    #[test]
    fn test_normalize_backslash() {
        let path: PluginSourcePath = "plugins\\foo".parse().unwrap();
        assert_eq!(path.as_str(), "plugins/foo");
    }

    // テストケース: "" または "." → InvalidSource エラー（明確なメッセージ）
    #[test]
    fn test_reject_empty_path() {
        let err = "".parse::<PluginSourcePath>().unwrap_err();
        match err {
            PlmError::InvalidSource(msg) => {
                assert!(msg.contains("subdirectory"));
            }
            _ => panic!("Expected InvalidSource error"),
        }
    }

    #[test]
    fn test_reject_dot_only() {
        let err = ".".parse::<PluginSourcePath>().unwrap_err();
        match err {
            PlmError::InvalidSource(msg) => {
                assert!(msg.contains("subdirectory"));
            }
            _ => panic!("Expected InvalidSource error"),
        }
    }

    // テストケース: "../plugins/foo" → InvalidSource エラー（セキュリティ）
    #[test]
    fn test_reject_parent_dir() {
        let err = "../plugins/foo".parse::<PluginSourcePath>().unwrap_err();
        match err {
            PlmError::InvalidSource(msg) => {
                assert!(msg.contains(".."));
            }
            _ => panic!("Expected InvalidSource error"),
        }
    }

    // テストケース: "/plugins/foo" → InvalidSource エラー（絶対パス拒否）
    #[test]
    fn test_reject_absolute_path() {
        let err = "/plugins/foo".parse::<PluginSourcePath>().unwrap_err();
        match err {
            PlmError::InvalidSource(msg) => {
                assert!(msg.contains("relative"));
            }
            _ => panic!("Expected InvalidSource error"),
        }
    }

    // テストケース: "C:/plugins/foo" → InvalidSource エラー（Windows ドライブ拒否）
    #[test]
    fn test_reject_windows_drive_with_separator() {
        let err = "C:/plugins/foo".parse::<PluginSourcePath>().unwrap_err();
        match err {
            PlmError::InvalidSource(msg) => {
                assert!(msg.contains("drive") || msg.contains("relative"));
            }
            _ => panic!("Expected InvalidSource error"),
        }
    }

    // テストケース: "a:plugins/foo" → InvalidSource エラー（Windows Prefix 拒否）
    #[test]
    fn test_reject_drive_letter_without_separator() {
        let err = "a:plugins/foo".parse::<PluginSourcePath>().unwrap_err();
        match err {
            PlmError::InvalidSource(msg) => {
                assert!(msg.contains("drive"));
            }
            _ => panic!("Expected InvalidSource error"),
        }
    }

    // テストケース: "\\\\server\\share" → InvalidSource エラー（UNC パス拒否）
    #[test]
    fn test_reject_unc_path() {
        let err = "\\\\server\\share".parse::<PluginSourcePath>().unwrap_err();
        match err {
            PlmError::InvalidSource(msg) => {
                assert!(msg.contains("UNC") || msg.contains("relative"));
            }
            _ => panic!("Expected InvalidSource error"),
        }
    }

    // 追加テスト: 複合ケース
    #[test]
    fn test_normalize_complex_path() {
        let path: PluginSourcePath = "./plugins/./foo/bar/".parse().unwrap();
        assert_eq!(path.as_str(), "plugins/foo/bar");
    }

    #[test]
    fn test_normalize_single_component() {
        let path: PluginSourcePath = "plugins".parse().unwrap();
        assert_eq!(path.as_str(), "plugins");
    }

    #[test]
    fn test_normalize_deep_path() {
        let path: PluginSourcePath = "a/b/c/d/e".parse().unwrap();
        assert_eq!(path.as_str(), "a/b/c/d/e");
    }
}

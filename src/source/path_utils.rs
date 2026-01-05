//! パス正規化ユーティリティ
//!
//! マーケットプレイスプラグインのサブディレクトリパスを正規化・検証する。

use crate::error::{PlmError, Result};
use std::path::{Component, Path};

/// サブディレクトリパスを正規化
///
/// - "./" や "." を除去し、Normal コンポーネントのみで再構成
/// - ".." や絶対パスを拒否（セキュリティ）
/// - バックスラッシュをスラッシュに変換
///
/// # Examples
///
/// ```ignore
/// assert_eq!(normalize_subdir_path("./plugins/foo")?, "plugins/foo");
/// assert_eq!(normalize_subdir_path("plugins\\foo")?, "plugins/foo");
/// assert!(normalize_subdir_path("../bad").is_err());
/// assert!(normalize_subdir_path(".").is_err());
/// ```
pub fn normalize_subdir_path(path: &str) -> Result<String> {
    // バックスラッシュをスラッシュに正規化（Windows 由来のパス対応）
    let path = path.replace('\\', "/");

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
    Ok(parts.join("/"))
}

#[cfg(test)]
mod tests {
    use super::*;

    // テストケース 4: "./plugins/foo" → "plugins/foo" に正規化
    #[test]
    fn test_normalize_leading_dot_slash() {
        assert_eq!(
            normalize_subdir_path("./plugins/foo").unwrap(),
            "plugins/foo"
        );
    }

    // テストケース 5: "plugins/./foo" → "plugins/foo" に正規化（内部 `.` 除去）
    #[test]
    fn test_normalize_internal_dot() {
        assert_eq!(
            normalize_subdir_path("plugins/./foo").unwrap(),
            "plugins/foo"
        );
    }

    // テストケース 6: "plugins/foo/" → "plugins/foo" に正規化（末尾スラッシュ除去）
    #[test]
    fn test_normalize_trailing_slash() {
        assert_eq!(
            normalize_subdir_path("plugins/foo/").unwrap(),
            "plugins/foo"
        );
    }

    // テストケース 7: "plugins\\foo" → "plugins/foo" に正規化（バックスラッシュ変換）
    #[test]
    fn test_normalize_backslash() {
        assert_eq!(
            normalize_subdir_path("plugins\\foo").unwrap(),
            "plugins/foo"
        );
    }

    // テストケース 8: "" または "." → InvalidSource エラー（明確なメッセージ）
    #[test]
    fn test_reject_empty_path() {
        let err = normalize_subdir_path("").unwrap_err();
        match err {
            PlmError::InvalidSource(msg) => {
                assert!(msg.contains("subdirectory"));
            }
            _ => panic!("Expected InvalidSource error"),
        }
    }

    #[test]
    fn test_reject_dot_only() {
        let err = normalize_subdir_path(".").unwrap_err();
        match err {
            PlmError::InvalidSource(msg) => {
                assert!(msg.contains("subdirectory"));
            }
            _ => panic!("Expected InvalidSource error"),
        }
    }

    // テストケース 9: "../plugins/foo" → InvalidSource エラー（セキュリティ）
    #[test]
    fn test_reject_parent_dir() {
        let err = normalize_subdir_path("../plugins/foo").unwrap_err();
        match err {
            PlmError::InvalidSource(msg) => {
                assert!(msg.contains(".."));
            }
            _ => panic!("Expected InvalidSource error"),
        }
    }

    // テストケース 10: "/plugins/foo" → InvalidSource エラー（絶対パス拒否）
    #[test]
    fn test_reject_absolute_path() {
        let err = normalize_subdir_path("/plugins/foo").unwrap_err();
        match err {
            PlmError::InvalidSource(msg) => {
                assert!(msg.contains("relative"));
            }
            _ => panic!("Expected InvalidSource error"),
        }
    }

    // テストケース 11: "C:/plugins/foo" → InvalidSource エラー（Windows ドライブ拒否）
    #[test]
    fn test_reject_windows_drive_with_separator() {
        let err = normalize_subdir_path("C:/plugins/foo").unwrap_err();
        match err {
            PlmError::InvalidSource(msg) => {
                assert!(msg.contains("drive") || msg.contains("relative"));
            }
            _ => panic!("Expected InvalidSource error"),
        }
    }

    // テストケース 12: "a:plugins/foo" → InvalidSource エラー（Windows Prefix 拒否）
    #[test]
    fn test_reject_drive_letter_without_separator() {
        let err = normalize_subdir_path("a:plugins/foo").unwrap_err();
        match err {
            PlmError::InvalidSource(msg) => {
                assert!(msg.contains("drive"));
            }
            _ => panic!("Expected InvalidSource error"),
        }
    }

    // テストケース 13: "\\\\server\\share" → InvalidSource エラー（UNC パス拒否）
    #[test]
    fn test_reject_unc_path() {
        let err = normalize_subdir_path("\\\\server\\share").unwrap_err();
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
        assert_eq!(
            normalize_subdir_path("./plugins/./foo/bar/").unwrap(),
            "plugins/foo/bar"
        );
    }

    #[test]
    fn test_normalize_single_component() {
        assert_eq!(normalize_subdir_path("plugins").unwrap(), "plugins");
    }

    #[test]
    fn test_normalize_deep_path() {
        assert_eq!(
            normalize_subdir_path("a/b/c/d/e").unwrap(),
            "a/b/c/d/e"
        );
    }
}

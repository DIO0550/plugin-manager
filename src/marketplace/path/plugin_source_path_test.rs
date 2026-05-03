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

// テストケース: "" や "." や "./" → リポジトリルート（空文字列）
#[test]
fn test_empty_path_is_root() {
    let path: PluginSourcePath = "".parse().unwrap();
    assert_eq!(path.as_str(), "");
}

#[test]
fn test_dot_only_is_root() {
    let path: PluginSourcePath = ".".parse().unwrap();
    assert_eq!(path.as_str(), "");
}

#[test]
fn test_dot_slash_is_root() {
    let path: PluginSourcePath = "./".parse().unwrap();
    assert_eq!(path.as_str(), "");
}

// テストケース: "../plugins/foo" → InvalidSource エラー（セキュリティ）
#[test]
fn test_reject_parent_dir() {
    let err = "../plugins/foo".parse::<PluginSourcePath>().unwrap_err();
    match err {
        PlmError::InvalidSource(msg) => {
            assert!(msg.contains("source_path contains '..'"));
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

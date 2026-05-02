//! scanner モジュールのテスト

use super::scan_components;
use std::fs;
use tempfile::TempDir;

/// フラット 1 階層構造を正常にスキャンできる
#[test]
fn test_scan_components_flat_dir() {
    let temp = TempDir::new().unwrap();
    let base = temp.path();

    // フラット構造: `<flattened_name>` を直下に配置
    let component_dir = base.join("plugin_my-skill");
    fs::create_dir_all(&component_dir).unwrap();

    let results = scan_components(base).unwrap();

    assert_eq!(results.len(), 1);
    let component = &results[0];
    assert_eq!(component.origin.marketplace, "_");
    assert_eq!(component.origin.plugin, "_");
    assert_eq!(component.name, "plugin_my-skill");
    assert!(component.is_dir);
    assert_eq!(component.path, component_dir);
}

/// 存在しないディレクトリで空Vecを返す
#[test]
fn test_scan_components_nonexistent_dir() {
    let temp = TempDir::new().unwrap();
    let nonexistent = temp.path().join("does-not-exist");

    let results = scan_components(&nonexistent).unwrap();

    assert!(results.is_empty());
}

/// 直下のファイルもエントリとして拾われる（ディレクトリかどうかは is_dir で区別）
#[test]
fn test_scan_components_includes_files_at_top_level() {
    let temp = TempDir::new().unwrap();
    let base = temp.path();

    fs::write(base.join("plugin_foo.agent.md"), "content").unwrap();
    fs::create_dir_all(base.join("plugin_bar")).unwrap();

    let results = scan_components(base).unwrap();

    assert_eq!(results.len(), 2);
    let dirs: Vec<_> = results.iter().filter(|c| c.is_dir).collect();
    let files: Vec<_> = results.iter().filter(|c| !c.is_dir).collect();
    assert_eq!(dirs.len(), 1);
    assert_eq!(files.len(), 1);
}

/// 複数のフラットエントリをスキャンできる
#[test]
fn test_scan_components_multiple() {
    let temp = TempDir::new().unwrap();
    let base = temp.path();

    fs::create_dir_all(base.join("plg-a_skill-1")).unwrap();
    fs::create_dir_all(base.join("plg-a_skill-2")).unwrap();
    fs::create_dir_all(base.join("plg-b_skill-3")).unwrap();
    fs::write(base.join("plg-c_test.agent.md"), "content").unwrap();

    let results = scan_components(base).unwrap();

    assert_eq!(results.len(), 4);
}

/// 空のディレクトリは空Vecを返す
#[test]
fn test_scan_components_empty_dir() {
    let temp = TempDir::new().unwrap();
    let base = temp.path();

    let results = scan_components(base).unwrap();

    assert!(results.is_empty());
}

/// ファイルを渡した場合はエラー
#[test]
fn test_scan_components_file_returns_error() {
    let temp = TempDir::new().unwrap();
    let file_path = temp.path().join("file.txt");
    fs::write(&file_path, "content").unwrap();

    let result = scan_components(&file_path);

    assert!(result.is_err());
}

//! scanner モジュールのテスト

use super::{scan_components, ScannedComponent};
use std::fs;
use tempfile::TempDir;

/// 3層構造を正常にスキャンできる
#[test]
fn test_scan_components_three_level_structure() {
    let temp = TempDir::new().unwrap();
    let base = temp.path();

    // 3層構造を作成: marketplace/plugin/component
    let component_dir = base.join("official").join("my-plugin").join("my-skill");
    fs::create_dir_all(&component_dir).unwrap();

    let results = scan_components(base).unwrap();

    assert_eq!(results.len(), 1);
    let component = &results[0];
    assert_eq!(component.origin.marketplace, "official");
    assert_eq!(component.origin.plugin, "my-plugin");
    assert_eq!(component.name, "my-skill");
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

/// 中間層のファイルはスキップされる
#[test]
fn test_scan_components_skips_files_at_intermediate_levels() {
    let temp = TempDir::new().unwrap();
    let base = temp.path();

    // 正常な3層構造
    let component_dir = base.join("mp").join("plugin").join("skill");
    fs::create_dir_all(&component_dir).unwrap();

    // 中間層にファイルを置く (これはスキップされるべき)
    fs::write(base.join("mp").join("stray-file.txt"), "ignored").unwrap();
    fs::write(base.join("top-level-file.txt"), "ignored").unwrap();

    let results = scan_components(base).unwrap();

    // 3層構造のコンポーネントのみが取得される
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].name, "skill");
}

/// 複数のコンポーネントをスキャンできる
#[test]
fn test_scan_components_multiple() {
    let temp = TempDir::new().unwrap();
    let base = temp.path();

    // 複数のコンポーネント
    fs::create_dir_all(base.join("mp1").join("plugin-a").join("skill-1")).unwrap();
    fs::create_dir_all(base.join("mp1").join("plugin-a").join("skill-2")).unwrap();
    fs::create_dir_all(base.join("mp2").join("plugin-b").join("agent")).unwrap();

    // ファイルコンポーネントも含む
    fs::create_dir_all(base.join("mp1").join("plugin-c")).unwrap();
    fs::write(
        base.join("mp1").join("plugin-c").join("test.agent.md"),
        "content",
    )
    .unwrap();

    let results = scan_components(base).unwrap();

    assert_eq!(results.len(), 4);

    // ディレクトリとファイルが両方含まれる
    let dirs: Vec<_> = results.iter().filter(|c| c.is_dir).collect();
    let files: Vec<_> = results.iter().filter(|c| !c.is_dir).collect();
    assert_eq!(dirs.len(), 3);
    assert_eq!(files.len(), 1);
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

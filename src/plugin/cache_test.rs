use super::*;
use std::fs;
use tempfile::TempDir;

/// テスト用のzipアーカイブを作成するヘルパー
fn create_test_archive(entries: &[(&str, &str)]) -> Vec<u8> {
    use std::io::Write;
    let mut buf = Vec::new();
    {
        let mut zip = zip::ZipWriter::new(std::io::Cursor::new(&mut buf));
        let options = zip::write::SimpleFileOptions::default();

        for (path, content) in entries {
            zip.start_file(*path, options).unwrap();
            zip.write_all(content.as_bytes()).unwrap();
        }
        zip.finish().unwrap();
    }
    buf
}

#[test]
fn test_store_from_archive_with_source_path_extracts_to_root() {
    // テストケース14: source_path 指定時、そのパス配下の内容がキャッシュ直下に展開される
    let temp_dir = TempDir::new().unwrap();
    let cache = PluginCache::with_cache_dir(temp_dir.path().to_path_buf()).unwrap();

    // GitHub 形式のアーカイブ（prefix + source_path）
    let archive = create_test_archive(&[
        (
            "repo-main/plugins/my-plugin/plugin.json",
            r#"{"name":"test","version":"1.0.0"}"#,
        ),
        ("repo-main/plugins/my-plugin/skills/test.md", "# Test Skill"),
        ("repo-main/other/file.txt", "should not be extracted"),
    ]);

    let result = cache.store_from_archive(
        Some("test-marketplace"),
        "my-plugin",
        &archive,
        Some("plugins/my-plugin"),
    );

    assert!(result.is_ok());
    let plugin_dir = result.unwrap();

    // サブディレクトリの内容がキャッシュ直下に展開されている
    assert!(plugin_dir.join("plugin.json").exists());
    assert!(plugin_dir.join("skills/test.md").exists());
    // 他のファイルは展開されない
    assert!(!plugin_dir.join("other").exists());
}

#[test]
fn test_store_from_archive_source_path_boundary_match() {
    // テストケース15: source_path = "plugins/foo" で plugins/foo-bar が誤抽出されない
    let temp_dir = TempDir::new().unwrap();
    let cache = PluginCache::with_cache_dir(temp_dir.path().to_path_buf()).unwrap();

    let archive = create_test_archive(&[
        ("repo-main/plugins/foo/file.txt", "correct"),
        ("repo-main/plugins/foo-bar/file.txt", "should not match"),
        ("repo-main/plugins/foobar/file.txt", "should not match either"),
    ]);

    let result = cache.store_from_archive(
        Some("test-marketplace"),
        "foo-plugin",
        &archive,
        Some("plugins/foo"),
    );

    assert!(result.is_ok());
    let plugin_dir = result.unwrap();

    // plugins/foo の内容のみ展開
    assert!(plugin_dir.join("file.txt").exists());
    let content = fs::read_to_string(plugin_dir.join("file.txt")).unwrap();
    assert_eq!(content, "correct");

    // plugins/foo-bar や plugins/foobar は展開されない
    // （ディレクトリ自体が存在しないことを確認）
    let entries: Vec<_> = fs::read_dir(&plugin_dir).unwrap().collect();
    assert_eq!(entries.len(), 2); // file.txt + .plm-meta.json
}

#[test]
fn test_store_from_archive_source_path_not_found() {
    // テストケース16: source_path がアーカイブ内に存在しない場合 → InvalidSource エラー
    let temp_dir = TempDir::new().unwrap();
    let cache = PluginCache::with_cache_dir(temp_dir.path().to_path_buf()).unwrap();

    let archive = create_test_archive(&[("repo-main/other/file.txt", "content")]);

    let result = cache.store_from_archive(
        Some("test-marketplace"),
        "my-plugin",
        &archive,
        Some("plugins/nonexistent"),
    );

    assert!(result.is_err());
    match result.unwrap_err() {
        PlmError::InvalidSource(msg) => {
            assert!(msg.contains("source_path not found"));
        }
        e => panic!("Expected InvalidSource error, got: {:?}", e),
    }
}

#[test]
fn test_store_from_archive_source_path_validation_dotdot() {
    // テストケース21: source_path に .. が含まれる場合 → エラー
    let temp_dir = TempDir::new().unwrap();
    let cache = PluginCache::with_cache_dir(temp_dir.path().to_path_buf()).unwrap();

    let archive = create_test_archive(&[("repo-main/plugins/foo/file.txt", "content")]);

    let result = cache.store_from_archive(
        Some("test-marketplace"),
        "my-plugin",
        &archive,
        Some("plugins/../foo"),
    );

    assert!(result.is_err());
    match result.unwrap_err() {
        PlmError::InvalidSource(msg) => {
            assert!(msg.contains("not normalized"));
        }
        e => panic!("Expected InvalidSource error, got: {:?}", e),
    }
}

#[test]
fn test_store_from_archive_source_path_validation_backslash() {
    // テストケース21: source_path に \ が含まれる場合 → エラー
    let temp_dir = TempDir::new().unwrap();
    let cache = PluginCache::with_cache_dir(temp_dir.path().to_path_buf()).unwrap();

    let archive = create_test_archive(&[("repo-main/plugins/foo/file.txt", "content")]);

    let result = cache.store_from_archive(
        Some("test-marketplace"),
        "my-plugin",
        &archive,
        Some("plugins\\foo"),
    );

    assert!(result.is_err());
    match result.unwrap_err() {
        PlmError::InvalidSource(msg) => {
            assert!(msg.contains("not normalized"));
        }
        e => panic!("Expected InvalidSource error, got: {:?}", e),
    }
}

#[test]
fn test_store_from_archive_source_path_validation_dot_slash() {
    // テストケース21: source_path に ./ が含まれる場合 → エラー
    let temp_dir = TempDir::new().unwrap();
    let cache = PluginCache::with_cache_dir(temp_dir.path().to_path_buf()).unwrap();

    let archive = create_test_archive(&[("repo-main/plugins/foo/file.txt", "content")]);

    let result = cache.store_from_archive(
        Some("test-marketplace"),
        "my-plugin",
        &archive,
        Some("./plugins/foo"),
    );

    assert!(result.is_err());
    match result.unwrap_err() {
        PlmError::InvalidSource(msg) => {
            assert!(msg.contains("not normalized"));
        }
        e => panic!("Expected InvalidSource error, got: {:?}", e),
    }
}

#[test]
fn test_store_from_archive_without_source_path_extracts_all() {
    // source_path = None の場合は従来通り全ファイルを展開
    let temp_dir = TempDir::new().unwrap();
    let cache = PluginCache::with_cache_dir(temp_dir.path().to_path_buf()).unwrap();

    let archive = create_test_archive(&[
        ("repo-main/plugin.json", r#"{"name":"test","version":"1.0.0"}"#),
        ("repo-main/skills/test.md", "# Test"),
        ("repo-main/other/file.txt", "content"),
    ]);

    let result = cache.store_from_archive(None, "test-plugin", &archive, None);

    assert!(result.is_ok());
    let plugin_dir = result.unwrap();

    // 全ファイルが展開される
    assert!(plugin_dir.join("plugin.json").exists());
    assert!(plugin_dir.join("skills/test.md").exists());
    assert!(plugin_dir.join("other/file.txt").exists());
}

#[test]
fn test_store_from_archive_handles_backslash_entries() {
    // テストケース20: zip内の \ 区切りエントリを / に正規化後一致
    let temp_dir = TempDir::new().unwrap();
    let cache = PluginCache::with_cache_dir(temp_dir.path().to_path_buf()).unwrap();

    // バックスラッシュを含むエントリ名（Windows由来のzip）
    // プレフィックスはスラッシュで書く（プレフィックス抽出は / でsplitするため）
    let archive = create_test_archive(&[(
        "repo-main/plugins\\foo\\file.txt",
        "content with backslash",
    )]);

    let result = cache.store_from_archive(
        Some("test-marketplace"),
        "foo-plugin",
        &archive,
        Some("plugins/foo"),
    );

    assert!(result.is_ok(), "Expected Ok, got: {:?}", result);
    let plugin_dir = result.unwrap();
    assert!(plugin_dir.join("file.txt").exists());
}

#[test]
fn test_store_from_archive_writes_installed_at_to_meta() {
    // store_from_archive 後に .plm-meta.json に installedAt が書き込まれることを確認
    let temp_dir = TempDir::new().unwrap();
    let cache = PluginCache::with_cache_dir(temp_dir.path().to_path_buf()).unwrap();

    let archive = create_test_archive(&[(
        "repo-main/plugin.json",
        r#"{"name":"test","version":"1.0.0"}"#,
    )]);

    let result = cache.store_from_archive(None, "test-plugin", &archive, None);
    assert!(result.is_ok());
    let plugin_dir = result.unwrap();

    // .plm-meta.json を読み込んで installedAt を確認
    let meta_path = plugin_dir.join(".plm-meta.json");
    assert!(meta_path.exists(), ".plm-meta.json should exist");

    let content = fs::read_to_string(&meta_path).unwrap();
    let json: serde_json::Value = serde_json::from_str(&content).unwrap();

    assert!(json.get("installedAt").is_some());
    let installed_at = json.get("installedAt").unwrap().as_str().unwrap();
    // RFC3339 形式であることを確認（YYYY-MM-DDTHH:MM:SSZ）
    assert!(installed_at.contains("T"));
    assert!(installed_at.ends_with("Z"));
}

#[test]
fn test_store_from_archive_does_not_modify_plugin_json() {
    // plugin.json が改変されないことを確認（上流成果物の保持）
    let temp_dir = TempDir::new().unwrap();
    let cache = PluginCache::with_cache_dir(temp_dir.path().to_path_buf()).unwrap();

    let original_content = r#"{"name":"test","version":"1.0.0","customField":"preserved"}"#;
    let archive = create_test_archive(&[("repo-main/plugin.json", original_content)]);

    let result = cache.store_from_archive(None, "test-plugin", &archive, None);
    assert!(result.is_ok());
    let plugin_dir = result.unwrap();

    // plugin.json が改変されていないことを確認
    let content = fs::read_to_string(plugin_dir.join("plugin.json")).unwrap();
    let json: serde_json::Value = serde_json::from_str(&content).unwrap();

    // customField が保持されている
    assert_eq!(
        json.get("customField").unwrap().as_str().unwrap(),
        "preserved"
    );
    // installedAt は追加されていない（.plm-meta.json に記録される）
    assert!(
        json.get("installedAt").is_none(),
        "plugin.json should not have installedAt"
    );

    // .plm-meta.json に installedAt がある
    let meta_content = fs::read_to_string(plugin_dir.join(".plm-meta.json")).unwrap();
    let meta_json: serde_json::Value = serde_json::from_str(&meta_content).unwrap();
    assert!(meta_json.get("installedAt").is_some());
}

// =============================================================================
// Backup/Restore tests
// =============================================================================

#[test]
fn test_backup_creates_copy() {
    let temp_dir = TempDir::new().unwrap();
    let cache = PluginCache::with_cache_dir(temp_dir.path().to_path_buf()).unwrap();

    // まずプラグインをインストール
    let archive = create_test_archive(&[
        ("repo-main/plugin.json", r#"{"name":"test","version":"1.0.0"}"#),
        ("repo-main/skills/test.md", "# Original Skill"),
    ]);
    cache
        .store_from_archive(Some("github"), "test-plugin", &archive, None)
        .unwrap();

    // バックアップ作成
    let backup_path = cache.backup(Some("github"), "test-plugin").unwrap();

    // バックアップが存在する
    assert!(backup_path.exists());
    assert!(backup_path.join("plugin.json").exists());
    assert!(backup_path.join("skills/test.md").exists());

    // 元のプラグインも存在する
    let original = cache.plugin_path(Some("github"), "test-plugin");
    assert!(original.exists());
}

#[test]
fn test_restore_recovers_from_backup() {
    let temp_dir = TempDir::new().unwrap();
    let cache = PluginCache::with_cache_dir(temp_dir.path().to_path_buf()).unwrap();

    // プラグインをインストール
    let archive = create_test_archive(&[
        ("repo-main/plugin.json", r#"{"name":"test","version":"1.0.0"}"#),
        ("repo-main/data.txt", "original content"),
    ]);
    cache
        .store_from_archive(Some("github"), "test-plugin", &archive, None)
        .unwrap();

    // バックアップ作成
    cache.backup(Some("github"), "test-plugin").unwrap();

    // 元のプラグインを変更（削除してから違う内容で作り直し）
    let plugin_path = cache.plugin_path(Some("github"), "test-plugin");
    fs::write(plugin_path.join("data.txt"), "modified content").unwrap();

    // 変更されていることを確認
    let content = fs::read_to_string(plugin_path.join("data.txt")).unwrap();
    assert_eq!(content, "modified content");

    // リストア
    cache.restore(Some("github"), "test-plugin").unwrap();

    // 元の内容に戻っている
    let restored_content = fs::read_to_string(plugin_path.join("data.txt")).unwrap();
    assert_eq!(restored_content, "original content");
}

#[test]
fn test_remove_backup_deletes_backup() {
    let temp_dir = TempDir::new().unwrap();
    let cache = PluginCache::with_cache_dir(temp_dir.path().to_path_buf()).unwrap();

    // プラグインをインストール
    let archive = create_test_archive(&[(
        "repo-main/plugin.json",
        r#"{"name":"test","version":"1.0.0"}"#,
    )]);
    cache
        .store_from_archive(Some("github"), "test-plugin", &archive, None)
        .unwrap();

    // バックアップ作成
    let backup_path = cache.backup(Some("github"), "test-plugin").unwrap();
    assert!(backup_path.exists());

    // バックアップ削除
    cache.remove_backup(Some("github"), "test-plugin").unwrap();

    // バックアップが存在しない
    assert!(!backup_path.exists());
}

#[test]
fn test_restore_without_backup_returns_error() {
    let temp_dir = TempDir::new().unwrap();
    let cache = PluginCache::with_cache_dir(temp_dir.path().to_path_buf()).unwrap();

    // プラグインをインストール（バックアップなし）
    let archive = create_test_archive(&[(
        "repo-main/plugin.json",
        r#"{"name":"test","version":"1.0.0"}"#,
    )]);
    cache
        .store_from_archive(Some("github"), "test-plugin", &archive, None)
        .unwrap();

    // バックアップなしでリストア → エラー
    let result = cache.restore(Some("github"), "test-plugin");
    assert!(result.is_err());
}

#[test]
fn test_backup_nonexistent_plugin_returns_error() {
    let temp_dir = TempDir::new().unwrap();
    let cache = PluginCache::with_cache_dir(temp_dir.path().to_path_buf()).unwrap();

    // 存在しないプラグインのバックアップ → エラー
    let result = cache.backup(Some("github"), "nonexistent-plugin");
    assert!(result.is_err());
}

// =============================================================================
// Atomic update tests
// =============================================================================

#[test]
fn test_atomic_update_success() {
    let temp_dir = TempDir::new().unwrap();
    let cache = PluginCache::with_cache_dir(temp_dir.path().to_path_buf()).unwrap();

    // 初期プラグインをインストール
    let archive_v1 = create_test_archive(&[
        ("repo-main/plugin.json", r#"{"name":"test","version":"1.0.0"}"#),
        ("repo-main/data.txt", "version 1"),
    ]);
    cache
        .store_from_archive(Some("github"), "test-plugin", &archive_v1, None)
        .unwrap();

    // 新しいバージョンのアーカイブ
    let archive_v2 = create_test_archive(&[
        ("repo-main/plugin.json", r#"{"name":"test","version":"2.0.0"}"#),
        ("repo-main/data.txt", "version 2"),
    ]);

    // アトミック更新
    let result = cache.atomic_update(Some("github"), "test-plugin", &archive_v2);

    assert!(result.is_ok());
    let plugin_path = result.unwrap();

    // 新しいバージョンが配置されている
    let content = fs::read_to_string(plugin_path.join("data.txt")).unwrap();
    assert_eq!(content, "version 2");

    let manifest = fs::read_to_string(plugin_path.join("plugin.json")).unwrap();
    assert!(manifest.contains("2.0.0"));
}

#[test]
fn test_atomic_update_without_plugin_json_returns_error() {
    let temp_dir = TempDir::new().unwrap();
    let cache = PluginCache::with_cache_dir(temp_dir.path().to_path_buf()).unwrap();

    // 初期プラグインをインストール
    let archive_v1 = create_test_archive(&[(
        "repo-main/plugin.json",
        r#"{"name":"test","version":"1.0.0"}"#,
    )]);
    cache
        .store_from_archive(Some("github"), "test-plugin", &archive_v1, None)
        .unwrap();

    // plugin.json がないアーカイブ
    let bad_archive = create_test_archive(&[("repo-main/readme.md", "# No plugin.json here")]);

    // アトミック更新 → エラー
    let result = cache.atomic_update(Some("github"), "test-plugin", &bad_archive);
    assert!(result.is_err());

    // 元のプラグインは残っている
    let plugin_path = cache.plugin_path(Some("github"), "test-plugin");
    assert!(plugin_path.join("plugin.json").exists());
}

#[test]
fn test_atomic_update_cleans_up_temp_on_failure() {
    let temp_dir = TempDir::new().unwrap();
    let cache = PluginCache::with_cache_dir(temp_dir.path().to_path_buf()).unwrap();

    // 初期プラグインをインストール
    let archive_v1 = create_test_archive(&[(
        "repo-main/plugin.json",
        r#"{"name":"test","version":"1.0.0"}"#,
    )]);
    cache
        .store_from_archive(Some("github"), "test-plugin", &archive_v1, None)
        .unwrap();

    // 不正なアーカイブでアトミック更新
    let bad_archive = create_test_archive(&[("repo-main/other.txt", "no plugin.json")]);

    let _ = cache.atomic_update(Some("github"), "test-plugin", &bad_archive);

    // .temp ディレクトリがクリーンアップされている
    let temp_base = temp_dir.path().join(".temp").join("github").join("test-plugin");
    assert!(
        !temp_base.exists(),
        "Temp directory should be cleaned up after failure"
    );
}

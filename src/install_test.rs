use std::fs;
use std::path::Path;

use tempfile::TempDir;

use super::*;
use crate::component::ComponentKind;
use crate::plugin::{CachedPackage, MarketplaceContent, PluginManifest};
use crate::target::{CodexTarget, CopilotTarget};

/// テスト用 CachedPackage を構築するヘルパー
fn create_test_cached_package(
    base_dir: &Path,
    skill_names: &[&str],
    agent_names: &[&str],
    command_names: &[&str],
) -> CachedPackage {
    // plugin.json を作成
    let manifest_content = serde_json::json!({
        "name": "test-plugin",
        "version": "1.0.0",
        "description": "A test plugin"
    });
    fs::write(base_dir.join("plugin.json"), manifest_content.to_string()).unwrap();

    // Skills ディレクトリ構造を作成
    if !skill_names.is_empty() {
        let skills_dir = base_dir.join("skills");
        for name in skill_names {
            let skill_dir = skills_dir.join(name);
            fs::create_dir_all(&skill_dir).unwrap();
            fs::write(skill_dir.join("SKILL.md"), "# Test Skill").unwrap();
        }
    }

    // Agents ファイルを作成
    if !agent_names.is_empty() {
        let agents_dir = base_dir.join("agents");
        fs::create_dir_all(&agents_dir).unwrap();
        for name in agent_names {
            fs::write(
                agents_dir.join(format!("{}.agent.md", name)),
                "# Test Agent",
            )
            .unwrap();
        }
    }

    // Commands ファイルを作成
    if !command_names.is_empty() {
        let commands_dir = base_dir.join("commands");
        fs::create_dir_all(&commands_dir).unwrap();
        for name in command_names {
            fs::write(
                commands_dir.join(format!("{}.prompt.md", name)),
                "# Test Command",
            )
            .unwrap();
        }
    }

    let manifest = PluginManifest::load(&base_dir.join("plugin.json")).unwrap();

    CachedPackage {
        name: "test-plugin".to_string(),
        cache_key: None,
        marketplace: Some("test-marketplace".to_string()),
        path: base_dir.to_path_buf(),
        manifest,
        git_ref: "main".to_string(),
        commit_sha: "abc123".to_string(),
        marketplace_manifest: None,
    }
}

// =============================================================================
// scan_plugin テスト
// =============================================================================

#[test]
fn test_scan_plugin_returns_all_components_when_no_filter() {
    let temp = TempDir::new().unwrap();
    let cached = create_test_cached_package(
        temp.path(),
        &["skill-a", "skill-b"],
        &["agent-a"],
        &["cmd-a"],
    );
    let package = MarketplaceContent::from(cached);

    let result = scan_plugin(&package, None).unwrap();

    assert_eq!(result.components.len(), 4);
    assert_eq!(result.name(), "test-plugin");
    assert_eq!(result.marketplace(), Some("test-marketplace"));
}

#[test]
fn test_scan_plugin_filters_by_skill_only() {
    let temp = TempDir::new().unwrap();
    let cached = create_test_cached_package(
        temp.path(),
        &["skill-a", "skill-b"],
        &["agent-a"],
        &["cmd-a"],
    );
    let package = MarketplaceContent::from(cached);

    let filter = [ComponentKind::Skill];
    let result = scan_plugin(&package, Some(&filter)).unwrap();

    assert_eq!(result.components.len(), 2);
    assert!(result
        .components
        .iter()
        .all(|c| c.kind == ComponentKind::Skill));
}

#[test]
fn test_scan_plugin_empty_components() {
    let temp = TempDir::new().unwrap();
    let cached = create_test_cached_package(temp.path(), &[], &[], &[]);
    let package = MarketplaceContent::from(cached);

    let result = scan_plugin(&package, None).unwrap();

    assert!(result.components.is_empty());
}

#[test]
fn test_scan_plugin_filter_with_no_match() {
    let temp = TempDir::new().unwrap();
    let cached = create_test_cached_package(temp.path(), &["skill-a"], &[], &[]);
    let package = MarketplaceContent::from(cached);

    let filter = [ComponentKind::Agent];
    let result = scan_plugin(&package, Some(&filter)).unwrap();

    assert!(result.components.is_empty());
}

// =============================================================================
// place_plugin テスト
// =============================================================================

#[test]
fn test_place_plugin_skill_to_codex() {
    let temp = TempDir::new().unwrap();
    let project_dir = TempDir::new().unwrap();
    let cached = create_test_cached_package(temp.path(), &["my-skill"], &[], &[]);
    let package = MarketplaceContent::from(cached);
    let scanned = scan_plugin(&package, None).unwrap();

    let targets: Vec<Box<dyn crate::target::Target>> = vec![Box::new(CodexTarget::new())];

    let result = place_plugin(&PlaceRequest {
        scanned: &scanned,
        targets: &targets,
        scope: crate::component::Scope::Project,
        project_root: project_dir.path(),
    });

    assert_eq!(result.plugin_name, "test-plugin");
    assert_eq!(result.successes.len(), 1);
    assert!(result.failures.is_empty());
    assert_eq!(result.successes[0].component_name, "my-skill");
    assert_eq!(result.successes[0].component_kind, ComponentKind::Skill);
    assert_eq!(result.successes[0].target, "codex");
}

#[test]
fn test_place_plugin_unsupported_component_skipped() {
    let temp = TempDir::new().unwrap();
    let project_dir = TempDir::new().unwrap();
    // Antigravity only supports Skills
    let cached = create_test_cached_package(temp.path(), &[], &["my-agent"], &[]);
    let package = MarketplaceContent::from(cached);
    let scanned = scan_plugin(&package, None).unwrap();

    let targets: Vec<Box<dyn crate::target::Target>> =
        vec![Box::new(crate::target::AntigravityTarget::new())];

    let result = place_plugin(&PlaceRequest {
        scanned: &scanned,
        targets: &targets,
        scope: crate::component::Scope::Project,
        project_root: project_dir.path(),
    });

    // Antigravity doesn't support Agent, so it should be skipped
    assert!(result.successes.is_empty());
    assert!(result.failures.is_empty());
}

#[test]
fn test_place_plugin_empty_components() {
    let temp = TempDir::new().unwrap();
    let project_dir = TempDir::new().unwrap();
    let cached = create_test_cached_package(temp.path(), &[], &[], &[]);
    let package = MarketplaceContent::from(cached);
    let scanned = scan_plugin(&package, None).unwrap();

    let targets: Vec<Box<dyn crate::target::Target>> = vec![Box::new(CodexTarget::new())];

    let result = place_plugin(&PlaceRequest {
        scanned: &scanned,
        targets: &targets,
        scope: crate::component::Scope::Project,
        project_root: project_dir.path(),
    });

    assert!(result.successes.is_empty());
    assert!(result.failures.is_empty());
}

#[test]
fn test_place_plugin_multiple_targets() {
    let temp = TempDir::new().unwrap();
    let project_dir = TempDir::new().unwrap();
    let cached = create_test_cached_package(temp.path(), &["my-skill"], &[], &[]);
    let package = MarketplaceContent::from(cached);
    let scanned = scan_plugin(&package, None).unwrap();

    let targets: Vec<Box<dyn crate::target::Target>> =
        vec![Box::new(CodexTarget::new()), Box::new(CopilotTarget::new())];

    let result = place_plugin(&PlaceRequest {
        scanned: &scanned,
        targets: &targets,
        scope: crate::component::Scope::Project,
        project_root: project_dir.path(),
    });

    assert_eq!(result.successes.len(), 2);
    let target_names: Vec<&str> = result.successes.iter().map(|s| s.target.as_str()).collect();
    assert!(target_names.contains(&"codex"));
    assert!(target_names.contains(&"copilot"));
}

// =============================================================================
// cache_key 伝搬テスト
// =============================================================================

#[test]
fn test_scan_plugin_propagates_cache_key() {
    let temp = TempDir::new().unwrap();
    let mut cached = create_test_cached_package(temp.path(), &["skill-a"], &[], &[]);
    cached.cache_key = Some("owner--repo".to_string());
    let package = MarketplaceContent::from(cached);

    let result = scan_plugin(&package, None).unwrap();

    assert_eq!(result.cache_key(), "owner--repo");
}

#[test]
fn test_scan_plugin_cache_key_none_falls_back_to_name() {
    let temp = TempDir::new().unwrap();
    let cached = create_test_cached_package(temp.path(), &["skill-a"], &[], &[]);
    let package = MarketplaceContent::from(cached);

    let result = scan_plugin(&package, None).unwrap();

    assert_eq!(result.cache_key(), "test-plugin");
}

/// place_plugin が cache_key を使って PluginOrigin を構成することを検証
#[test]
fn test_place_plugin_uses_cache_key_for_origin() {
    let temp = TempDir::new().unwrap();
    let project_dir = TempDir::new().unwrap();
    let mut cached = create_test_cached_package(temp.path(), &["my-skill"], &[], &[]);
    // name はマニフェスト名、cache_key はキャッシュディレクトリ名
    cached.name = "My Plugin".to_string();
    cached.cache_key = Some("owner--repo".to_string());
    let package = MarketplaceContent::from(cached);
    let scanned = scan_plugin(&package, None).unwrap();

    // cache_key が "owner--repo" であることを確認
    assert_eq!(scanned.cache_key(), "owner--repo");
    assert_eq!(scanned.name(), "My Plugin");

    let targets: Vec<Box<dyn crate::target::Target>> = vec![Box::new(CodexTarget::new())];

    let result = place_plugin(&PlaceRequest {
        scanned: &scanned,
        targets: &targets,
        scope: crate::component::Scope::Project,
        project_root: project_dir.path(),
    });

    // PluginOrigin の plugin フィールドが cache_key() ("owner--repo") を使用していることを
    // 配置先パスに "owner--repo" が含まれることで間接的に検証
    assert_eq!(result.plugin_name, "My Plugin");
    assert_eq!(result.successes.len(), 1);
    let target_path = result.successes[0].target_path.to_string_lossy();
    assert!(
        target_path.contains("owner--repo"),
        "target path should contain cache_key 'owner--repo', got: {}",
        target_path
    );
}

// =============================================================================
// download_plugin テスト（オフライン）
// =============================================================================

#[tokio::test]
async fn test_download_plugin_with_cache_invalid_source_returns_error() {
    let temp_cache = TempDir::new().unwrap();
    let cache =
        crate::plugin::PackageCache::with_cache_dir(temp_cache.path().to_path_buf()).unwrap();

    // "/" のような不正なソース文字列は parse_source で確実に失敗し、早期にエラーとなる
    let result = download_plugin_with_cache("/", false, &cache).await;
    assert!(result.is_err());
}

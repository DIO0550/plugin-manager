use std::fs;
use std::path::Path;

use tempfile::TempDir;

use super::*;
use crate::component::ComponentKind;
use crate::plugin::{CachedPlugin, PluginManifest};
use crate::target::{CodexTarget, CopilotTarget};

/// テスト用 CachedPlugin を構築するヘルパー
///
/// 指定されたスキル名で実際のディレクトリ構造を作成し、
/// `CachedPlugin::components()` が正しくスキャンできるようにする。
fn create_test_cached_plugin(
    base_dir: &Path,
    skill_names: &[&str],
    agent_names: &[&str],
    command_names: &[&str],
) -> CachedPlugin {
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

    CachedPlugin {
        name: "test-plugin".to_string(),
        marketplace: Some("test-marketplace".to_string()),
        path: base_dir.to_path_buf(),
        manifest,
        git_ref: "main".to_string(),
        commit_sha: "abc123".to_string(),
    }
}

/// テスト用 DownloadedPlugin を構築するヘルパー
fn create_test_downloaded(cached: CachedPlugin) -> DownloadedPlugin {
    DownloadedPlugin::from_cached(cached)
}

// =============================================================================
// scan_plugin テスト
// =============================================================================

#[test]
fn test_scan_plugin_returns_all_components_when_no_filter() {
    let temp = TempDir::new().unwrap();
    let cached = create_test_cached_plugin(
        temp.path(),
        &["skill-a", "skill-b"],
        &["agent-a"],
        &["cmd-a"],
    );
    let downloaded = create_test_downloaded(cached);

    let result = scan_plugin(&downloaded, None).unwrap();

    assert_eq!(result.components.len(), 4);
    assert_eq!(result.name, "test-plugin");
    assert_eq!(result.marketplace, Some("test-marketplace".to_string()));
}

#[test]
fn test_scan_plugin_filters_by_skill_only() {
    let temp = TempDir::new().unwrap();
    let cached = create_test_cached_plugin(
        temp.path(),
        &["skill-a", "skill-b"],
        &["agent-a"],
        &["cmd-a"],
    );
    let downloaded = create_test_downloaded(cached);

    let filter = [ComponentKind::Skill];
    let result = scan_plugin(&downloaded, Some(&filter)).unwrap();

    assert_eq!(result.components.len(), 2);
    assert!(result
        .components
        .iter()
        .all(|c| c.kind == ComponentKind::Skill));
}

#[test]
fn test_scan_plugin_empty_components() {
    let temp = TempDir::new().unwrap();
    let cached = create_test_cached_plugin(temp.path(), &[], &[], &[]);
    let downloaded = create_test_downloaded(cached);

    let result = scan_plugin(&downloaded, None).unwrap();

    assert!(result.components.is_empty());
}

#[test]
fn test_scan_plugin_filter_with_no_match() {
    let temp = TempDir::new().unwrap();
    let cached = create_test_cached_plugin(temp.path(), &["skill-a"], &[], &[]);
    let downloaded = create_test_downloaded(cached);

    let filter = [ComponentKind::Agent];
    let result = scan_plugin(&downloaded, Some(&filter)).unwrap();

    assert!(result.components.is_empty());
}

// =============================================================================
// place_plugin テスト
// =============================================================================

#[test]
fn test_place_plugin_skill_to_codex() {
    let temp = TempDir::new().unwrap();
    let project_dir = TempDir::new().unwrap();
    let cached = create_test_cached_plugin(temp.path(), &["my-skill"], &[], &[]);
    let downloaded = create_test_downloaded(cached);
    let scanned = scan_plugin(&downloaded, None).unwrap();

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
    let cached = create_test_cached_plugin(temp.path(), &[], &["my-agent"], &[]);
    let downloaded = create_test_downloaded(cached);
    let scanned = scan_plugin(&downloaded, None).unwrap();

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
    let cached = create_test_cached_plugin(temp.path(), &[], &[], &[]);
    let downloaded = create_test_downloaded(cached);
    let scanned = scan_plugin(&downloaded, None).unwrap();

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
    let cached = create_test_cached_plugin(temp.path(), &["my-skill"], &[], &[]);
    let downloaded = create_test_downloaded(cached);
    let scanned = scan_plugin(&downloaded, None).unwrap();

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
// DownloadedPlugin テスト
// =============================================================================

#[test]
fn test_downloaded_plugin_from_cached() {
    let temp = TempDir::new().unwrap();
    let cached = create_test_cached_plugin(temp.path(), &[], &[], &[]);
    let downloaded = create_test_downloaded(cached);

    assert_eq!(downloaded.name(), "test-plugin");
    assert_eq!(downloaded.version(), "1.0.0");
    assert_eq!(downloaded.description(), Some("A test plugin"));
    assert_eq!(downloaded.marketplace(), Some("test-marketplace"));
    assert_eq!(downloaded.cached_path(), temp.path());
}

// =============================================================================
// download_plugin テスト（オフライン）
// =============================================================================

#[tokio::test]
async fn test_download_plugin_with_cache_invalid_source_returns_error() {
    let temp_cache = TempDir::new().unwrap();
    let cache =
        crate::plugin::PluginCache::with_cache_dir(temp_cache.path().to_path_buf()).unwrap();

    // "/" のような不正なソース文字列は parse_source で確実に失敗し、早期にエラーとなる
    let result = download_plugin_with_cache("/", false, &cache).await;
    assert!(result.is_err());
}

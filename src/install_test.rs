use std::fs;
use std::path::Path;

use tempfile::TempDir;

use super::*;
use crate::component::ComponentKind;
use crate::hooks::converter::{ConversionWarning, SourceFormat};
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
        id: None,
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
    let package = MarketplaceContent::try_from(cached).unwrap();

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
    let package = MarketplaceContent::try_from(cached).unwrap();

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
    let package = MarketplaceContent::try_from(cached).unwrap();

    let result = scan_plugin(&package, None).unwrap();

    assert!(result.components.is_empty());
}

#[test]
fn test_scan_plugin_filter_with_no_match() {
    let temp = TempDir::new().unwrap();
    let cached = create_test_cached_package(temp.path(), &["skill-a"], &[], &[]);
    let package = MarketplaceContent::try_from(cached).unwrap();

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
    let package = MarketplaceContent::try_from(cached).unwrap();
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
    assert_eq!(result.successes[0].component_name, "test-plugin_my-skill");
    assert_eq!(result.successes[0].component_kind, ComponentKind::Skill);
    assert_eq!(result.successes[0].target, "codex");
}

#[test]
fn test_place_plugin_unsupported_component_skipped() {
    let temp = TempDir::new().unwrap();
    let project_dir = TempDir::new().unwrap();
    // Antigravity only supports Skills
    let cached = create_test_cached_package(temp.path(), &[], &["my-agent"], &[]);
    let package = MarketplaceContent::try_from(cached).unwrap();
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
    let package = MarketplaceContent::try_from(cached).unwrap();
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
    let package = MarketplaceContent::try_from(cached).unwrap();
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
// id 伝搬テスト
// =============================================================================

#[test]
fn test_scan_plugin_propagates_id() {
    let temp = TempDir::new().unwrap();
    let mut cached = create_test_cached_package(temp.path(), &["skill-a"], &[], &[]);
    cached.id = Some("owner--repo".to_string());
    let package = MarketplaceContent::try_from(cached).unwrap();

    let result = scan_plugin(&package, None).unwrap();

    assert_eq!(result.id(), "owner--repo");
}

#[test]
fn test_scan_plugin_id_none_falls_back_to_name() {
    let temp = TempDir::new().unwrap();
    let cached = create_test_cached_package(temp.path(), &["skill-a"], &[], &[]);
    let package = MarketplaceContent::try_from(cached).unwrap();

    let result = scan_plugin(&package, None).unwrap();

    assert_eq!(result.id(), "test-plugin");
}

/// place_plugin が id を使って PluginOrigin を構成することを検証
#[test]
fn test_place_plugin_uses_id_for_origin() {
    let temp = TempDir::new().unwrap();
    let project_dir = TempDir::new().unwrap();
    let mut cached = create_test_cached_package(temp.path(), &["my-skill"], &[], &[]);
    // name はマニフェスト名、id はキャッシュディレクトリ名
    cached.name = "My Plugin".to_string();
    cached.manifest.name = "My Plugin".to_string();
    cached.id = Some("owner--repo".to_string());
    let package = MarketplaceContent::try_from(cached).unwrap();
    let scanned = scan_plugin(&package, None).unwrap();

    // id が "owner--repo" であることを確認
    assert_eq!(scanned.id(), "owner--repo");
    assert_eq!(scanned.name(), "My Plugin");

    let targets: Vec<Box<dyn crate::target::Target>> = vec![Box::new(CodexTarget::new())];

    let result = place_plugin(&PlaceRequest {
        scanned: &scanned,
        targets: &targets,
        scope: crate::component::Scope::Project,
        project_root: project_dir.path(),
    });

    // フラット配置: target_path 末尾は flattened_name (= "{plugin}_{original}")
    assert_eq!(result.plugin_name, "My Plugin");
    assert_eq!(result.successes.len(), 1);
    let component_name = &result.successes[0].component_name;
    let target_path = &result.successes[0].target_path;
    let last_segment = target_path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("");
    // ディレクトリ配置 (Skill) は完全一致、ファイル配置 (Agent/Command/Hook) は
    // `<flattened_name>.<ext>` のため接頭辞 + `.` で判定する。
    let matches = last_segment == component_name.as_str()
        || last_segment.starts_with(&format!("{}.", component_name));
    assert!(
        matches,
        "target path last segment should be flattened component name '{}', got: {}",
        component_name,
        target_path.display()
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

// =============================================================================
// Hook 配置時の hook_source_format 伝播テスト
// =============================================================================

/// `hooks/` 配下に 1 つの hook ファイルを持つ test fixture を作成
fn create_test_hook_package(base_dir: &Path, hook_name: &str, hook_json: &str) -> CachedPackage {
    let manifest_content = serde_json::json!({
        "name": "test-plugin",
        "version": "1.0.0",
        "description": "A test plugin"
    });
    fs::write(base_dir.join("plugin.json"), manifest_content.to_string()).unwrap();

    let hooks_dir = base_dir.join("hooks");
    fs::create_dir_all(&hooks_dir).unwrap();
    fs::write(hooks_dir.join(format!("{}.json", hook_name)), hook_json).unwrap();

    let manifest = PluginManifest::load(&base_dir.join("plugin.json")).unwrap();

    CachedPackage {
        name: "test-plugin".to_string(),
        id: None,
        marketplace: Some("test-marketplace".to_string()),
        path: base_dir.to_path_buf(),
        manifest,
        git_ref: "main".to_string(),
        commit_sha: "abc123".to_string(),
        marketplace_manifest: None,
    }
}

#[test]
fn test_place_plugin_claude_code_hook_propagates_source_format() {
    let temp = TempDir::new().unwrap();
    let project_dir = TempDir::new().unwrap();

    // Claude Code 形式 Hook（PreToolUse, command 型 → wrapper を生成）
    let claude_code_hook = r#"{
  "hooks": {
    "PreToolUse": [
      {
        "hooks": [
          {
            "type": "command",
            "command": "echo 'pre check'"
          }
        ]
      }
    ]
  }
}"#;
    let cached = create_test_hook_package(temp.path(), "my-hook", claude_code_hook);
    let package = MarketplaceContent::try_from(cached).unwrap();
    let scanned = scan_plugin(&package, None).unwrap();

    let targets: Vec<Box<dyn crate::target::Target>> = vec![Box::new(CopilotTarget::new())];

    let result = place_plugin(&PlaceRequest {
        scanned: &scanned,
        targets: &targets,
        scope: crate::component::Scope::Project,
        project_root: project_dir.path(),
    });

    assert_eq!(result.successes.len(), 1, "failures: {:?}", result.failures);
    let success = &result.successes[0];
    assert_eq!(success.component_kind, ComponentKind::Hook);
    assert_eq!(
        success.hook_source_format,
        Some(SourceFormat::ClaudeCode),
        "Claude Code 形式 Hook では hook_source_format == Some(ClaudeCode) であるべき"
    );
    assert!(
        success.script_count > 0,
        "Claude Code 形式の command フックは wrapper を生成するため script_count > 0"
    );
}

#[test]
fn test_place_plugin_copilot_hook_with_missing_version_propagates_target_format() {
    let temp = TempDir::new().unwrap();
    let project_dir = TempDir::new().unwrap();

    // Copilot 形式 + version 欠落 → MissingVersion warning が 1 件、wrapper なし。
    // この経路で `hook_source_format == Some(SourceFormat::TargetFormat)` を担保する
    // （false positive 防止：suffix を出さないことの根拠）。
    let copilot_hook = r#"{"hooks":{"preToolUse":[{"type":"command","bash":"echo hi"}]}}"#;
    let cached = create_test_hook_package(temp.path(), "copilot-hook", copilot_hook);
    let package = MarketplaceContent::try_from(cached).unwrap();
    let scanned = scan_plugin(&package, None).unwrap();

    let targets: Vec<Box<dyn crate::target::Target>> = vec![Box::new(CopilotTarget::new())];

    let result = place_plugin(&PlaceRequest {
        scanned: &scanned,
        targets: &targets,
        scope: crate::component::Scope::Project,
        project_root: project_dir.path(),
    });

    assert_eq!(result.successes.len(), 1, "failures: {:?}", result.failures);
    let success = &result.successes[0];
    assert_eq!(success.component_kind, ComponentKind::Hook);
    assert_eq!(
        success.hook_source_format,
        Some(SourceFormat::TargetFormat),
        "Copilot 形式 passthrough では hook_source_format == Some(TargetFormat) であるべき"
    );
    assert!(
        success
            .hook_warnings
            .iter()
            .any(|w| matches!(w, ConversionWarning::MissingVersion)),
        "MissingVersion warning が伝播しているはず"
    );
    assert_eq!(success.script_count, 0);
}

#[test]
fn test_place_plugin_codex_hook_installs_inline_hooks_json() {
    let temp = TempDir::new().unwrap();
    let project_dir = TempDir::new().unwrap();

    let claude_code_hook = r#"{
  "hooks": {
    "PreToolUse": [
      {
        "matcher": "Bash",
        "hooks": [
          {
            "type": "command",
            "command": "echo 'pre check'",
            "timeout": 5
          }
        ]
      }
    ]
  }
}"#;
    let cached = create_test_hook_package(temp.path(), "my-hook", claude_code_hook);
    let package = MarketplaceContent::try_from(cached).unwrap();
    let scanned = scan_plugin(&package, None).unwrap();

    let targets: Vec<Box<dyn crate::target::Target>> = vec![Box::new(CodexTarget::new())];

    let result = place_plugin(&PlaceRequest {
        scanned: &scanned,
        targets: &targets,
        scope: crate::component::Scope::Project,
        project_root: project_dir.path(),
    });

    assert_eq!(result.successes.len(), 1, "failures: {:?}", result.failures);
    let success = &result.successes[0];
    assert_eq!(success.target, "codex");
    assert_eq!(success.component_kind, ComponentKind::Hook);
    assert_eq!(
        success.target_path,
        project_dir.path().join(".codex/hooks.json")
    );
    assert_eq!(success.hook_source_format, Some(SourceFormat::ClaudeCode));
    assert_eq!(
        success.script_count, 0,
        "Codex command hooks stay inline and do not generate wrapper scripts"
    );
    assert_eq!(success.hook_count, 1);

    let rendered = fs::read_to_string(&success.target_path).unwrap();
    let json: serde_json::Value = serde_json::from_str(&rendered).unwrap();
    assert_eq!(json["hooks"]["PreToolUse"][0]["matcher"], "Bash");
    assert_eq!(
        json["hooks"]["PreToolUse"][0]["hooks"][0]["command"],
        "echo 'pre check'"
    );
    assert!(json["hooks"]["PreToolUse"][0]["hooks"][0]
        .get("bash")
        .is_none());
}

#[test]
fn test_place_plugin_codex_rejects_multiple_hook_components() {
    let temp = TempDir::new().unwrap();
    let project_dir = TempDir::new().unwrap();
    let hook_json = r#"{
  "hooks": {
    "PreToolUse": [
      {
        "matcher": "Bash",
        "hooks": [
          {
            "type": "command",
            "command": "echo 'pre check'"
          }
        ]
      }
    ]
  }
}"#;

    let manifest_content = serde_json::json!({
        "name": "test-plugin",
        "version": "1.0.0",
        "description": "A test plugin"
    });
    fs::write(
        temp.path().join("plugin.json"),
        manifest_content.to_string(),
    )
    .unwrap();
    let hooks_dir = temp.path().join("hooks");
    fs::create_dir_all(&hooks_dir).unwrap();
    fs::write(hooks_dir.join("first.json"), hook_json).unwrap();
    fs::write(hooks_dir.join("second.json"), hook_json).unwrap();

    let manifest = PluginManifest::load(&temp.path().join("plugin.json")).unwrap();
    let cached = CachedPackage {
        name: "test-plugin".to_string(),
        id: None,
        marketplace: Some("test-marketplace".to_string()),
        path: temp.path().to_path_buf(),
        manifest,
        git_ref: "main".to_string(),
        commit_sha: "abc123".to_string(),
        marketplace_manifest: None,
    };
    let package = MarketplaceContent::try_from(cached).unwrap();
    let scanned = scan_plugin(&package, None).unwrap();

    let targets: Vec<Box<dyn crate::target::Target>> = vec![Box::new(CodexTarget::new())];
    let result = place_plugin(&PlaceRequest {
        scanned: &scanned,
        targets: &targets,
        scope: crate::component::Scope::Project,
        project_root: project_dir.path(),
    });

    assert!(result.successes.is_empty());
    assert_eq!(result.failures.len(), 2);
    assert!(result
        .failures
        .iter()
        .all(|failure| failure.component_kind == ComponentKind::Hook));
    assert!(result
        .failures
        .iter()
        .all(|failure| failure.error.contains("would overwrite each other")));
    assert!(!project_dir.path().join(".codex/hooks.json").exists());
}

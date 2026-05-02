use crate::component::{
    Component, ComponentDeployment, ComponentKind, ConversionConfig, DeploymentOutput, Scope,
};
use crate::hooks::converter::{ConversionWarning, SourceFormat};
use crate::hooks::name::HookName;
use crate::target::TargetKind;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

/// Claude Code 形式の Hook JSON テストデータ
fn sample_claude_code_hook_json() -> &'static str {
    r#"{
  "hooks": {
    "PreToolUse": [
      {
        "hooks": [
          {
            "type": "command",
            "command": "echo 'pre-tool check'"
          }
        ]
      }
    ]
  }
}"#
}

fn hook_deployment(
    source: PathBuf,
    name: &str,
    target: PathBuf,
    plugin_root: Option<PathBuf>,
) -> ComponentDeployment {
    ComponentDeployment::builder()
        .component(Component {
            kind: ComponentKind::Hook,
            name: name.to_string(),
            path: source,
        })
        .scope(Scope::Project)
        .target_path(target)
        .conversion(ConversionConfig::Hook {
            target_kind: TargetKind::Copilot,
            plugin_root,
        })
        .build()
        .unwrap()
}

#[test]
fn test_hook_convert_without_plugin_root_errors_when_wrappers_needed() {
    let temp = TempDir::new().unwrap();
    let source = temp.path().join("hook.json");
    let target = temp.path().join("dest/hook.json");

    // Claude Code 形式 → wrapper が生成される → plugin_root 必須
    fs::write(&source, sample_claude_code_hook_json()).unwrap();

    let deployment = hook_deployment(source, "test-hook", target, None);

    let err = deployment.execute().unwrap_err();
    assert!(err.to_string().contains("plugin_root"));
}

#[test]
fn test_hook_convert_without_plugin_root_ok_for_warnings_only() {
    let temp = TempDir::new().unwrap();
    let source = temp.path().join("hook.json");
    let target = temp.path().join("dest/hook.json");

    // Copilot CLI 形式 + version 欠落 → warnings あり、wrapper なし → plugin_root 不要
    let json = r#"{"hooks":{"preToolUse":[{"type":"command","bash":"echo hi"}]}}"#;
    fs::write(&source, json).unwrap();

    let deployment = hook_deployment(source, "test-hook", target, None);

    let result = deployment.execute().unwrap();
    match result {
        DeploymentOutput::HookConverted(hr) => {
            assert!(!hr.warnings.is_empty());
            assert_eq!(hr.script_count, 0);
            // Copilot 形式（version 欠落）入力では source_format は TargetFormat になる。
            assert_eq!(hr.source_format, SourceFormat::TargetFormat);
        }
        _ => panic!("Expected HookConverted with warnings only"),
    }
}

#[test]
fn test_hook_convert_false_copies_file() {
    let temp = TempDir::new().unwrap();
    let source = temp.path().join("hook.json");
    let target = temp.path().join("dest/hook.json");

    fs::write(&source, r#"{"hooks":{}}"#).unwrap();

    // 変換なし (ConversionConfig::None) → ファイルコピー
    let deployment = ComponentDeployment::builder()
        .component(Component {
            kind: ComponentKind::Hook,
            name: "test-hook".to_string(),
            path: source,
        })
        .scope(Scope::Project)
        .target_path(&target)
        .build()
        .unwrap();

    let result = deployment.execute().unwrap();
    assert!(matches!(result, DeploymentOutput::Copied));
    assert!(target.exists());
    assert_eq!(fs::read_to_string(&target).unwrap(), r#"{"hooks":{}}"#);
}

#[test]
fn test_hook_convert_true_deploys_converted() {
    let temp = TempDir::new().unwrap();
    let source = temp.path().join("hook.json");
    let target_dir = temp.path().join("dest");
    let target = target_dir.join("my-hook.json");
    let plugin_root = temp.path().join("cache/plugin");
    fs::create_dir_all(&plugin_root).unwrap();

    fs::write(&source, sample_claude_code_hook_json()).unwrap();

    let deployment = hook_deployment(source, "my-hook", target.clone(), Some(plugin_root));

    let result = deployment.execute().unwrap();

    match result {
        DeploymentOutput::HookConverted(hr) => {
            assert!(hr.script_count > 0);
            // Claude Code 形式入力 → source_format は ClaudeCode
            assert_eq!(hr.source_format, SourceFormat::ClaudeCode);
        }
        _ => panic!("Expected HookConverted"),
    }

    // JSON ファイルが配置されていること
    assert!(target.exists());
    let json_content = fs::read_to_string(&target).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json_content).unwrap();
    assert_eq!(parsed.get("version").unwrap(), 1);

    let scripts_dir = target_dir.join("wrappers").join("my-hook");
    assert!(scripts_dir.exists());
    let script_files: Vec<_> = fs::read_dir(&scripts_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .collect();
    assert!(!script_files.is_empty());

    assert!(json_content.contains("./wrappers/my-hook/"));
}

/// Copilot CLI 形式入力 → そのまま配置（version があるので変換されない）
#[test]
fn test_hook_convert_copilot_format_passthrough() {
    let temp = TempDir::new().unwrap();
    let source = temp.path().join("hook.json");
    let target = temp.path().join("dest/hook.json");
    let plugin_root = temp.path().join("cache/plugin");
    fs::create_dir_all(&plugin_root).unwrap();

    let copilot_json = r#"{
  "version": 1,
  "hooks": {
    "preToolUse": [
      {
        "type": "command",
        "bash": "echo hello"
      }
    ]
  }
}"#;
    fs::write(&source, copilot_json).unwrap();

    let deployment = hook_deployment(source, "copilot-hook", target.clone(), Some(plugin_root));

    let result = deployment.execute().unwrap();

    // Copilot CLI 形式はファイルコピーにフォールバック
    assert!(matches!(result, DeploymentOutput::Copied));

    assert!(target.exists());
    assert_eq!(fs::read_to_string(&target).unwrap(), copilot_json);
}

/// @@PLUGIN_ROOT@@ が plugin_root の実パスに置換されること
#[test]
fn test_hook_convert_plugin_root_replacement() {
    let temp = TempDir::new().unwrap();
    let source = temp.path().join("hook.json");
    let target_dir = temp.path().join("dest");
    let target = target_dir.join("test-hook.json");
    let plugin_root = temp.path().join("cache/my-plugin");
    fs::create_dir_all(&plugin_root).unwrap();

    fs::write(&source, sample_claude_code_hook_json()).unwrap();

    let deployment = hook_deployment(source, "test-hook", target, Some(plugin_root));

    deployment.execute().unwrap();

    let wrappers_dir = target_dir.join("wrappers").join("test-hook");
    for entry in fs::read_dir(&wrappers_dir).unwrap() {
        let entry = entry.unwrap();
        let content = fs::read_to_string(entry.path()).unwrap();
        assert!(
            !content.contains("@@PLUGIN_ROOT@@"),
            "@@PLUGIN_ROOT@@ should be replaced in {:?}",
            entry.path()
        );
    }
}

/// 実行権限 0o755 の検証（Unix のみ）
#[cfg(unix)]
#[test]
fn test_hook_convert_wrapper_executable_permission() {
    use std::os::unix::fs::PermissionsExt;

    let temp = TempDir::new().unwrap();
    let source = temp.path().join("hook.json");
    let target_dir = temp.path().join("dest");
    let target = target_dir.join("perm-hook.json");
    let plugin_root = temp.path().join("cache/plugin");
    fs::create_dir_all(&plugin_root).unwrap();

    fs::write(&source, sample_claude_code_hook_json()).unwrap();

    let deployment = hook_deployment(source, "perm-hook", target, Some(plugin_root));

    deployment.execute().unwrap();

    let wrappers_dir = target_dir.join("wrappers").join("perm-hook");
    for entry in fs::read_dir(&wrappers_dir).unwrap() {
        let entry = entry.unwrap();
        let perms = fs::metadata(entry.path()).unwrap().permissions();
        assert_eq!(perms.mode() & 0o777, 0o755);
    }
}

/// 存在しないソースファイル → Err
#[test]
fn test_hook_convert_missing_source_returns_err() {
    let temp = TempDir::new().unwrap();
    let source = temp.path().join("nonexistent.json");
    let target = temp.path().join("dest/hook.json");
    let plugin_root = temp.path().join("cache/plugin");

    let deployment = hook_deployment(source, "hook", target, Some(plugin_root));

    assert!(deployment.execute().is_err());
}

/// 親ディレクトリが自動作成されること
#[test]
fn test_hook_convert_creates_parent_dir() {
    let temp = TempDir::new().unwrap();
    let source = temp.path().join("hook.json");
    let target = temp.path().join("deep/nested/dir/hook.json");
    let plugin_root = temp.path().join("cache/plugin");
    fs::create_dir_all(&plugin_root).unwrap();

    fs::write(&source, sample_claude_code_hook_json()).unwrap();

    let deployment = hook_deployment(source, "hook", target.clone(), Some(plugin_root));

    deployment.execute().unwrap();
    assert!(target.exists());
}

/// 全イベント非対応 → 空 hooks + 警告
#[test]
fn test_hook_convert_unsupported_events_produce_warnings() {
    let temp = TempDir::new().unwrap();
    let source = temp.path().join("hook.json");
    let target = temp.path().join("dest/hook.json");
    let plugin_root = temp.path().join("cache/plugin");
    fs::create_dir_all(&plugin_root).unwrap();

    let json = r#"{
  "hooks": {
    "UnknownEvent": [
      { "hooks": [{ "type": "command", "command": "echo test" }] }
    ]
  }
}"#;
    fs::write(&source, json).unwrap();

    let deployment = hook_deployment(source, "hook", target, Some(plugin_root));

    let result = deployment.execute().unwrap();
    match result {
        DeploymentOutput::HookConverted(hr) => {
            assert!(!hr.warnings.is_empty());
            assert!(hr
                .warnings
                .iter()
                .any(|w| matches!(w, ConversionWarning::UnsupportedEvent { .. })));
        }
        _ => panic!("Expected HookConverted"),
    }
}

/// 元スクリプトが配置先にコピーされないこと（@@PLUGIN_ROOT@@ 経由で参照）
#[test]
fn test_hook_convert_original_scripts_not_copied() {
    let temp = TempDir::new().unwrap();
    let source = temp.path().join("hook.json");
    let target_dir = temp.path().join("dest");
    let target = target_dir.join("hook.json");
    let plugin_root = temp.path().join("cache/plugin");
    fs::create_dir_all(&plugin_root).unwrap();

    let scripts_dir = plugin_root.join("scripts");
    fs::create_dir_all(&scripts_dir).unwrap();
    fs::write(
        scripts_dir.join("original.sh"),
        "#!/bin/bash\necho original",
    )
    .unwrap();

    let hook_json = r#"{
  "hooks": {
    "PreToolUse": [
      {
        "hooks": [
          {
            "type": "command",
            "command": "@@PLUGIN_ROOT@@/scripts/original.sh"
          }
        ]
      }
    ]
  }
}"#;
    fs::write(&source, hook_json).unwrap();

    let deployment = hook_deployment(source, "hook", target, Some(plugin_root));

    deployment.execute().unwrap();

    assert!(!target_dir.join("scripts").exists());
    assert!(!target_dir.join("original.sh").exists());

    let wrappers_dir = target_dir.join("wrappers").join("hook");
    assert!(wrappers_dir.exists());
    for entry in fs::read_dir(&wrappers_dir).unwrap() {
        let content = fs::read_to_string(entry.unwrap().path()).unwrap();
        assert!(!content.contains("@@PLUGIN_ROOT@@"));
        assert!(content.contains("original.sh"));
    }
}

/// 複数 Hook ファイルの wrapper が名前衝突しないこと
#[test]
fn test_hook_convert_multiple_hooks_no_name_collision() {
    let temp = TempDir::new().unwrap();
    let source = temp.path().join("hook.json");
    let target_dir = temp.path().join("dest");
    let plugin_root = temp.path().join("cache/plugin");
    fs::create_dir_all(&plugin_root).unwrap();

    fs::write(&source, sample_claude_code_hook_json()).unwrap();

    let target_a = target_dir.join("hook-a.json");
    let deployment_a = hook_deployment(
        source.clone(),
        "hook-a",
        target_a.clone(),
        Some(plugin_root.clone()),
    );
    deployment_a.execute().unwrap();

    let target_b = target_dir.join("hook-b.json");
    let deployment_b = hook_deployment(source, "hook-b", target_b.clone(), Some(plugin_root));
    deployment_b.execute().unwrap();

    assert!(target_dir.join("wrappers/hook-a").exists());
    assert!(target_dir.join("wrappers/hook-b").exists());

    let json_a = fs::read_to_string(&target_a).unwrap();
    let json_b = fs::read_to_string(&target_b).unwrap();
    assert!(json_a.contains("./wrappers/hook-a/"));
    assert!(json_b.contains("./wrappers/hook-b/"));
    assert!(!json_a.contains("./wrappers/hook-b/"));
    assert!(!json_b.contains("./wrappers/hook-a/"));
}

#[test]
fn test_hook_convert_with_unsafe_name_uses_sanitized_dir() {
    let temp = TempDir::new().unwrap();
    let source = temp.path().join("hook.json");
    let target_dir = temp.path().join("dest");
    let target = target_dir.join("hook.json");
    let plugin_root = temp.path().join("cache/plugin");
    fs::create_dir_all(&plugin_root).unwrap();

    fs::write(&source, sample_claude_code_hook_json()).unwrap();

    let deployment = hook_deployment(source, "my hook$name", target.clone(), Some(plugin_root));

    deployment.execute().unwrap();

    let hook_name = HookName::new("my hook$name");
    let safe_name = hook_name.as_safe();

    assert!(target_dir.join("wrappers").join(safe_name).exists());
    let json_content = fs::read_to_string(&target).unwrap();
    assert!(json_content.contains(&format!("./wrappers/{}/", safe_name)));
}

/// 変換後 JSON に version: 1 が含まれること
#[test]
fn test_hook_convert_output_has_version() {
    let temp = TempDir::new().unwrap();
    let source = temp.path().join("hook.json");
    let target = temp.path().join("dest/hook.json");
    let plugin_root = temp.path().join("cache/plugin");
    fs::create_dir_all(&plugin_root).unwrap();

    fs::write(&source, sample_claude_code_hook_json()).unwrap();

    let deployment = hook_deployment(source, "hook", target.clone(), Some(plugin_root));

    deployment.execute().unwrap();

    let content = fs::read_to_string(&target).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();
    assert_eq!(parsed.get("version").unwrap(), 1);
}

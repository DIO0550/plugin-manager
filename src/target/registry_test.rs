use super::*;
use tempfile::TempDir;

fn create_test_registry() -> (TargetRegistry, TempDir) {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("targets.json");
    let registry = TargetRegistry::with_path(config_path);
    (registry, temp_dir)
}

#[test]
fn test_default_config() {
    let config = TargetsConfig::default();
    assert_eq!(config.targets.len(), 3);
    assert!(config.targets.contains(&TargetKind::Antigravity));
    assert!(config.targets.contains(&TargetKind::Codex));
    assert!(config.targets.contains(&TargetKind::Copilot));
}

#[test]
fn test_default_includes_antigravity() {
    let config = TargetsConfig::default();
    assert!(config.targets.contains(&TargetKind::Antigravity));
}

#[test]
fn test_load_nonexistent() {
    let (mut registry, _temp_dir) = create_test_registry();
    let config = registry.load().unwrap();

    // ファイルが存在しない場合はデフォルト
    assert_eq!(config.targets.len(), 3);
    assert!(config.targets.contains(&TargetKind::Antigravity));
    assert!(config.targets.contains(&TargetKind::Codex));
    assert!(config.targets.contains(&TargetKind::Copilot));
}

#[test]
fn test_save_and_load() {
    let (mut registry, _temp_dir) = create_test_registry();

    // 追加して保存
    registry.add(TargetKind::Codex).unwrap();

    // 新しいレジストリで読み込み
    let mut registry2 = TargetRegistry::with_path(registry.config_path.clone());
    let config = registry2.load().unwrap();

    assert!(config.targets.contains(&TargetKind::Codex));
}

#[test]
fn test_add_target() {
    let (mut registry, _temp_dir) = create_test_registry();

    // まず空にする
    registry.load().unwrap();
    registry.config.as_mut().unwrap().targets.clear();

    // Codexを追加
    let result = registry.add(TargetKind::Codex).unwrap();
    assert_eq!(result, AddResult::Added);

    let targets = registry.list().unwrap();
    assert_eq!(targets.len(), 1);
    assert!(targets.contains(&TargetKind::Codex));
}

#[test]
fn test_add_duplicate() {
    let (mut registry, _temp_dir) = create_test_registry();

    // デフォルトでCodexが含まれている
    let result = registry.add(TargetKind::Codex).unwrap();
    assert_eq!(result, AddResult::AlreadyExists);
}

#[test]
fn test_remove_target() {
    let (mut registry, _temp_dir) = create_test_registry();

    let result = registry.remove(TargetKind::Codex).unwrap();
    assert_eq!(result, RemoveResult::Removed);

    let targets = registry.list().unwrap();
    assert!(!targets.contains(&TargetKind::Codex));
    assert!(targets.contains(&TargetKind::Copilot));
}

#[test]
fn test_remove_nonexistent() {
    let (mut registry, _temp_dir) = create_test_registry();

    // Codexを削除
    registry.remove(TargetKind::Codex).unwrap();

    // 再度削除を試みる
    let result = registry.remove(TargetKind::Codex).unwrap();
    assert_eq!(result, RemoveResult::NotFound);
}

#[test]
fn test_load_deduplicates() {
    let (mut registry, temp_dir) = create_test_registry();

    // 重複を含むJSONを直接書き込み
    let json = r#"{"targets": ["codex", "codex", "copilot"]}"#;
    std::fs::write(temp_dir.path().join("targets.json"), json).unwrap();

    let config = registry.load().unwrap();

    // 重複が排除される
    assert_eq!(config.targets.len(), 2);
}

#[test]
fn test_save_creates_valid_json() {
    let (mut registry, temp_dir) = create_test_registry();

    // まずCodexを削除してから追加し直す（saveを発生させるため）
    registry.remove(TargetKind::Codex).unwrap();
    registry.add(TargetKind::Codex).unwrap();

    // ファイルを読み込んでパースできることを確認
    let content = std::fs::read_to_string(temp_dir.path().join("targets.json")).unwrap();
    let parsed: TargetsConfig = serde_json::from_str(&content).unwrap();

    assert!(parsed.targets.contains(&TargetKind::Codex));
}

#[test]
fn test_unknown_target_parse_error() {
    let (mut registry, temp_dir) = create_test_registry();

    // 未知のターゲットを含むJSONを書き込み
    let json = r#"{"targets": ["codex", "invalid"]}"#;
    std::fs::write(temp_dir.path().join("targets.json"), json).unwrap();

    let result = registry.load();
    assert!(result.is_err());
}

#[test]
fn test_state_transitions() {
    let (mut registry, _temp_dir) = create_test_registry();

    assert_eq!(registry.current_state(), "Idle");

    registry.load().unwrap();
    assert_eq!(registry.current_state(), "Loaded");

    // 既存のターゲットを削除（save発生 → Idle）
    registry.remove(TargetKind::Codex).unwrap();
    assert_eq!(registry.current_state(), "Idle");

    // 削除したターゲットを追加（save発生 → Idle）
    registry.add(TargetKind::Codex).unwrap();
    assert_eq!(registry.current_state(), "Idle");
}

#[test]
fn test_normalize_sorts() {
    let mut config = TargetsConfig {
        targets: vec![TargetKind::Copilot, TargetKind::Codex],
    };

    config.normalize();

    // Codex < Copilot の順でソート
    assert_eq!(config.targets[0], TargetKind::Codex);
    assert_eq!(config.targets[1], TargetKind::Copilot);
}

#[test]
fn test_normalize_sorts_antigravity_first() {
    let mut config = TargetsConfig {
        targets: vec![
            TargetKind::Copilot,
            TargetKind::Antigravity,
            TargetKind::Codex,
        ],
    };

    config.normalize();

    // Antigravity < Codex < Copilot の順でソート
    assert_eq!(config.targets[0], TargetKind::Antigravity);
    assert_eq!(config.targets[1], TargetKind::Codex);
    assert_eq!(config.targets[2], TargetKind::Copilot);
}

#[test]
fn test_empty_targets_allowed() {
    let (mut registry, _temp_dir) = create_test_registry();

    // 全て削除
    registry.remove(TargetKind::Antigravity).unwrap();
    registry.remove(TargetKind::Codex).unwrap();
    registry.remove(TargetKind::Copilot).unwrap();

    let targets = registry.list().unwrap();
    assert!(targets.is_empty());
}

#[test]
fn test_antigravity_serialization() {
    let config = TargetsConfig {
        targets: vec![TargetKind::Antigravity],
    };
    let json = serde_json::to_string(&config).unwrap();
    assert!(json.contains("\"antigravity\""));
}

#[test]
fn test_antigravity_deserialization() {
    let json = r#"{"targets": ["antigravity"]}"#;
    let config: TargetsConfig = serde_json::from_str(json).unwrap();
    assert_eq!(config.targets, vec![TargetKind::Antigravity]);
}

use super::*;
use std::fs;
use tempfile::TempDir;

// ---- pure helper: edit_toml_str ----

#[test]
fn edit_empty_input_creates_features_section() {
    let result = edit_toml_str("").unwrap();
    match result {
        EditResult::Changed(s) => {
            assert!(s.contains("[features]"), "missing [features] in: {s}");
            assert!(
                s.contains("codex_hooks = true"),
                "missing codex_hooks key: {s}"
            );
        }
        other => panic!("expected Changed, got {other:?}"),
    }
}

#[test]
fn edit_no_features_section_appends_section() {
    let input = "[model]\nname = \"gpt-5\"\n";
    let result = edit_toml_str(input).unwrap();
    match result {
        EditResult::Changed(s) => {
            assert!(s.contains("[model]"));
            assert!(s.contains("name = \"gpt-5\""));
            assert!(s.contains("[features]"));
            assert!(s.contains("codex_hooks = true"));
        }
        other => panic!("expected Changed, got {other:?}"),
    }
}

#[test]
fn edit_features_section_without_codex_hooks_appends_key() {
    let input = "[features]\nother = true\n";
    let result = edit_toml_str(input).unwrap();
    match result {
        EditResult::Changed(s) => {
            assert!(s.contains("other = true"));
            assert!(s.contains("codex_hooks = true"));
        }
        other => panic!("expected Changed, got {other:?}"),
    }
}

#[test]
fn edit_codex_hooks_true_is_unchanged() {
    let input = "[features]\ncodex_hooks = true\n";
    assert_eq!(edit_toml_str(input).unwrap(), EditResult::Unchanged);
}

#[test]
fn edit_codex_hooks_false_returns_skipped_false() {
    let input = "[features]\ncodex_hooks = false\n";
    assert_eq!(edit_toml_str(input).unwrap(), EditResult::SkippedFalse);
}

#[test]
fn edit_preserves_existing_comments_and_keys() {
    let input = "# user comment\n[model]\nname = \"gpt-5\"\n\n[features]\nother = true\n";
    let result = edit_toml_str(input).unwrap();
    match result {
        EditResult::Changed(s) => {
            assert!(s.contains("# user comment"), "lost comment: {s}");
            assert!(s.contains("name = \"gpt-5\""));
            assert!(s.contains("other = true"));
            assert!(s.contains("codex_hooks = true"));
        }
        other => panic!("expected Changed, got {other:?}"),
    }
}

#[test]
fn edit_invalid_toml_returns_err() {
    let input = "[features\ncodex_hooks = true";
    assert!(edit_toml_str(input).is_err());
}

#[test]
fn edit_features_is_non_table_returns_err() {
    // `features` がテーブルでない場合（文字列・bool 等）は挿入できないため、
    // 黙って Changed と返さず Err を返すことで呼出側に best-effort 警告を出させる。
    let input = "features = \"not-a-table\"\n";
    assert!(edit_toml_str(input).is_err());
}

// ---- apply: apply_codex_hooks_flag ----

#[test]
fn apply_creates_file_when_missing() {
    let temp = TempDir::new().unwrap();
    let path = temp.path().join("subdir/config.toml");
    let outcome = apply_codex_hooks_flag(&path).unwrap();
    assert!(outcome.applied);
    assert!(outcome.skipped_reason.is_none());
    assert_eq!(outcome.target_path, path);
    assert!(path.exists());
    let content = fs::read_to_string(&path).unwrap();
    assert!(content.contains("codex_hooks = true"), "got: {content}");
}

#[test]
fn apply_appends_to_existing_config_without_features_section() {
    let temp = TempDir::new().unwrap();
    let path = temp.path().join("config.toml");
    fs::write(&path, "[model]\nname = \"gpt-5\"\n").unwrap();
    let outcome = apply_codex_hooks_flag(&path).unwrap();
    assert!(outcome.applied);
    let content = fs::read_to_string(&path).unwrap();
    assert!(content.contains("name = \"gpt-5\""));
    assert!(content.contains("[features]"));
    assert!(content.contains("codex_hooks = true"));
}

#[test]
fn apply_is_idempotent_when_true_already_set() {
    let temp = TempDir::new().unwrap();
    let path = temp.path().join("config.toml");
    let before = "[features]\ncodex_hooks = true\n";
    fs::write(&path, before).unwrap();
    let outcome = apply_codex_hooks_flag(&path).unwrap();
    assert!(!outcome.applied);
    assert_eq!(outcome.skipped_reason.as_deref(), Some("already enabled"));
    assert_eq!(fs::read_to_string(&path).unwrap(), before);
}

#[test]
fn apply_respects_explicit_false() {
    let temp = TempDir::new().unwrap();
    let path = temp.path().join("config.toml");
    let before = "[features]\ncodex_hooks = false\n";
    fs::write(&path, before).unwrap();
    let outcome = apply_codex_hooks_flag(&path).unwrap();
    assert!(!outcome.applied);
    assert!(outcome
        .skipped_reason
        .as_deref()
        .is_some_and(|s| s.contains("false")));
    assert_eq!(fs::read_to_string(&path).unwrap(), before);
}

#[test]
fn apply_returns_err_on_invalid_toml() {
    let temp = TempDir::new().unwrap();
    let path = temp.path().join("config.toml");
    fs::write(&path, "[features\nbroken").unwrap();
    assert!(apply_codex_hooks_flag(&path).is_err());
}

#[test]
fn apply_preserves_comments_when_writing() {
    let temp = TempDir::new().unwrap();
    let path = temp.path().join("config.toml");
    let before = "# user note\n[model]\nname = \"gpt-5\"\n";
    fs::write(&path, before).unwrap();
    let outcome = apply_codex_hooks_flag(&path).unwrap();
    assert!(outcome.applied);
    let after = fs::read_to_string(&path).unwrap();
    assert!(after.contains("# user note"), "lost comment: {after}");
    assert!(after.contains("name = \"gpt-5\""));
    assert!(after.contains("codex_hooks = true"));
}

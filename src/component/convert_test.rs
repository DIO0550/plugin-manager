//! Tests for component/convert module

use super::*;
use crate::error::PlmError;
use std::fs;
use tempfile::TempDir;

/// テスト用の ClaudeCode コマンドコンテンツ
fn sample_claude_code_content() -> &'static str {
    r#"---
name: commit
description: Generate a commit message
allowed-tools: Bash(git:*), Read
model: sonnet
---

Please generate a commit message for $ARGUMENTS.
"#
}

/// テスト用の Copilot プロンプトコンテンツ
fn sample_copilot_content() -> &'static str {
    r#"---
name: commit
description: Generate a commit message
tools: ['githubRepo', 'codebase']
model: GPT-4o
hint: Enter commit description
---

Please generate a commit message for ${arguments}.
"#
}

/// テスト用の Codex プロンプトコンテンツ
fn sample_codex_content() -> &'static str {
    r#"---
description: Generate a commit message
---

Please generate a commit message.
"#
}

#[test]
fn test_command_format_display() {
    assert_eq!(format!("{}", CommandFormat::ClaudeCode), "ClaudeCode");
    assert_eq!(format!("{}", CommandFormat::Copilot), "Copilot");
    assert_eq!(format!("{}", CommandFormat::Codex), "Codex");
}

#[test]
fn test_same_format_copies_file() {
    let tmp = TempDir::new().unwrap();
    let source = tmp.path().join("source.md");
    let dest = tmp.path().join("dest.md");

    fs::write(&source, sample_claude_code_content()).unwrap();

    let result = convert_and_write(
        &source,
        &dest,
        CommandFormat::ClaudeCode,
        CommandFormat::ClaudeCode,
    )
    .unwrap();

    assert!(!result.converted);
    assert_eq!(result.source_format, CommandFormat::ClaudeCode);
    assert_eq!(result.dest_format, CommandFormat::ClaudeCode);
    assert!(dest.exists());
    assert_eq!(
        fs::read_to_string(&dest).unwrap(),
        sample_claude_code_content()
    );
}

#[test]
fn test_claude_code_to_copilot() {
    let tmp = TempDir::new().unwrap();
    let source = tmp.path().join("source.md");
    let dest = tmp.path().join("dest.prompt.md");

    fs::write(&source, sample_claude_code_content()).unwrap();

    let result = convert_and_write(
        &source,
        &dest,
        CommandFormat::ClaudeCode,
        CommandFormat::Copilot,
    )
    .unwrap();

    assert!(result.converted);
    assert_eq!(result.source_format, CommandFormat::ClaudeCode);
    assert_eq!(result.dest_format, CommandFormat::Copilot);
    assert!(dest.exists());

    let content = fs::read_to_string(&dest).unwrap();
    // Copilot 形式の特徴を確認
    assert!(content.contains("tools:"));
    assert!(content.contains("${arguments}"));
    assert!(content.contains("GPT-4o"));
}

#[test]
fn test_claude_code_to_codex() {
    let tmp = TempDir::new().unwrap();
    let source = tmp.path().join("source.md");
    let dest = tmp.path().join("dest.md");

    fs::write(&source, sample_claude_code_content()).unwrap();

    let result = convert_and_write(
        &source,
        &dest,
        CommandFormat::ClaudeCode,
        CommandFormat::Codex,
    )
    .unwrap();

    assert!(result.converted);
    assert_eq!(result.source_format, CommandFormat::ClaudeCode);
    assert_eq!(result.dest_format, CommandFormat::Codex);
    assert!(dest.exists());

    let content = fs::read_to_string(&dest).unwrap();
    // Codex 形式の特徴を確認（description のみ）
    assert!(content.contains("description:"));
    // Codex は tools や model を持たない
    assert!(!content.contains("tools:"));
    assert!(!content.contains("allowed-tools:"));
}

#[test]
fn test_copilot_to_claude_code_unsupported() {
    let tmp = TempDir::new().unwrap();
    let source = tmp.path().join("source.prompt.md");
    let dest = tmp.path().join("dest.md");

    fs::write(&source, sample_copilot_content()).unwrap();

    let result = convert_and_write(
        &source,
        &dest,
        CommandFormat::Copilot,
        CommandFormat::ClaudeCode,
    );

    // 非 ClaudeCode からの変換はサポートされない
    assert!(result.is_err());
    match result.unwrap_err() {
        PlmError::UnsupportedConversion { from, to } => {
            assert_eq!(from, "Copilot");
            assert_eq!(to, "ClaudeCode");
        }
        e => panic!("Expected UnsupportedConversion, got {:?}", e),
    }
    // 出力ファイルは作成されない
    assert!(!dest.exists());
}

#[test]
fn test_codex_to_claude_code_unsupported() {
    let tmp = TempDir::new().unwrap();
    let source = tmp.path().join("source.md");
    let dest = tmp.path().join("dest.md");

    fs::write(&source, sample_codex_content()).unwrap();

    let result = convert_and_write(
        &source,
        &dest,
        CommandFormat::Codex,
        CommandFormat::ClaudeCode,
    );

    // 非 ClaudeCode からの変換はサポートされない
    assert!(result.is_err());
    match result.unwrap_err() {
        PlmError::UnsupportedConversion { from, to } => {
            assert_eq!(from, "Codex");
            assert_eq!(to, "ClaudeCode");
        }
        e => panic!("Expected UnsupportedConversion, got {:?}", e),
    }
    // 出力ファイルは作成されない
    assert!(!dest.exists());
}

#[test]
fn test_copilot_to_codex_unsupported() {
    let tmp = TempDir::new().unwrap();
    let source = tmp.path().join("source.prompt.md");
    let dest = tmp.path().join("dest.md");

    fs::write(&source, sample_copilot_content()).unwrap();

    let result = convert_and_write(&source, &dest, CommandFormat::Copilot, CommandFormat::Codex);

    // 非 ClaudeCode からの変換はサポートされない
    assert!(result.is_err());
    match result.unwrap_err() {
        PlmError::UnsupportedConversion { from, to } => {
            assert_eq!(from, "Copilot");
            assert_eq!(to, "Codex");
        }
        e => panic!("Expected UnsupportedConversion, got {:?}", e),
    }
    // 出力ファイルは作成されない
    assert!(!dest.exists());
}

#[test]
fn test_codex_to_copilot_unsupported() {
    let tmp = TempDir::new().unwrap();
    let source = tmp.path().join("source.md");
    let dest = tmp.path().join("dest.prompt.md");

    fs::write(&source, sample_codex_content()).unwrap();

    let result = convert_and_write(&source, &dest, CommandFormat::Codex, CommandFormat::Copilot);

    // 非 ClaudeCode からの変換はサポートされない
    assert!(result.is_err());
    match result.unwrap_err() {
        PlmError::UnsupportedConversion { from, to } => {
            assert_eq!(from, "Codex");
            assert_eq!(to, "Copilot");
        }
        e => panic!("Expected UnsupportedConversion, got {:?}", e),
    }
    // 出力ファイルは作成されない
    assert!(!dest.exists());
}

#[test]
fn test_creates_parent_directories() {
    let tmp = TempDir::new().unwrap();
    let source = tmp.path().join("source.md");
    let dest = tmp.path().join("nested").join("dir").join("dest.md");

    fs::write(&source, sample_claude_code_content()).unwrap();

    let result = convert_and_write(
        &source,
        &dest,
        CommandFormat::ClaudeCode,
        CommandFormat::ClaudeCode,
    )
    .unwrap();

    assert!(!result.converted);
    assert!(dest.exists());
}

#[test]
fn test_parse_error_returns_error() {
    let tmp = TempDir::new().unwrap();
    let source = tmp.path().join("source.md");
    let dest = tmp.path().join("dest.prompt.md");

    // 不正な frontmatter
    fs::write(
        &source,
        r#"---
invalid: [unclosed array
---

body
"#,
    )
    .unwrap();

    let result = convert_and_write(
        &source,
        &dest,
        CommandFormat::ClaudeCode,
        CommandFormat::Copilot,
    );

    assert!(result.is_err());
    // 出力ファイルは作成されない
    assert!(!dest.exists());
}

#[test]
fn test_source_not_found_returns_error() {
    let tmp = TempDir::new().unwrap();
    let source = tmp.path().join("nonexistent.md");
    let dest = tmp.path().join("dest.md");

    let result = convert_and_write(
        &source,
        &dest,
        CommandFormat::ClaudeCode,
        CommandFormat::Copilot,
    );

    assert!(result.is_err());
}

#[test]
fn test_atomic_write_no_partial_file_on_success() {
    let tmp = TempDir::new().unwrap();
    let source = tmp.path().join("source.md");
    let dest = tmp.path().join("dest.prompt.md");
    let tmp_file = tmp.path().join("dest.prompt.md.tmp");

    fs::write(&source, sample_claude_code_content()).unwrap();

    convert_and_write(
        &source,
        &dest,
        CommandFormat::ClaudeCode,
        CommandFormat::Copilot,
    )
    .unwrap();

    // 一時ファイルが残っていない
    assert!(!tmp_file.exists());
    // 出力ファイルが存在する
    assert!(dest.exists());
}

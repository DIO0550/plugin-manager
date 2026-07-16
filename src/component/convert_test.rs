//! Tests for component/convert module

use super::*;
use crate::error::PlmError;
use crate::target::TargetKind;
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
fn test_agent_format_display() {
    assert_eq!(format!("{}", AgentFormat::ClaudeCode), "ClaudeCode");
    assert_eq!(format!("{}", AgentFormat::Copilot), "Copilot");
    assert_eq!(format!("{}", AgentFormat::Codex), "Codex");
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

// =============================================================================
// Agent 変換テスト
// =============================================================================

/// テスト用の ClaudeCode Agent コンテンツ
fn sample_claude_code_agent_content() -> &'static str {
    r#"---
name: code-review
description: Review code for best practices
tools: Read, Write, Bash
model: sonnet
---

You are a code review agent. Please review the code for best practices.
"#
}

/// テスト用の Copilot Agent コンテンツ（変換元として使用）
fn sample_copilot_agent_content() -> &'static str {
    r#"---
name: code-review
description: Review code for best practices
tools:
  - codebase
model: GPT-4o
target: vscode
---

You are a code review agent.
"#
}

#[test]
fn test_agent_same_format_copies_file() {
    let tmp = TempDir::new().unwrap();
    let source = tmp.path().join("source.md");
    let dest = tmp.path().join("dest.md");

    fs::write(&source, sample_claude_code_agent_content()).unwrap();

    let result = convert_agent_and_write(
        &source,
        &dest,
        AgentFormat::ClaudeCode,
        AgentFormat::ClaudeCode,
    )
    .unwrap();

    assert!(!result.converted);
    assert_eq!(result.source_format, AgentFormat::ClaudeCode);
    assert_eq!(result.dest_format, AgentFormat::ClaudeCode);
    assert!(dest.exists());
    assert_eq!(
        fs::read_to_string(&dest).unwrap(),
        sample_claude_code_agent_content()
    );
}

#[test]
fn test_agent_claude_code_to_copilot() {
    let tmp = TempDir::new().unwrap();
    let source = tmp.path().join("source.md");
    let dest = tmp.path().join("dest.agent.md");

    fs::write(&source, sample_claude_code_agent_content()).unwrap();

    let result = convert_agent_and_write(
        &source,
        &dest,
        AgentFormat::ClaudeCode,
        AgentFormat::Copilot,
    )
    .unwrap();

    assert!(result.converted);
    assert_eq!(result.source_format, AgentFormat::ClaudeCode);
    assert_eq!(result.dest_format, AgentFormat::Copilot);
    assert!(dest.exists());

    let content = fs::read_to_string(&dest).unwrap();
    // Copilot Agent 形式の特徴を確認
    assert!(content.contains("tools:"));
    assert!(content.contains("codebase")); // tools 変換
    assert!(content.contains("GPT-4o")); // model 変換
    assert!(content.contains("target: vscode")); // target 追加
}

#[test]
fn test_agent_claude_code_to_codex() {
    let tmp = TempDir::new().unwrap();
    let source = tmp.path().join("source.md");
    let dest = tmp.path().join("dest.agent.md");

    fs::write(&source, sample_claude_code_agent_content()).unwrap();

    let result =
        convert_agent_and_write(&source, &dest, AgentFormat::ClaudeCode, AgentFormat::Codex)
            .unwrap();

    assert!(result.converted);
    assert_eq!(result.source_format, AgentFormat::ClaudeCode);
    assert_eq!(result.dest_format, AgentFormat::Codex);
    assert!(dest.exists());

    let content = fs::read_to_string(&dest).unwrap();
    // Codex Agent 形式の特徴を確認（description と body のみ、name は frontmatter に含まない）
    assert!(content.contains("description:"));
    // Codex は name, tools, model を frontmatter に持たない
    assert!(!content.contains("name:"));
    assert!(!content.contains("tools:"));
    assert!(!content.contains("model:"));
}

#[test]
fn test_agent_copilot_to_claude_code_unsupported() {
    let tmp = TempDir::new().unwrap();
    let source = tmp.path().join("source.agent.md");
    let dest = tmp.path().join("dest.md");

    fs::write(&source, sample_copilot_agent_content()).unwrap();

    let result = convert_agent_and_write(
        &source,
        &dest,
        AgentFormat::Copilot,
        AgentFormat::ClaudeCode,
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
fn test_agent_copilot_to_codex_unsupported() {
    let tmp = TempDir::new().unwrap();
    let source = tmp.path().join("source.agent.md");
    let dest = tmp.path().join("dest.agent.md");

    fs::write(&source, sample_copilot_agent_content()).unwrap();

    let result = convert_agent_and_write(&source, &dest, AgentFormat::Copilot, AgentFormat::Codex);

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
fn test_agent_creates_parent_directories() {
    let tmp = TempDir::new().unwrap();
    let source = tmp.path().join("source.md");
    let dest = tmp.path().join("nested").join("dir").join("dest.md");

    fs::write(&source, sample_claude_code_agent_content()).unwrap();

    let result = convert_agent_and_write(
        &source,
        &dest,
        AgentFormat::ClaudeCode,
        AgentFormat::ClaudeCode,
    )
    .unwrap();

    assert!(!result.converted);
    assert!(dest.exists());
}

// ========================================
// Skill frontmatter stripping tests
// ========================================

#[test]
fn test_skill_allowed_fields_codex_keeps_name_description_metadata() {
    let allowed = skill_allowed_fields(TargetKind::Codex).unwrap();
    assert!(allowed.contains(&"name"));
    assert!(allowed.contains(&"description"));
    assert!(allowed.contains(&"metadata"));
    assert!(!allowed.contains(&"allowed-tools"));
}

#[test]
fn test_skill_allowed_fields_gemini_keeps_only_name_description() {
    let allowed = skill_allowed_fields(TargetKind::GeminiCli).unwrap();
    assert!(allowed.contains(&"name"));
    assert!(allowed.contains(&"description"));
    // Gemini CLI は metadata 非対応
    assert!(!allowed.contains(&"metadata"));
    assert!(!allowed.contains(&"allowed-tools"));
}

#[test]
fn test_skill_allowed_fields_unrestricted_targets() {
    assert!(skill_allowed_fields(TargetKind::Copilot).is_none());
    assert!(skill_allowed_fields(TargetKind::Antigravity).is_none());
    assert!(skill_allowed_fields(TargetKind::Cursor).is_none());
}

#[test]
fn test_strip_skill_frontmatter_crlf_preserves_body() {
    // CRLF 入力でも本文（閉じ fence 以降）をバイト単位でそのまま保持する。
    // frontmatter 行の改行は LF に正規化されるが、本文はオフセットずれなく無傷で残る。
    let content =
        "---\r\nname: demo\r\ndescription: hi\r\nallowed-tools: Read\r\n---\r\n\r\nline1\r\nline2\r\n";
    let allowed = ["name", "description"];

    let result = strip_skill_frontmatter_fields(content, &allowed);

    // 非対応フィールドは除去される
    assert!(!result.contains("allowed-tools"));
    // 本文（閉じ fence の改行以降）は CRLF のままバイト単位で保持される
    // = 先頭欠落や改行混入がない
    assert!(result.ends_with("\r\nline1\r\nline2\r\n"));
    assert_eq!(
        result,
        "---\nname: demo\ndescription: hi\n---\n\r\nline1\r\nline2\r\n"
    );
}

#[test]
fn test_strip_skill_frontmatter_removes_unsupported_fields() {
    let content = "---\nname: demo\ndescription: hello\ndisable-model-invocation: true\nallowed-tools: Bash(ls *), Read\n---\n\n# Body\n";
    let allowed = ["name", "description", "metadata"];

    let result = strip_skill_frontmatter_fields(content, &allowed);

    assert!(result.contains("name: demo"));
    assert!(result.contains("description: hello"));
    assert!(!result.contains("disable-model-invocation"));
    assert!(!result.contains("allowed-tools"));
    // 本文は保持される
    assert!(result.ends_with("\n# Body\n"));
}

#[test]
fn test_strip_skill_frontmatter_keeps_metadata_with_nested_keys() {
    let content =
        "---\nname: demo\ndescription: hi\nmetadata:\n  short-description: PDF utils\n  category: docs\nargument-hint: [a] [b]\n---\nbody\n";
    let allowed = ["name", "description", "metadata"];

    let result = strip_skill_frontmatter_fields(content, &allowed);

    assert!(result.contains("metadata:"));
    assert!(result.contains("  short-description: PDF utils"));
    assert!(result.contains("  category: docs"));
    assert!(!result.contains("argument-hint"));
}

#[test]
fn test_strip_skill_frontmatter_drops_invalid_yaml_bracket_value() {
    // `argument-hint: [threshold] [min-lines]` は YAML フローシーケンスとして壊れるが、
    // 行ベース除去なので影響を受けずに取り除ける。除去後は YAML として valid になる。
    let content = "---\nname: similarity\ndescription: dup detection\ndisable-model-invocation: true\nallowed-tools: Bash(similarity-ts *)\nargument-hint: [threshold] [min-lines]\n---\nbody\n";
    let allowed = ["name", "description", "metadata"];

    let result = strip_skill_frontmatter_fields(content, &allowed);

    assert!(!result.contains("argument-hint"));
    assert!(!result.contains("allowed-tools"));
    // 除去後の frontmatter は serde_yaml でパースできる
    let fm = result
        .strip_prefix("---\n")
        .and_then(|s| s.split("\n---").next())
        .unwrap();
    let parsed: serde_yaml::Value = serde_yaml::from_str(fm).unwrap();
    assert_eq!(parsed["name"], serde_yaml::Value::from("similarity"));
}

#[test]
fn test_strip_skill_frontmatter_without_frontmatter_is_unchanged() {
    let content = "# Just a body\n\nno frontmatter here\n";
    let result = strip_skill_frontmatter_fields(content, &["name"]);
    assert_eq!(result, content);
}

#[test]
fn test_strip_skill_frontmatter_without_closing_fence_is_unchanged() {
    let content = "---\nname: demo\ndescription: hi\n\n# body but no closing fence\n";
    let result = strip_skill_frontmatter_fields(content, &["name", "description"]);
    assert_eq!(result, content);
}

#[test]
fn test_strip_skill_frontmatter_all_supported_is_unchanged() {
    let content = "---\nname: demo\ndescription: hi\n---\nbody\n";
    let result = strip_skill_frontmatter_fields(content, &["name", "description", "metadata"]);
    assert_eq!(result, content);
}

#[test]
fn test_strip_skill_frontmatter_preserves_body_with_fence_lines() {
    // 本文中に `---` が含まれていても、最初の閉じ fence 以降はそのまま保持される
    let content = "---\nname: demo\ndescription: hi\nallowed-tools: Read\n---\n\nbody line\n\n---\n\nmore body\n";
    let result = strip_skill_frontmatter_fields(content, &["name", "description"]);
    assert!(!result.contains("allowed-tools"));
    assert!(result.ends_with("\n\nbody line\n\n---\n\nmore body\n"));
}

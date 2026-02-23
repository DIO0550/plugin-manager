//! components モジュールの単体テスト

use super::*;
use std::fs;
use std::path::Path;
use tempfile::TempDir;

#[test]
fn test_list_skill_names_extracts_dirs_with_skill_md() {
    let temp_dir = TempDir::new().unwrap();
    let skills_dir = temp_dir.path();

    // SKILL.md を持つディレクトリ
    let skill1 = skills_dir.join("skill1");
    fs::create_dir(&skill1).unwrap();
    fs::write(skill1.join("SKILL.md"), "# Skill 1").unwrap();

    let skill2 = skills_dir.join("skill2");
    fs::create_dir(&skill2).unwrap();
    fs::write(skill2.join("SKILL.md"), "# Skill 2").unwrap();

    // SKILL.md を持たないディレクトリ
    let no_skill = skills_dir.join("no_skill");
    fs::create_dir(&no_skill).unwrap();

    let mut names = list_skill_names(skills_dir);
    names.sort();

    assert_eq!(names, vec!["skill1", "skill2"]);
}

#[test]
fn test_list_skill_names_excludes_dirs_without_skill_md() {
    let temp_dir = TempDir::new().unwrap();
    let skills_dir = temp_dir.path();

    // SKILL.md を持たないディレクトリのみ
    fs::create_dir(skills_dir.join("dir1")).unwrap();
    fs::create_dir(skills_dir.join("dir2")).unwrap();

    let names = list_skill_names(skills_dir);
    assert!(names.is_empty());
}

#[test]
fn test_list_skill_names_returns_empty_for_nonexistent_dir() {
    let temp_dir = TempDir::new().unwrap();
    let nonexistent = temp_dir.path().join("nonexistent");

    let names = list_skill_names(&nonexistent);
    assert!(names.is_empty());
}

#[test]
fn test_list_skill_names_returns_empty_for_file() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("file.txt");
    fs::write(&file_path, "content").unwrap();

    let names = list_skill_names(&file_path);
    assert!(names.is_empty());
}

// =========================================================================
// list_agent_names tests
// =========================================================================

#[test]
fn test_list_agent_names_single_file() {
    let temp_dir = TempDir::new().unwrap();
    // 単一ファイルの場合は file_stem を使用するため、.agent 部分が残る
    // これは既存動作を維持するための仕様
    let agent_file = temp_dir.path().join("my-agent.agent.md");
    fs::write(&agent_file, "# Agent").unwrap();

    let names = list_agent_names(&agent_file);
    assert_eq!(names, vec!["my-agent.agent"]);
}

#[test]
fn test_list_agent_names_single_file_md_only() {
    let temp_dir = TempDir::new().unwrap();
    let agent_file = temp_dir.path().join("my-agent.md");
    fs::write(&agent_file, "# Agent").unwrap();

    let names = list_agent_names(&agent_file);
    assert_eq!(names, vec!["my-agent"]);
}

#[test]
fn test_list_agent_names_directory_with_agent_suffix() {
    let temp_dir = TempDir::new().unwrap();
    let agents_dir = temp_dir.path();

    fs::write(agents_dir.join("agent1.agent.md"), "# Agent 1").unwrap();
    fs::write(agents_dir.join("agent2.agent.md"), "# Agent 2").unwrap();

    let mut names = list_agent_names(agents_dir);
    names.sort();

    assert_eq!(names, vec!["agent1", "agent2"]);
}

#[test]
fn test_list_agent_names_directory_with_md_fallback() {
    let temp_dir = TempDir::new().unwrap();
    let agents_dir = temp_dir.path();

    fs::write(agents_dir.join("agent1.agent.md"), "# Agent 1").unwrap();
    fs::write(agents_dir.join("agent2.md"), "# Agent 2").unwrap();

    let mut names = list_agent_names(agents_dir);
    names.sort();

    assert_eq!(names, vec!["agent1", "agent2"]);
}

#[test]
fn test_list_agent_names_excludes_non_md() {
    let temp_dir = TempDir::new().unwrap();
    let agents_dir = temp_dir.path();

    fs::write(agents_dir.join("agent.agent.md"), "# Agent").unwrap();
    fs::write(agents_dir.join("readme.txt"), "Text file").unwrap();

    let names = list_agent_names(agents_dir);
    assert_eq!(names, vec!["agent"]);
}

#[test]
fn test_list_agent_names_returns_empty_for_nonexistent() {
    let temp_dir = TempDir::new().unwrap();
    let nonexistent = temp_dir.path().join("nonexistent");

    let names = list_agent_names(&nonexistent);
    assert!(names.is_empty());
}

// =========================================================================
// list_command_names tests
// =========================================================================

#[test]
fn test_list_command_names_with_prompt_suffix() {
    let temp_dir = TempDir::new().unwrap();
    let commands_dir = temp_dir.path();

    fs::write(commands_dir.join("cmd1.prompt.md"), "# Command 1").unwrap();
    fs::write(commands_dir.join("cmd2.prompt.md"), "# Command 2").unwrap();

    let mut names = list_command_names(commands_dir);
    names.sort();

    assert_eq!(names, vec!["cmd1", "cmd2"]);
}

#[test]
fn test_list_command_names_with_md_fallback() {
    let temp_dir = TempDir::new().unwrap();
    let commands_dir = temp_dir.path();

    fs::write(commands_dir.join("cmd1.prompt.md"), "# Command 1").unwrap();
    fs::write(commands_dir.join("cmd2.md"), "# Command 2").unwrap();

    let mut names = list_command_names(commands_dir);
    names.sort();

    assert_eq!(names, vec!["cmd1", "cmd2"]);
}

#[test]
fn test_list_command_names_returns_empty_for_nonexistent() {
    let temp_dir = TempDir::new().unwrap();
    let nonexistent = temp_dir.path().join("nonexistent");

    let names = list_command_names(&nonexistent);
    assert!(names.is_empty());
}

// =========================================================================
// list_hook_names tests
// =========================================================================

#[test]
fn test_list_hook_names_removes_extension() {
    let temp_dir = TempDir::new().unwrap();
    let hooks_dir = temp_dir.path();

    fs::write(hooks_dir.join("pre-commit.sh"), "#!/bin/bash").unwrap();
    fs::write(hooks_dir.join("post-build.py"), "#!/usr/bin/env python").unwrap();

    let mut names = list_hook_names(hooks_dir);
    names.sort();

    assert_eq!(names, vec!["post-build", "pre-commit"]);
}

#[test]
fn test_list_hook_names_handles_multiple_dots() {
    let temp_dir = TempDir::new().unwrap();
    let hooks_dir = temp_dir.path();

    fs::write(hooks_dir.join("pre.commit.hook.sh"), "#!/bin/bash").unwrap();

    let names = list_hook_names(hooks_dir);
    assert_eq!(names, vec!["pre.commit.hook"]);
}

#[test]
fn test_list_hook_names_handles_no_extension() {
    let temp_dir = TempDir::new().unwrap();
    let hooks_dir = temp_dir.path();

    fs::write(hooks_dir.join("pre-commit"), "#!/bin/bash").unwrap();

    let names = list_hook_names(hooks_dir);
    assert_eq!(names, vec!["pre-commit"]);
}

#[test]
fn test_list_hook_names_returns_empty_for_nonexistent() {
    let temp_dir = TempDir::new().unwrap();
    let nonexistent = temp_dir.path().join("nonexistent");

    let names = list_hook_names(&nonexistent);
    assert!(names.is_empty());
}

// =========================================================================
// list_markdown_names tests
// =========================================================================

#[test]
fn test_list_markdown_names_extracts_md_files() {
    let temp_dir = TempDir::new().unwrap();
    let dir = temp_dir.path();

    fs::write(dir.join("readme.md"), "# README").unwrap();
    fs::write(dir.join("guide.md"), "# Guide").unwrap();
    fs::write(dir.join("script.sh"), "#!/bin/bash").unwrap();

    let mut names = list_markdown_names(dir);
    names.sort();

    assert_eq!(names, vec!["guide", "readme"]);
}

#[test]
fn test_list_markdown_names_excludes_non_md() {
    let temp_dir = TempDir::new().unwrap();
    let dir = temp_dir.path();

    fs::write(dir.join("file.txt"), "text").unwrap();
    fs::write(dir.join("file.rs"), "fn main() {}").unwrap();

    let names = list_markdown_names(dir);
    assert!(names.is_empty());
}

#[test]
fn test_list_markdown_names_returns_empty_for_nonexistent() {
    let temp_dir = TempDir::new().unwrap();
    let nonexistent = temp_dir.path().join("nonexistent");

    let names = list_markdown_names(&nonexistent);
    assert!(names.is_empty());
}

// =========================================================================
// file_stem_name tests
// =========================================================================

#[test]
fn test_file_stem_name_removes_extension() {
    let path = Path::new("/path/to/file.md");
    assert_eq!(file_stem_name(path), Some("file".to_string()));
}

#[test]
fn test_file_stem_name_handles_multiple_dots() {
    let path = Path::new("/path/to/file.agent.md");
    // file_stem returns "file.agent" (removes only last extension)
    assert_eq!(file_stem_name(path), Some("file.agent".to_string()));
}

#[test]
fn test_file_stem_name_handles_no_extension() {
    let path = Path::new("/path/to/file");
    assert_eq!(file_stem_name(path), Some("file".to_string()));
}

// =========================================================================
// 境界値テスト: list_skill_names
// =========================================================================

#[test]
fn test_list_skill_names_lowercase_skill_md() {
    // skill.md（小文字）は SKILL.md として認識されない
    let temp_dir = TempDir::new().unwrap();
    let skills_dir = temp_dir.path();

    let skill = skills_dir.join("skill1");
    fs::create_dir(&skill).unwrap();
    fs::write(skill.join("skill.md"), "# Skill").unwrap(); // 小文字

    let names = list_skill_names(skills_dir);
    assert!(names.is_empty());
}

#[test]
fn test_list_skill_names_mixed_case_skill_md() {
    // Skill.md（混在）は SKILL.md として認識されない
    let temp_dir = TempDir::new().unwrap();
    let skills_dir = temp_dir.path();

    let skill = skills_dir.join("skill1");
    fs::create_dir(&skill).unwrap();
    fs::write(skill.join("Skill.md"), "# Skill").unwrap();

    let names = list_skill_names(skills_dir);
    assert!(names.is_empty());
}

#[test]
fn test_list_skill_names_ignores_files() {
    // ファイルは無視される（ディレクトリのみ）
    let temp_dir = TempDir::new().unwrap();
    let skills_dir = temp_dir.path();

    fs::write(skills_dir.join("SKILL.md"), "# Skill").unwrap();

    let names = list_skill_names(skills_dir);
    assert!(names.is_empty());
}

#[test]
fn test_list_skill_names_skill_md_uppercase_extension_rejected() {
    // "SKILL.MD"（拡張子大文字）は認識されない
    let temp_dir = TempDir::new().unwrap();
    let skills_dir = temp_dir.path();

    let skill = skills_dir.join("skill1");
    fs::create_dir(&skill).unwrap();
    fs::write(skill.join("SKILL.MD"), "# Skill").unwrap();

    let names = list_skill_names(skills_dir);
    assert!(names.is_empty());
}

#[test]
fn test_list_skill_names_empty_subdir_not_detected() {
    // 空のサブディレクトリはスキルとして検出されない
    let temp_dir = TempDir::new().unwrap();
    let skills_dir = temp_dir.path();

    fs::create_dir(skills_dir.join("empty_skill")).unwrap();

    let names = list_skill_names(skills_dir);
    assert!(names.is_empty());
}

// =========================================================================
// 境界値テスト: list_agent_names
// =========================================================================

#[test]
fn test_list_agent_names_duplicate_names() {
    // .agent.md と .md が同名で共存する場合
    let temp_dir = TempDir::new().unwrap();
    let agents_dir = temp_dir.path();

    fs::write(agents_dir.join("test.agent.md"), "# Agent").unwrap();
    fs::write(agents_dir.join("test.md"), "# Agent").unwrap();

    let mut names = list_agent_names(agents_dir);
    names.sort();

    // 両方とも "test" として抽出される（重複）
    assert_eq!(names, vec!["test", "test"]);
}

#[test]
fn test_list_agent_names_uppercase_md() {
    // 拡張子大文字 (.MD) は認識されない
    let temp_dir = TempDir::new().unwrap();
    let agents_dir = temp_dir.path();

    fs::write(agents_dir.join("test.agent.MD"), "# Agent").unwrap();

    let names = list_agent_names(agents_dir);
    assert!(names.is_empty());
}

#[test]
fn test_list_agent_names_hidden_file() {
    // 隠しファイル (.hidden.md) も通常通り処理される
    let temp_dir = TempDir::new().unwrap();
    let agents_dir = temp_dir.path();

    fs::write(agents_dir.join(".hidden.md"), "# Hidden Agent").unwrap();

    let names = list_agent_names(agents_dir);
    assert_eq!(names, vec![".hidden"]);
}

#[test]
fn test_list_agent_names_ignores_directories() {
    // ディレクトリは無視される
    let temp_dir = TempDir::new().unwrap();
    let agents_dir = temp_dir.path();

    let subdir = agents_dir.join("subdir.agent.md");
    fs::create_dir(&subdir).unwrap();

    let names = list_agent_names(agents_dir);
    assert!(names.is_empty());
}

// =========================================================================
// 境界値テスト: list_command_names
// =========================================================================

#[test]
fn test_list_command_names_duplicate_names() {
    // .prompt.md と .md が同名で共存する場合
    let temp_dir = TempDir::new().unwrap();
    let commands_dir = temp_dir.path();

    fs::write(commands_dir.join("cmd.prompt.md"), "# Command").unwrap();
    fs::write(commands_dir.join("cmd.md"), "# Command").unwrap();

    let mut names = list_command_names(commands_dir);
    names.sort();

    assert_eq!(names, vec!["cmd", "cmd"]);
}

#[test]
fn test_list_command_names_uppercase_extension() {
    // .PROMPT.MD は認識されない
    let temp_dir = TempDir::new().unwrap();
    let commands_dir = temp_dir.path();

    fs::write(commands_dir.join("test.PROMPT.MD"), "# Command").unwrap();

    let names = list_command_names(commands_dir);
    assert!(names.is_empty());
}

// =========================================================================
// 境界値テスト: list_hook_names
// =========================================================================

#[test]
fn test_list_hook_names_dot_file() {
    // .env のようなドットファイルは拡張子除去で空名になる
    let temp_dir = TempDir::new().unwrap();
    let hooks_dir = temp_dir.path();

    fs::write(hooks_dir.join(".env"), "KEY=VALUE").unwrap();

    let names = list_hook_names(hooks_dir);
    // rsplit_once('.') で ("", "env") となり、空文字が返る
    assert_eq!(names, vec![""]);
}

#[test]
fn test_list_hook_names_trailing_dot() {
    // 末尾ドット (file.) の扱い
    let temp_dir = TempDir::new().unwrap();
    let hooks_dir = temp_dir.path();

    fs::write(hooks_dir.join("file."), "content").unwrap();

    let names = list_hook_names(hooks_dir);
    // rsplit_once('.') で ("file", "") となる
    assert_eq!(names, vec!["file"]);
}

#[test]
fn test_list_hook_names_only_extension() {
    // .gitignore のような純粋なドットファイル
    let temp_dir = TempDir::new().unwrap();
    let hooks_dir = temp_dir.path();

    fs::write(hooks_dir.join(".gitignore"), "*.log").unwrap();

    let names = list_hook_names(hooks_dir);
    assert_eq!(names, vec![""]);
}

#[test]
fn test_list_hook_names_hidden_with_extension() {
    // .hidden.sh のような隠しファイル
    let temp_dir = TempDir::new().unwrap();
    let hooks_dir = temp_dir.path();

    fs::write(hooks_dir.join(".hidden.sh"), "#!/bin/bash").unwrap();

    let names = list_hook_names(hooks_dir);
    assert_eq!(names, vec![".hidden"]);
}

// =========================================================================
// 境界値テスト: list_markdown_names
// =========================================================================

#[test]
fn test_list_markdown_names_uppercase_md() {
    // .MD は認識されない
    let temp_dir = TempDir::new().unwrap();
    let dir = temp_dir.path();

    fs::write(dir.join("README.MD"), "# Readme").unwrap();

    let names = list_markdown_names(dir);
    assert!(names.is_empty());
}

#[test]
fn test_list_markdown_names_mixed_case() {
    // .Md は認識されない
    let temp_dir = TempDir::new().unwrap();
    let dir = temp_dir.path();

    fs::write(dir.join("readme.Md"), "# Readme").unwrap();

    let names = list_markdown_names(dir);
    assert!(names.is_empty());
}

#[test]
fn test_list_markdown_names_hidden_md() {
    // 隠しMarkdownファイル
    let temp_dir = TempDir::new().unwrap();
    let dir = temp_dir.path();

    fs::write(dir.join(".hidden.md"), "# Hidden").unwrap();

    let names = list_markdown_names(dir);
    assert_eq!(names, vec![".hidden"]);
}

// =========================================================================
// 境界値テスト: file_stem_name
// =========================================================================

#[test]
fn test_file_stem_name_dot_file() {
    // .gitignore の file_stem は ".gitignore"（隠しファイルとして扱われ、拡張子なし）
    let path = Path::new(".gitignore");
    // Rust の file_stem は先頭ドット付きのファイルを拡張子なしとして扱う
    assert_eq!(file_stem_name(path), Some(".gitignore".to_string()));
}

#[test]
fn test_file_stem_name_double_extension() {
    // .tar.gz のような二重拡張子
    let path = Path::new("archive.tar.gz");
    // file_stem は "archive.tar" を返す
    assert_eq!(file_stem_name(path), Some("archive.tar".to_string()));
}

#[test]
fn test_file_stem_name_trailing_dot() {
    // 末尾ドット
    let path = Path::new("file.");
    assert_eq!(file_stem_name(path), Some("file".to_string()));
}

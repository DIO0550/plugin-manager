//! components モジュールの単体テスト

use super::*;
use std::fs;
use std::path::Path;
use tempfile::TempDir;

fn names(items: Vec<(String, PathBuf)>) -> Vec<String> {
    items.into_iter().map(|(name, _)| name).collect()
}

#[test]
fn test_list_skill_names_extracts_dirs_with_skill_md() {
    let temp_dir = TempDir::new().unwrap();
    let skills_dir = temp_dir.path();

    let skill1 = skills_dir.join("skill1");
    fs::create_dir(&skill1).unwrap();
    fs::write(skill1.join("SKILL.md"), "# Skill 1").unwrap();

    let skill2 = skills_dir.join("skill2");
    fs::create_dir(&skill2).unwrap();
    fs::write(skill2.join("SKILL.md"), "# Skill 2").unwrap();

    let no_skill = skills_dir.join("no_skill");
    fs::create_dir(&no_skill).unwrap();

    let mut result = names(list_skill_names(skills_dir));
    result.sort();

    assert_eq!(result, vec!["skill1", "skill2"]);
}

#[test]
fn test_list_skill_names_excludes_dirs_without_skill_md() {
    let temp_dir = TempDir::new().unwrap();
    let skills_dir = temp_dir.path();

    fs::create_dir(skills_dir.join("dir1")).unwrap();
    fs::create_dir(skills_dir.join("dir2")).unwrap();

    assert!(list_skill_names(skills_dir).is_empty());
}

#[test]
fn test_list_skill_names_returns_empty_for_nonexistent_dir() {
    let temp_dir = TempDir::new().unwrap();
    let nonexistent = temp_dir.path().join("nonexistent");

    assert!(list_skill_names(&nonexistent).is_empty());
}

#[test]
fn test_list_skill_names_returns_empty_for_file() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("file.txt");
    fs::write(&file_path, "content").unwrap();

    assert!(list_skill_names(&file_path).is_empty());
}

// =========================================================================
// list_agent_names tests
// =========================================================================

#[test]
fn test_list_agent_names_single_file() {
    let temp_dir = TempDir::new().unwrap();
    let agent_file = temp_dir.path().join("my-agent.agent.md");
    fs::write(&agent_file, "# Agent").unwrap();

    assert_eq!(names(list_agent_names(&agent_file)), vec!["my-agent"]);
}

#[test]
fn test_list_agent_names_single_file_md_only() {
    let temp_dir = TempDir::new().unwrap();
    let agent_file = temp_dir.path().join("my-agent.md");
    fs::write(&agent_file, "# Agent").unwrap();

    assert_eq!(names(list_agent_names(&agent_file)), vec!["my-agent"]);
}

#[test]
fn test_list_agent_names_directory_with_agent_suffix() {
    let temp_dir = TempDir::new().unwrap();
    let agents_dir = temp_dir.path();

    fs::write(agents_dir.join("agent1.agent.md"), "# Agent 1").unwrap();
    fs::write(agents_dir.join("agent2.agent.md"), "# Agent 2").unwrap();

    let mut result = names(list_agent_names(agents_dir));
    result.sort();

    assert_eq!(result, vec!["agent1", "agent2"]);
}

#[test]
fn test_list_agent_names_directory_with_md_fallback() {
    let temp_dir = TempDir::new().unwrap();
    let agents_dir = temp_dir.path();

    fs::write(agents_dir.join("agent1.agent.md"), "# Agent 1").unwrap();
    fs::write(agents_dir.join("agent2.md"), "# Agent 2").unwrap();

    let mut result = names(list_agent_names(agents_dir));
    result.sort();

    assert_eq!(result, vec!["agent1", "agent2"]);
}

#[test]
fn test_list_agent_names_excludes_non_md() {
    let temp_dir = TempDir::new().unwrap();
    let agents_dir = temp_dir.path();

    fs::write(agents_dir.join("agent.agent.md"), "# Agent").unwrap();
    fs::write(agents_dir.join("readme.txt"), "Text file").unwrap();

    assert_eq!(names(list_agent_names(agents_dir)), vec!["agent"]);
}

#[test]
fn test_list_agent_names_returns_empty_for_nonexistent() {
    let temp_dir = TempDir::new().unwrap();
    let nonexistent = temp_dir.path().join("nonexistent");

    assert!(list_agent_names(&nonexistent).is_empty());
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

    let mut result = names(list_command_names(commands_dir));
    result.sort();

    assert_eq!(result, vec!["cmd1", "cmd2"]);
}

#[test]
fn test_list_command_names_with_md_fallback() {
    let temp_dir = TempDir::new().unwrap();
    let commands_dir = temp_dir.path();

    fs::write(commands_dir.join("cmd1.prompt.md"), "# Command 1").unwrap();
    fs::write(commands_dir.join("cmd2.md"), "# Command 2").unwrap();

    let mut result = names(list_command_names(commands_dir));
    result.sort();

    assert_eq!(result, vec!["cmd1", "cmd2"]);
}

#[test]
fn test_list_command_names_returns_empty_for_nonexistent() {
    let temp_dir = TempDir::new().unwrap();
    let nonexistent = temp_dir.path().join("nonexistent");

    assert!(list_command_names(&nonexistent).is_empty());
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

    let mut result = names(list_hook_names(hooks_dir));
    result.sort();

    assert_eq!(result, vec!["post-build", "pre-commit"]);
}

#[test]
fn test_list_hook_names_handles_multiple_dots() {
    let temp_dir = TempDir::new().unwrap();
    let hooks_dir = temp_dir.path();

    fs::write(hooks_dir.join("pre.commit.hook.sh"), "#!/bin/bash").unwrap();

    assert_eq!(names(list_hook_names(hooks_dir)), vec!["pre.commit.hook"]);
}

#[test]
fn test_list_hook_names_handles_no_extension() {
    let temp_dir = TempDir::new().unwrap();
    let hooks_dir = temp_dir.path();

    fs::write(hooks_dir.join("pre-commit"), "#!/bin/bash").unwrap();

    assert_eq!(names(list_hook_names(hooks_dir)), vec!["pre-commit"]);
}

#[test]
fn test_list_hook_names_returns_empty_for_nonexistent() {
    let temp_dir = TempDir::new().unwrap();
    let nonexistent = temp_dir.path().join("nonexistent");

    assert!(list_hook_names(&nonexistent).is_empty());
}

#[test]
fn test_list_hook_names_skips_dotfiles_with_empty_stem() {
    let temp_dir = TempDir::new().unwrap();
    let hooks_dir = temp_dir.path();

    // ドットファイルは rsplit_once('.') が ("", ext) を返すため stem が空文字
    // になる。flatten_name 後も `<plugin>_` という不完全な名前になり
    // validate_path_segment で失敗するため、ここで除外する。
    fs::write(hooks_dir.join(".gitignore"), "*.tmp").unwrap();
    fs::write(hooks_dir.join(".env"), "FOO=bar").unwrap();
    fs::write(hooks_dir.join("pre-commit.sh"), "#!/bin/sh").unwrap();

    assert_eq!(names(list_hook_names(hooks_dir)), vec!["pre-commit"]);
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

    let mut result = names(list_markdown_names(dir));
    result.sort();

    assert_eq!(result, vec!["guide", "readme"]);
}

#[test]
fn test_list_markdown_names_excludes_non_md() {
    let temp_dir = TempDir::new().unwrap();
    let dir = temp_dir.path();

    fs::write(dir.join("file.txt"), "text").unwrap();
    fs::write(dir.join("file.rs"), "fn main() {}").unwrap();

    assert!(list_markdown_names(dir).is_empty());
}

#[test]
fn test_list_markdown_names_returns_empty_for_nonexistent() {
    let temp_dir = TempDir::new().unwrap();
    let nonexistent = temp_dir.path().join("nonexistent");

    assert!(list_markdown_names(&nonexistent).is_empty());
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
    let temp_dir = TempDir::new().unwrap();
    let skills_dir = temp_dir.path();

    let skill = skills_dir.join("skill1");
    fs::create_dir(&skill).unwrap();
    fs::write(skill.join("skill.md"), "# Skill").unwrap();

    assert!(list_skill_names(skills_dir).is_empty());
}

#[test]
fn test_list_skill_names_mixed_case_skill_md() {
    let temp_dir = TempDir::new().unwrap();
    let skills_dir = temp_dir.path();

    let skill = skills_dir.join("skill1");
    fs::create_dir(&skill).unwrap();
    fs::write(skill.join("Skill.md"), "# Skill").unwrap();

    assert!(list_skill_names(skills_dir).is_empty());
}

#[test]
fn test_list_skill_names_ignores_files() {
    let temp_dir = TempDir::new().unwrap();
    let skills_dir = temp_dir.path();

    fs::write(skills_dir.join("SKILL.md"), "# Skill").unwrap();

    assert!(list_skill_names(skills_dir).is_empty());
}

#[test]
fn test_list_skill_names_skill_md_uppercase_extension_rejected() {
    let temp_dir = TempDir::new().unwrap();
    let skills_dir = temp_dir.path();

    let skill = skills_dir.join("skill1");
    fs::create_dir(&skill).unwrap();
    fs::write(skill.join("SKILL.MD"), "# Skill").unwrap();

    assert!(list_skill_names(skills_dir).is_empty());
}

#[test]
fn test_list_skill_names_empty_subdir_not_detected() {
    let temp_dir = TempDir::new().unwrap();
    let skills_dir = temp_dir.path();

    fs::create_dir(skills_dir.join("empty_skill")).unwrap();

    assert!(list_skill_names(skills_dir).is_empty());
}

#[test]
fn test_list_skill_names_skill_md_directory_rejected() {
    let temp_dir = TempDir::new().unwrap();
    let skills_dir = temp_dir.path();

    let skill = skills_dir.join("skill1");
    fs::create_dir(&skill).unwrap();
    fs::create_dir(skill.join("SKILL.md")).unwrap();

    assert!(list_skill_names(skills_dir).is_empty());
}

// =========================================================================
// 境界値テスト: list_agent_names
// =========================================================================

#[test]
fn test_list_agent_names_duplicate_names() {
    let temp_dir = TempDir::new().unwrap();
    let agents_dir = temp_dir.path();

    fs::write(agents_dir.join("test.agent.md"), "# Agent").unwrap();
    fs::write(agents_dir.join("test.md"), "# Agent").unwrap();

    let mut result = names(list_agent_names(agents_dir));
    result.sort();

    assert_eq!(result, vec!["test", "test"]);
}

#[test]
fn test_list_agent_names_uppercase_md() {
    let temp_dir = TempDir::new().unwrap();
    let agents_dir = temp_dir.path();

    fs::write(agents_dir.join("test.agent.MD"), "# Agent").unwrap();

    assert!(list_agent_names(agents_dir).is_empty());
}

#[test]
fn test_list_agent_names_hidden_file() {
    let temp_dir = TempDir::new().unwrap();
    let agents_dir = temp_dir.path();

    fs::write(agents_dir.join(".hidden.md"), "# Hidden Agent").unwrap();

    assert_eq!(names(list_agent_names(agents_dir)), vec![".hidden"]);
}

#[test]
fn test_list_agent_names_ignores_directories() {
    let temp_dir = TempDir::new().unwrap();
    let agents_dir = temp_dir.path();

    let subdir = agents_dir.join("subdir.agent.md");
    fs::create_dir(&subdir).unwrap();

    assert!(list_agent_names(agents_dir).is_empty());
}

// =========================================================================
// 境界値テスト: list_command_names
// =========================================================================

#[test]
fn test_list_command_names_duplicate_names() {
    let temp_dir = TempDir::new().unwrap();
    let commands_dir = temp_dir.path();

    fs::write(commands_dir.join("cmd.prompt.md"), "# Command").unwrap();
    fs::write(commands_dir.join("cmd.md"), "# Command").unwrap();

    let mut result = names(list_command_names(commands_dir));
    result.sort();

    assert_eq!(result, vec!["cmd", "cmd"]);
}

#[test]
fn test_list_command_names_uppercase_extension() {
    let temp_dir = TempDir::new().unwrap();
    let commands_dir = temp_dir.path();

    fs::write(commands_dir.join("test.PROMPT.MD"), "# Command").unwrap();

    assert!(list_command_names(commands_dir).is_empty());
}

// =========================================================================
// 境界値テスト: list_hook_names
// =========================================================================

#[test]
fn test_list_hook_names_dot_file() {
    let temp_dir = TempDir::new().unwrap();
    let hooks_dir = temp_dir.path();

    fs::write(hooks_dir.join(".env"), "KEY=VALUE").unwrap();

    // ドットファイルは stem が空文字になるため Hook 候補から除外する。
    assert!(list_hook_names(hooks_dir).is_empty());
}

#[test]
fn test_list_hook_names_trailing_dot() {
    let temp_dir = TempDir::new().unwrap();
    let hooks_dir = temp_dir.path();

    fs::write(hooks_dir.join("file."), "content").unwrap();

    assert_eq!(names(list_hook_names(hooks_dir)), vec!["file"]);
}

#[test]
fn test_list_hook_names_only_extension() {
    let temp_dir = TempDir::new().unwrap();
    let hooks_dir = temp_dir.path();

    fs::write(hooks_dir.join(".gitignore"), "*.log").unwrap();

    // 拡張子のみのドットファイルも stem が空のため除外する。
    assert!(list_hook_names(hooks_dir).is_empty());
}

#[test]
fn test_list_hook_names_hidden_with_extension() {
    let temp_dir = TempDir::new().unwrap();
    let hooks_dir = temp_dir.path();

    fs::write(hooks_dir.join(".hidden.sh"), "#!/bin/bash").unwrap();

    assert_eq!(names(list_hook_names(hooks_dir)), vec![".hidden"]);
}

// =========================================================================
// 境界値テスト: list_markdown_names
// =========================================================================

#[test]
fn test_list_markdown_names_uppercase_md() {
    let temp_dir = TempDir::new().unwrap();
    let dir = temp_dir.path();

    fs::write(dir.join("README.MD"), "# Readme").unwrap();

    assert!(list_markdown_names(dir).is_empty());
}

#[test]
fn test_list_markdown_names_mixed_case() {
    let temp_dir = TempDir::new().unwrap();
    let dir = temp_dir.path();

    fs::write(dir.join("readme.Md"), "# Readme").unwrap();

    assert!(list_markdown_names(dir).is_empty());
}

#[test]
fn test_list_markdown_names_hidden_md() {
    let temp_dir = TempDir::new().unwrap();
    let dir = temp_dir.path();

    fs::write(dir.join(".hidden.md"), "# Hidden").unwrap();

    assert_eq!(names(list_markdown_names(dir)), vec![".hidden"]);
}

// =========================================================================
// 境界値テスト: file_stem_name
// =========================================================================

#[test]
fn test_file_stem_name_dot_file() {
    let path = Path::new(".gitignore");
    assert_eq!(file_stem_name(path), Some(".gitignore".to_string()));
}

#[test]
fn test_file_stem_name_double_extension() {
    let path = Path::new("archive.tar.gz");
    assert_eq!(file_stem_name(path), Some("archive.tar".to_string()));
}

#[test]
fn test_file_stem_name_trailing_dot() {
    let path = Path::new("file.");
    assert_eq!(file_stem_name(path), Some("file".to_string()));
}

// =========================================================================
// 再帰スキャン: list_skill_names
// =========================================================================

#[test]
fn test_list_skill_names_one_level_nested() {
    let temp_dir = TempDir::new().unwrap();
    let skills_dir = temp_dir.path();

    let nested = skills_dir.join("bar").join("foo");
    fs::create_dir_all(&nested).unwrap();
    fs::write(nested.join("SKILL.md"), "# Skill").unwrap();

    let result = list_skill_names(skills_dir);
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].0, "foo");
    assert_eq!(result[0].1, nested);
}

#[test]
fn test_list_skill_names_multi_level_nested() {
    let temp_dir = TempDir::new().unwrap();
    let skills_dir = temp_dir.path();

    let nested = skills_dir.join("a").join("b").join("baz");
    fs::create_dir_all(&nested).unwrap();
    fs::write(nested.join("SKILL.md"), "# Skill").unwrap();

    let result = list_skill_names(skills_dir);
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].0, "baz");
    assert_eq!(result[0].1, nested);
}

#[test]
fn test_list_skill_names_does_not_descend_into_skill() {
    let temp_dir = TempDir::new().unwrap();
    let skills_dir = temp_dir.path();

    // skill1 が SKILL.md を持つ
    let skill1 = skills_dir.join("skill1");
    fs::create_dir(&skill1).unwrap();
    fs::write(skill1.join("SKILL.md"), "# Skill").unwrap();

    // skill1/assets/inner にも SKILL.md がある（誤検出してはいけない）
    let inner = skill1.join("assets").join("inner");
    fs::create_dir_all(&inner).unwrap();
    fs::write(inner.join("SKILL.md"), "# Inner").unwrap();

    let result = list_skill_names(skills_dir);
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].0, "skill1");
}

#[test]
fn test_list_skill_names_mixed_flat_and_nested() {
    let temp_dir = TempDir::new().unwrap();
    let skills_dir = temp_dir.path();

    // 直下: skills/foo/SKILL.md
    let flat = skills_dir.join("foo");
    fs::create_dir(&flat).unwrap();
    fs::write(flat.join("SKILL.md"), "# foo").unwrap();

    // ネスト: skills/bar/baz/SKILL.md
    let nested = skills_dir.join("bar").join("baz");
    fs::create_dir_all(&nested).unwrap();
    fs::write(nested.join("SKILL.md"), "# baz").unwrap();

    let mut result = names(list_skill_names(skills_dir));
    result.sort();
    assert_eq!(result, vec!["baz", "foo"]);
}

#[test]
fn test_list_skill_names_duplicate_basename_in_nested() {
    let temp_dir = TempDir::new().unwrap();
    let skills_dir = temp_dir.path();

    let a = skills_dir.join("a").join("foo");
    let b = skills_dir.join("b").join("foo");
    fs::create_dir_all(&a).unwrap();
    fs::create_dir_all(&b).unwrap();
    fs::write(a.join("SKILL.md"), "# a/foo").unwrap();
    fs::write(b.join("SKILL.md"), "# b/foo").unwrap();

    // scan 層では衝突検出しない（呼び出し側 build_components の責務）
    let mut result = names(list_skill_names(skills_dir));
    result.sort();
    assert_eq!(result, vec!["foo", "foo"]);
}

// =========================================================================
// 再帰スキャン: list_agent_names
// =========================================================================

#[test]
fn test_list_agent_names_one_level_nested() {
    let temp_dir = TempDir::new().unwrap();
    let agents_dir = temp_dir.path();

    let nested = agents_dir.join("bar");
    fs::create_dir(&nested).unwrap();
    fs::write(nested.join("foo.agent.md"), "# Agent").unwrap();

    let result = list_agent_names(agents_dir);
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].0, "foo");
    assert_eq!(result[0].1, nested.join("foo.agent.md"));
}

#[test]
fn test_list_agent_names_multi_level_nested() {
    let temp_dir = TempDir::new().unwrap();
    let agents_dir = temp_dir.path();

    let nested = agents_dir.join("a").join("b");
    fs::create_dir_all(&nested).unwrap();
    fs::write(nested.join("foo.agent.md"), "# Agent").unwrap();

    let result = list_agent_names(agents_dir);
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].0, "foo");
    assert_eq!(result[0].1, nested.join("foo.agent.md"));
}

#[test]
fn test_list_agent_names_recursive_skips_non_md() {
    let temp_dir = TempDir::new().unwrap();
    let agents_dir = temp_dir.path();

    let nested = agents_dir.join("nested");
    fs::create_dir(&nested).unwrap();
    fs::write(nested.join("notes.txt"), "skip").unwrap();
    fs::write(nested.join("foo.agent.md"), "# Agent").unwrap();

    let result = names(list_agent_names(agents_dir));
    assert_eq!(result, vec!["foo"]);
}

// =========================================================================
// 再帰スキャン: list_command_names
// =========================================================================

#[test]
fn test_list_command_names_one_level_nested() {
    let temp_dir = TempDir::new().unwrap();
    let commands_dir = temp_dir.path();

    let nested = commands_dir.join("bar");
    fs::create_dir(&nested).unwrap();
    fs::write(nested.join("foo.prompt.md"), "# Cmd").unwrap();

    let result = list_command_names(commands_dir);
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].0, "foo");
    assert_eq!(result[0].1, nested.join("foo.prompt.md"));
}

// =========================================================================
// 再帰スキャン: symlink ループ防止
// =========================================================================

#[cfg(unix)]
#[test]
fn test_list_skill_names_does_not_follow_symlinks() {
    use std::os::unix::fs::symlink;

    let temp_dir = TempDir::new().unwrap();
    let skills_dir = temp_dir.path();

    // 通常の skill
    let skill1 = skills_dir.join("skill1");
    fs::create_dir(&skill1).unwrap();
    fs::write(skill1.join("SKILL.md"), "# Skill1").unwrap();

    // 自分自身を指す symlink — 辿ったら無限ループ
    symlink(skills_dir, skills_dir.join("loop")).unwrap();

    let mut result = names(list_skill_names(skills_dir));
    result.sort();
    assert_eq!(result, vec!["skill1"]);
}

#[cfg(unix)]
#[test]
fn test_list_agent_names_does_not_follow_symlinks() {
    use std::os::unix::fs::symlink;

    let temp_dir = TempDir::new().unwrap();
    let agents_dir = temp_dir.path();

    fs::write(agents_dir.join("a.agent.md"), "# Agent").unwrap();
    symlink(agents_dir, agents_dir.join("loop")).unwrap();

    let result = names(list_agent_names(agents_dir));
    assert_eq!(result, vec!["a"]);
}

#[test]
fn test_list_command_names_multi_level_nested() {
    let temp_dir = TempDir::new().unwrap();
    let commands_dir = temp_dir.path();

    let nested = commands_dir.join("a").join("b");
    fs::create_dir_all(&nested).unwrap();
    fs::write(nested.join("foo.prompt.md"), "# Cmd").unwrap();

    let result = list_command_names(commands_dir);
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].0, "foo");
}

//! コンポーネントスキャン共通関数
//!
//! SKILL.md を持つサブディレクトリの列挙など、複数箇所で使用される
//! スキャンロジックを提供する。

use crate::path_ext::PathExt;
use std::path::Path;

// ============================================================================
// 定数
// ============================================================================

/// スキルマニフェストファイル名
pub(crate) const SKILL_MANIFEST: &str = "SKILL.md";

/// コンポーネント検出用のファイルサフィックス
pub(crate) const AGENT_SUFFIX: &str = ".agent.md";
pub(crate) const PROMPT_SUFFIX: &str = ".prompt.md";
pub(crate) const MARKDOWN_SUFFIX: &str = ".md";

/// デフォルトのコンポーネントディレクトリパス
pub(crate) const DEFAULT_SKILLS_DIR: &str = "skills";
pub(crate) const DEFAULT_AGENTS_DIR: &str = "agents";
pub(crate) const DEFAULT_COMMANDS_DIR: &str = "commands";
pub(crate) const DEFAULT_HOOKS_DIR: &str = "hooks";

/// デフォルトのインストラクション設定
pub(crate) const DEFAULT_INSTRUCTIONS_FILE: &str = "instructions.md";
pub(crate) const DEFAULT_INSTRUCTIONS_DIR: &str = "instructions";

// ============================================================================
// 関数
// ============================================================================

/// スキル名一覧を取得
///
/// 指定されたディレクトリ配下で `SKILL.md` を持つサブディレクトリを列挙し、
/// そのディレクトリ名を返す。
///
/// # Arguments
/// * `skills_dir` - スキルディレクトリのパス
///
/// # Returns
/// スキル名（ディレクトリ名）の一覧。順序は保証されない。
///
/// # Behavior
/// - `skills_dir` がディレクトリでない場合は空配列を返す
/// - サブディレクトリのうち `SKILL.md` が存在するものだけを抽出
/// - UTF-8 変換不可のディレクトリ名は除外
pub(crate) fn list_skill_names(skills_dir: &Path) -> Vec<String> {
    if !skills_dir.is_dir() {
        return Vec::new();
    }

    skills_dir
        .read_dir_entries()
        .into_iter()
        .filter(|path| path.is_dir() && path.join(SKILL_MANIFEST).exists())
        .filter_map(|path| path.file_name().and_then(|n| n.to_str()).map(String::from))
        .collect()
}

/// エージェント名一覧を取得
///
/// 指定されたパスが単一ファイルの場合はそのファイル名（拡張子除去）を返す。
/// ディレクトリの場合は .agent.md / .md ファイルを列挙する。
///
/// # Arguments
/// * `agents_path` - エージェントファイルまたはディレクトリのパス
///
/// # Returns
/// エージェント名の一覧。単一ファイルの場合は None を返す可能性がある
/// （呼び出し側でフォールバック処理が必要）。
pub(crate) fn list_agent_names(agents_path: &Path) -> Vec<String> {
    // 単一ファイルの場合
    if agents_path.is_file() {
        return file_stem_name(agents_path)
            .map(|name| vec![name])
            .unwrap_or_default();
    }

    if !agents_path.is_dir() {
        return Vec::new();
    }

    agents_path
        .read_dir_entries()
        .into_iter()
        .filter(|path| path.is_file())
        .filter_map(|path| {
            let name = path.file_name()?.to_str()?;
            if name.ends_with(AGENT_SUFFIX) {
                Some(name.trim_end_matches(AGENT_SUFFIX).to_string())
            } else if name.ends_with(MARKDOWN_SUFFIX) {
                Some(name.trim_end_matches(MARKDOWN_SUFFIX).to_string())
            } else {
                None
            }
        })
        .collect()
}

/// コマンド名一覧を取得
///
/// ディレクトリ内の .prompt.md / .md ファイルを列挙する。
///
/// # Arguments
/// * `commands_dir` - コマンドディレクトリのパス
///
/// # Returns
/// コマンド名の一覧。
pub(crate) fn list_command_names(commands_dir: &Path) -> Vec<String> {
    if !commands_dir.is_dir() {
        return Vec::new();
    }

    commands_dir
        .read_dir_entries()
        .into_iter()
        .filter(|path| path.is_file())
        .filter_map(|path| {
            let name = path.file_name()?.to_str()?;
            if name.ends_with(PROMPT_SUFFIX) {
                Some(name.trim_end_matches(PROMPT_SUFFIX).to_string())
            } else if name.ends_with(MARKDOWN_SUFFIX) {
                Some(name.trim_end_matches(MARKDOWN_SUFFIX).to_string())
            } else {
                None
            }
        })
        .collect()
}

/// フック名一覧を取得
///
/// ディレクトリ内のファイルを列挙し、拡張子を除去した名前を返す。
///
/// # Arguments
/// * `hooks_dir` - フックディレクトリのパス
///
/// # Returns
/// フック名の一覧。
pub(crate) fn list_hook_names(hooks_dir: &Path) -> Vec<String> {
    if !hooks_dir.is_dir() {
        return Vec::new();
    }

    hooks_dir
        .read_dir_entries()
        .into_iter()
        .filter(|path| path.is_file())
        .filter_map(|path| {
            let name = path.file_name()?.to_str()?;
            // 拡張子を除去（複数ドットの場合は最後のみ）
            let hook_name = name
                .rsplit_once('.')
                .map(|(n, _)| n.to_string())
                .unwrap_or_else(|| name.to_string());
            Some(hook_name)
        })
        .collect()
}

/// Markdown ファイル名一覧を取得
///
/// ディレクトリ内の .md ファイルを列挙し、拡張子を除去した名前を返す。
///
/// # Arguments
/// * `dir` - 対象ディレクトリのパス
///
/// # Returns
/// Markdown ファイル名（拡張子除去済み）の一覧。
pub(crate) fn list_markdown_names(dir: &Path) -> Vec<String> {
    if !dir.is_dir() {
        return Vec::new();
    }

    dir.read_dir_entries()
        .into_iter()
        .filter(|path| path.is_file())
        .filter_map(|path| {
            let name = path.file_name()?.to_str()?;
            if name.ends_with(MARKDOWN_SUFFIX) {
                Some(name.trim_end_matches(MARKDOWN_SUFFIX).to_string())
            } else {
                None
            }
        })
        .collect()
}

/// パスからファイル名（拡張子除去）を取得
///
/// # Arguments
/// * `path` - ファイルパス
///
/// # Returns
/// ファイル名（拡張子除去済み）。UTF-8変換不可の場合は None。
pub(crate) fn file_stem_name(path: &Path) -> Option<String> {
    path.file_stem().and_then(|s| s.to_str()).map(String::from)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
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
}

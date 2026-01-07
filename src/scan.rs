//! コンポーネントスキャン共通関数
//!
//! SKILL.md を持つサブディレクトリの列挙など、複数箇所で使用される
//! スキャンロジックを提供する。
//!
//! ## 主要API
//!
//! - [`scan_components`]: プラグインディレクトリからコンポーネントをスキャン（統一API）
//! - [`ComponentScan`]: スキャン結果（名前のみ）
//!
//! ## 低レベル関数
//!
//! - [`list_skill_names`], [`list_agent_names`], etc.: 個別コンポーネントのスキャン

use crate::path_ext::PathExt;
use crate::plugin::PluginManifest;
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
// 統一スキャンAPI
// ============================================================================

/// コンポーネントスキャン結果
///
/// プラグインディレクトリから検出されたコンポーネントの名前一覧。
/// パスは含まず、名前のみを保持する（パス解決は呼び出し側の責務）。
#[derive(Debug, Clone, Default)]
pub struct ComponentScan {
    /// スキル名一覧
    pub skills: Vec<String>,
    /// エージェント名一覧
    pub agents: Vec<String>,
    /// コマンド名一覧
    pub commands: Vec<String>,
    /// インストラクション名一覧
    pub instructions: Vec<String>,
    /// フック名一覧
    pub hooks: Vec<String>,
}

impl ComponentScan {
    /// コンポーネントの総数を取得
    pub fn total_count(&self) -> usize {
        self.skills.len()
            + self.agents.len()
            + self.commands.len()
            + self.instructions.len()
            + self.hooks.len()
    }

    /// コンポーネントが存在するか
    pub fn has_components(&self) -> bool {
        self.total_count() > 0
    }
}

/// プラグインディレクトリからコンポーネントをスキャン
///
/// マニフェストのカスタムパス定義を尊重し、全コンポーネント種別を一括スキャン。
/// これが唯一の統一スキャンAPIであり、Application層はこれを使用すべき。
///
/// # Arguments
/// * `plugin_path` - プラグインのルートディレクトリ
/// * `manifest` - プラグインマニフェスト
///
/// # Returns
/// 検出されたコンポーネント名の一覧
pub fn scan_components(plugin_path: &Path, manifest: &PluginManifest) -> ComponentScan {
    ComponentScan {
        skills: scan_skills_internal(plugin_path, manifest),
        agents: scan_agents_internal(plugin_path, manifest),
        commands: scan_commands_internal(plugin_path, manifest),
        instructions: scan_instructions_internal(plugin_path, manifest),
        hooks: scan_hooks_internal(plugin_path, manifest),
    }
}

// ============================================================================
// 内部スキャン関数（scan_components から呼ばれる）
// ============================================================================

/// Skills をスキャン（内部用）
fn scan_skills_internal(plugin_path: &Path, manifest: &PluginManifest) -> Vec<String> {
    let skills_dir = manifest.skills_dir(plugin_path);
    list_skill_names(&skills_dir)
}

/// Agents をスキャン（内部用）
fn scan_agents_internal(plugin_path: &Path, manifest: &PluginManifest) -> Vec<String> {
    let agents_path = manifest.agents_dir(plugin_path);
    list_agent_names(&agents_path)
}

/// Commands をスキャン（内部用）
fn scan_commands_internal(plugin_path: &Path, manifest: &PluginManifest) -> Vec<String> {
    let commands_dir = manifest.commands_dir(plugin_path);
    list_command_names(&commands_dir)
}

/// Instructions をスキャン（内部用）
///
/// マニフェスト指定時: ファイルまたはディレクトリを走査。
/// 未指定時: instructions/ を走査 + ルートの AGENTS.md を追加。
fn scan_instructions_internal(plugin_path: &Path, manifest: &PluginManifest) -> Vec<String> {
    if let Some(path_str) = &manifest.instructions {
        // マニフェストで指定された場合
        let path = plugin_path.join(path_str);

        if path.is_file() {
            return file_stem_name(&path)
                .map(|name| vec![name])
                .unwrap_or_default();
        }

        if path.is_dir() {
            return list_markdown_names(&path);
        }

        return Vec::new();
    }

    // デフォルト: instructions/ ディレクトリ + AGENTS.md
    let instructions_dir = manifest.instructions_dir(plugin_path);
    let mut instructions = list_markdown_names(&instructions_dir);

    // ルートの AGENTS.md もチェック
    if plugin_path.join("AGENTS.md").exists() {
        instructions.push("AGENTS".to_string());
    }

    instructions
}

/// Hooks をスキャン（内部用）
fn scan_hooks_internal(plugin_path: &Path, manifest: &PluginManifest) -> Vec<String> {
    let hooks_dir = manifest.hooks_dir(plugin_path);
    list_hook_names(&hooks_dir)
}

// ============================================================================
// 低レベル関数
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
}

#[cfg(test)]
mod proptests {
    use super::*;
    use proptest::prelude::*;
    use std::fs;
    use tempfile::TempDir;

    /// 有効なファイル名に使える文字列（英数字、ハイフン、アンダースコア）
    fn valid_name_strategy() -> impl Strategy<Value = String> {
        "[a-zA-Z][a-zA-Z0-9_-]{0,15}".prop_map(|s| s)
    }

    proptest! {
        /// list_agent_names は .agent.md サフィックスを除去する
        #[test]
        fn prop_list_agent_names_removes_agent_suffix(
            name in valid_name_strategy()
        ) {
            let temp_dir = TempDir::new().unwrap();
            let agents_dir = temp_dir.path();

            let filename = format!("{}.agent.md", name);
            fs::write(agents_dir.join(&filename), "# Agent").unwrap();

            let names = list_agent_names(agents_dir);

            prop_assert_eq!(names.len(), 1);
            prop_assert_eq!(&names[0], &name);
            // サフィックスが残っていないことを確認
            prop_assert!(!names[0].ends_with(".agent.md"));
            prop_assert!(!names[0].ends_with(".md"));
        }

        /// list_agent_names は .md サフィックスを除去する
        #[test]
        fn prop_list_agent_names_removes_md_suffix(
            name in valid_name_strategy()
        ) {
            let temp_dir = TempDir::new().unwrap();
            let agents_dir = temp_dir.path();

            let filename = format!("{}.md", name);
            fs::write(agents_dir.join(&filename), "# Agent").unwrap();

            let names = list_agent_names(agents_dir);

            prop_assert_eq!(names.len(), 1);
            prop_assert_eq!(&names[0], &name);
            prop_assert!(!names[0].ends_with(".md"));
        }

        /// list_command_names は .prompt.md サフィックスを除去する
        #[test]
        fn prop_list_command_names_removes_prompt_suffix(
            name in valid_name_strategy()
        ) {
            let temp_dir = TempDir::new().unwrap();
            let commands_dir = temp_dir.path();

            let filename = format!("{}.prompt.md", name);
            fs::write(commands_dir.join(&filename), "# Command").unwrap();

            let names = list_command_names(commands_dir);

            prop_assert_eq!(names.len(), 1);
            prop_assert_eq!(&names[0], &name);
            prop_assert!(!names[0].ends_with(".prompt.md"));
            prop_assert!(!names[0].ends_with(".md"));
        }

        /// list_hook_names は最後の拡張子のみを除去する
        #[test]
        fn prop_list_hook_names_removes_only_last_extension(
            name in valid_name_strategy(),
            ext in "[a-z]{2,4}"
        ) {
            let temp_dir = TempDir::new().unwrap();
            let hooks_dir = temp_dir.path();

            let filename = format!("{}.{}", name, ext);
            fs::write(hooks_dir.join(&filename), "#!/bin/bash").unwrap();

            let names = list_hook_names(hooks_dir);

            prop_assert_eq!(names.len(), 1);
            prop_assert_eq!(&names[0], &name);
            // 拡張子が除去されていることを確認
            let ext_suffix = format!(".{}", ext);
            prop_assert!(!names[0].ends_with(&ext_suffix));
        }

        /// list_hook_names は複数ドットがあっても最後の拡張子のみ除去
        #[test]
        fn prop_list_hook_names_preserves_multiple_dots(
            base in valid_name_strategy(),
            middle in valid_name_strategy(),
            ext in "[a-z]{2,4}"
        ) {
            let temp_dir = TempDir::new().unwrap();
            let hooks_dir = temp_dir.path();

            // name.middle.ext 形式
            let filename = format!("{}.{}.{}", base, middle, ext);
            fs::write(hooks_dir.join(&filename), "#!/bin/bash").unwrap();

            let names = list_hook_names(hooks_dir);

            prop_assert_eq!(names.len(), 1);
            // base.middle が残る
            let expected = format!("{}.{}", base, middle);
            prop_assert_eq!(&names[0], &expected);
        }

        /// list_markdown_names は .md サフィックスを除去する
        #[test]
        fn prop_list_markdown_names_removes_md_suffix(
            name in valid_name_strategy()
        ) {
            let temp_dir = TempDir::new().unwrap();
            let dir = temp_dir.path();

            let filename = format!("{}.md", name);
            fs::write(dir.join(&filename), "# Markdown").unwrap();

            let names = list_markdown_names(dir);

            prop_assert_eq!(names.len(), 1);
            prop_assert_eq!(&names[0], &name);
            prop_assert!(!names[0].ends_with(".md"));
        }

        /// list_skill_names は SKILL.md を持つディレクトリ名を返す
        #[test]
        fn prop_list_skill_names_returns_dir_names(
            name in valid_name_strategy()
        ) {
            let temp_dir = TempDir::new().unwrap();
            let skills_dir = temp_dir.path();

            let skill_dir = skills_dir.join(&name);
            fs::create_dir(&skill_dir).unwrap();
            fs::write(skill_dir.join("SKILL.md"), "# Skill").unwrap();

            let names = list_skill_names(skills_dir);

            prop_assert_eq!(names.len(), 1);
            prop_assert_eq!(&names[0], &name);
        }
    }
}

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
}

//! components モジュールのプロパティテスト

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

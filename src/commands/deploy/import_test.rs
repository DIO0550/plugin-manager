//! import コマンドのユニットテスト

use super::*;

mod parse_component_path_tests {
    use super::*;
    use crate::component::ComponentKind;

    #[test]
    fn basic_skill_path() {
        let result = parse_component_path("skills/pdf");
        assert!(result.is_ok());
        let (kind, name) = result.unwrap();
        assert_eq!(kind, ComponentKind::Skill);
        assert_eq!(name, "pdf");
    }

    #[test]
    fn uppercase_kind_is_normalized() {
        let result = parse_component_path("SKILLS/pdf");
        assert!(result.is_ok());
        let (kind, name) = result.unwrap();
        assert_eq!(kind, ComponentKind::Skill);
        assert_eq!(name, "pdf");
    }

    #[test]
    fn name_case_is_preserved() {
        let result = parse_component_path("skills/PDF");
        assert!(result.is_ok());
        let (kind, name) = result.unwrap();
        assert_eq!(kind, ComponentKind::Skill);
        assert_eq!(name, "PDF");
    }

    #[test]
    fn trailing_slash_is_removed() {
        let result = parse_component_path("skills/pdf/");
        assert!(result.is_ok());
        let (kind, name) = result.unwrap();
        assert_eq!(kind, ComponentKind::Skill);
        assert_eq!(name, "pdf");
    }

    #[test]
    fn whitespace_is_trimmed() {
        let result = parse_component_path("  skills/pdf  ");
        assert!(result.is_ok());
        let (kind, name) = result.unwrap();
        assert_eq!(kind, ComponentKind::Skill);
        assert_eq!(name, "pdf");
    }

    #[test]
    fn agents_kind() {
        let result = parse_component_path("agents/review");
        assert!(result.is_ok());
        let (kind, name) = result.unwrap();
        assert_eq!(kind, ComponentKind::Agent);
        assert_eq!(name, "review");
    }

    #[test]
    fn commands_kind() {
        let result = parse_component_path("commands/test");
        assert!(result.is_ok());
        let (kind, name) = result.unwrap();
        assert_eq!(kind, ComponentKind::Command);
        assert_eq!(name, "test");
    }

    #[test]
    fn instructions_kind() {
        let result = parse_component_path("instructions/guide");
        assert!(result.is_ok());
        let (kind, name) = result.unwrap();
        assert_eq!(kind, ComponentKind::Instruction);
        assert_eq!(name, "guide");
    }

    #[test]
    fn hooks_kind() {
        let result = parse_component_path("hooks/pre-commit");
        assert!(result.is_ok());
        let (kind, name) = result.unwrap();
        assert_eq!(kind, ComponentKind::Hook);
        assert_eq!(name, "pre-commit");
    }

    #[test]
    fn singular_kind_is_error() {
        let result = parse_component_path("skill/pdf");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("skill"));
    }

    #[test]
    fn no_slash_is_error() {
        let result = parse_component_path("invalid");
        assert!(result.is_err());
    }

    #[test]
    fn empty_name_is_error() {
        let result = parse_component_path("skills/");
        assert!(result.is_err());
    }

    #[test]
    fn nested_path_is_error() {
        let result = parse_component_path("skills/a/b");
        assert!(result.is_err());
    }

    #[test]
    fn consecutive_slashes_is_error() {
        let result = parse_component_path("skills//pdf");
        assert!(result.is_err());
    }

    #[test]
    fn unknown_kind_is_error() {
        let result = parse_component_path("unknown/test");
        assert!(result.is_err());
    }
}

mod filter_components_tests {
    use super::*;
    use crate::component::{Component, ComponentKind};
    use std::path::PathBuf;

    fn make_component(kind: ComponentKind, name: &str) -> Component {
        Component {
            kind,
            name: name.to_string(),
            path: PathBuf::from(format!("{}/{}", kind.plural(), name)),
        }
    }

    fn make_test_components() -> Vec<Component> {
        vec![
            make_component(ComponentKind::Skill, "pdf"),
            make_component(ComponentKind::Skill, "json"),
            make_component(ComponentKind::Agent, "review"),
            make_component(ComponentKind::Command, "test"),
            make_component(ComponentKind::Hook, "pre-commit"),
        ]
    }

    #[test]
    fn empty_filter_returns_all() {
        let components = make_test_components();
        let (filtered, skipped) = filter_components(components.clone(), &[], &[]);
        assert_eq!(filtered.len(), 5);
        assert!(skipped.is_empty());
    }

    #[test]
    fn filter_by_single_type() {
        let components = make_test_components();
        let (filtered, skipped) = filter_components(components, &[], &[ComponentKind::Skill]);
        assert_eq!(filtered.len(), 2);
        assert!(filtered.iter().all(|c| c.kind == ComponentKind::Skill));
        assert!(skipped.is_empty());
    }

    #[test]
    fn filter_by_multiple_types() {
        let components = make_test_components();
        let (filtered, skipped) = filter_components(
            components,
            &[],
            &[ComponentKind::Skill, ComponentKind::Agent],
        );
        assert_eq!(filtered.len(), 3);
        assert!(skipped.is_empty());
    }

    #[test]
    fn filter_by_component_path() {
        let components = make_test_components();
        let paths = vec![(ComponentKind::Skill, "pdf".to_string())];
        let (filtered, skipped) = filter_components(components, &paths, &[]);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].name, "pdf");
        assert_eq!(filtered[0].kind, ComponentKind::Skill);
        assert!(skipped.is_empty());
    }

    #[test]
    fn filter_by_multiple_component_paths() {
        let components = make_test_components();
        let paths = vec![
            (ComponentKind::Skill, "pdf".to_string()),
            (ComponentKind::Agent, "review".to_string()),
        ];
        let (filtered, skipped) = filter_components(components, &paths, &[]);
        assert_eq!(filtered.len(), 2);
        assert!(skipped.is_empty());
    }

    #[test]
    fn nonexistent_path_is_skipped() {
        let components = make_test_components();
        let paths = vec![
            (ComponentKind::Skill, "pdf".to_string()),
            (ComponentKind::Skill, "nonexistent".to_string()),
        ];
        let (filtered, skipped) = filter_components(components, &paths, &[]);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].name, "pdf");
        assert_eq!(skipped.len(), 1);
        assert!(skipped.contains(&"skills/nonexistent".to_string()));
    }

    #[test]
    fn all_nonexistent_returns_empty() {
        let components = make_test_components();
        let paths = vec![(ComponentKind::Skill, "nonexistent".to_string())];
        let (filtered, skipped) = filter_components(components, &paths, &[]);
        assert!(filtered.is_empty());
        assert_eq!(skipped.len(), 1);
    }

    #[test]
    fn case_sensitive_name_matching() {
        let components = make_test_components();
        let paths = vec![(ComponentKind::Skill, "PDF".to_string())];
        let (filtered, skipped) = filter_components(components, &paths, &[]);
        assert!(filtered.is_empty());
        assert_eq!(skipped.len(), 1);
        assert!(skipped.contains(&"skills/PDF".to_string()));
    }

    #[test]
    fn preserves_input_order() {
        let components = make_test_components();
        let paths = vec![
            (ComponentKind::Agent, "review".to_string()),
            (ComponentKind::Skill, "pdf".to_string()),
        ];
        let (filtered, _) = filter_components(components, &paths, &[]);
        assert_eq!(filtered.len(), 2);
        assert_eq!(filtered[0].kind, ComponentKind::Agent);
        assert_eq!(filtered[0].name, "review");
        assert_eq!(filtered[1].kind, ComponentKind::Skill);
        assert_eq!(filtered[1].name, "pdf");
    }

    #[test]
    fn duplicate_paths_are_deduplicated() {
        let components = make_test_components();
        let paths = vec![
            (ComponentKind::Skill, "pdf".to_string()),
            (ComponentKind::Skill, "pdf".to_string()),
        ];
        let (filtered, _) = filter_components(components, &paths, &[]);
        assert_eq!(filtered.len(), 1);
    }

    #[test]
    fn duplicate_skipped_paths_are_deduplicated() {
        let components = make_test_components();
        let paths = vec![
            (ComponentKind::Skill, "nonexistent".to_string()),
            (ComponentKind::Skill, "nonexistent".to_string()),
        ];
        let (filtered, skipped) = filter_components(components, &paths, &[]);
        assert!(filtered.is_empty());
        assert_eq!(skipped.len(), 1);
    }
}

mod place_components_tests {
    use super::*;
    use crate::component::{Component, ComponentKind};
    use crate::import::ImportRegistry;
    use std::path::PathBuf;
    use tempfile::TempDir;

    fn make_hook(name: &str) -> Component {
        Component {
            kind: ComponentKind::Hook,
            name: name.to_string(),
            path: PathBuf::from(format!("hooks/{}.json", name)),
        }
    }

    #[test]
    fn codex_rejects_multiple_hook_components() {
        let project_dir = TempDir::new().unwrap();
        let plugin_dir = TempDir::new().unwrap();
        let registry_dir = TempDir::new().unwrap();
        let origin = PluginOrigin::from_marketplace("test-marketplace", "test-plugin");
        let ctx = ImportContext {
            origin: &origin,
            scope: Scope::Project,
            project_root: project_dir.path(),
            plugin_root: plugin_dir.path(),
            source_repo: "owner/repo",
            git_ref: "main",
            commit_sha: "abc123",
        };
        let components = vec![make_hook("first"), make_hook("second")];
        let mut registry = ImportRegistry::with_path(registry_dir.path().join("imports.json"));

        let result = place_components(&["codex".to_string()], &components, &ctx, &mut registry);

        assert_eq!(result.unwrap(), (0, 2));
        assert!(!project_dir.path().join(".codex/hooks.json").exists());
    }
}

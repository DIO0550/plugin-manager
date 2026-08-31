//! placement_names 定数の不変条件

use super::*;
use crate::component::ComponentKind;

#[test]
fn all_instruction_filenames_are_unique_and_non_empty() {
    let mut seen = std::collections::HashSet::new();
    for name in ALL_INSTRUCTION_FILENAMES {
        assert!(!name.is_empty());
        assert!(seen.insert(*name), "duplicate instruction filename: {name}");
    }
}

#[test]
fn copilot_command_subdir_differs_from_plural() {
    assert_ne!(COPILOT_COMMAND_SUBDIR, ComponentKind::Command.plural());
    assert_eq!(COPILOT_COMMAND_SUBDIR, "prompts");
}

#[test]
fn antigravity_skills_use_official_roots_and_keep_legacy_roots_distinct() {
    assert_eq!(ANTIGRAVITY_SKILLS_PERSONAL_CHILD, "config");
    assert_eq!(ANTIGRAVITY_SKILLS_PROJECT_SUBDIR, ".agents");
    assert_eq!(ANTIGRAVITY_LEGACY_PERSONAL_CHILD, "antigravity");
    assert_eq!(ANTIGRAVITY_LEGACY_PROJECT_SUBDIR, ".agent");
    assert_ne!(
        ANTIGRAVITY_SKILLS_PERSONAL_CHILD,
        ANTIGRAVITY_LEGACY_PERSONAL_CHILD
    );
    assert_ne!(
        ANTIGRAVITY_SKILLS_PROJECT_SUBDIR,
        ANTIGRAVITY_LEGACY_PROJECT_SUBDIR
    );
}

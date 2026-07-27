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
fn attached_exact_exclusions_cover_reserved_names() {
    assert!(ATTACHED_EXACT_EXCLUSIONS.contains(&PLUGIN_MANIFEST_DIR));
    assert!(ATTACHED_EXACT_EXCLUSIONS.contains(&PLUGIN_MANIFEST_FILE));
    assert!(ATTACHED_EXACT_EXCLUSIONS.contains(&PLM_META_FILE));
    assert!(ATTACHED_RESOURCES_MAX_BYTES > 0);
}

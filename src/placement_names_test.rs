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
fn default_placement_subdir_matches_plural_except_instruction() {
    for kind in ComponentKind::all() {
        match kind {
            ComponentKind::Instruction => {
                assert_eq!(default_placement_subdir(*kind), None);
            }
            other => {
                assert_eq!(default_placement_subdir(*other), Some(other.plural()));
            }
        }
    }
}

#[test]
fn copilot_command_subdir_differs_from_plural() {
    assert_ne!(COPILOT_COMMAND_SUBDIR, ComponentKind::Command.plural());
    assert_eq!(COPILOT_COMMAND_SUBDIR, "prompts");
}

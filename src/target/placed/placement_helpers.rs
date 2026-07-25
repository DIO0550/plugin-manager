//! `placement_location` 共通パターンヘルパ

use crate::component::{ComponentKind, PlacementLocation, Scope};
use std::path::Path;

/// Skill: `base/skills/<name>/`
pub(crate) fn skill_dir(base: &Path, name: &str) -> PlacementLocation {
    let subdir = ComponentKind::Skill
        .default_subdir()
        .expect("Skill always has a default subdir");
    PlacementLocation::dir(base.join(subdir).join(name))
}

/// `base/<subdir>/<name><suffix>` ファイル配置。
pub(crate) fn named_file(base: &Path, subdir: &str, name: &str, suffix: &str) -> PlacementLocation {
    PlacementLocation::file(base.join(subdir).join(format!("{name}{suffix}")))
}

/// Agent（Codex / Copilot）: `base/agents/<name>.agent.md`
pub(crate) fn agent_file(base: &Path, name: &str) -> PlacementLocation {
    let suffix = ComponentKind::Agent
        .file_suffix()
        .expect("Agent always has a file suffix");
    let subdir = ComponentKind::Agent
        .default_subdir()
        .expect("Agent always has a default subdir");
    named_file(base, subdir, name, suffix)
}

/// Instruction: Project → `project_root/<filename>`, Personal → `base/<filename>`
pub(crate) fn instruction_file(
    scope: Scope,
    project_root: &Path,
    base: &Path,
    filename: &str,
) -> PlacementLocation {
    match scope {
        Scope::Project => PlacementLocation::file(project_root.join(filename)),
        Scope::Personal => PlacementLocation::file(base.join(filename)),
    }
}

/// Instruction: 常に `base/<filename>`（Copilot Project など）。
pub(crate) fn instruction_under_base(base: &Path, filename: &str) -> PlacementLocation {
    PlacementLocation::file(base.join(filename))
}

#[cfg(test)]
#[path = "placement_helpers_test.rs"]
mod tests;

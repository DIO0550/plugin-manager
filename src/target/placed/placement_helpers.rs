//! `placement_location` 共通パターンヘルパ

use crate::component::{PlacementLocation, Scope};
use std::path::Path;

/// Skill: `base/skills/<name>/`
pub(crate) fn skill_dir(base: &Path, name: &str) -> PlacementLocation {
    PlacementLocation::dir(base.join("skills").join(name))
}

/// `base/<subdir>/<name><suffix>` ファイル配置。
pub(crate) fn named_file(base: &Path, subdir: &str, name: &str, suffix: &str) -> PlacementLocation {
    PlacementLocation::file(base.join(subdir).join(format!("{name}{suffix}")))
}

/// Agent（Codex / Copilot）: `base/agents/<name>.agent.md`
pub(crate) fn agent_file(base: &Path, name: &str) -> PlacementLocation {
    named_file(base, "agents", name, ".agent.md")
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

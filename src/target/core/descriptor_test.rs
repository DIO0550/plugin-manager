use super::*;
use crate::target::env::all_layouts;
use std::path::Path;

#[test]
fn every_layout_is_internally_consistent() {
    for layout in all_layouts() {
        layout.assert_consistent();
        assert_eq!(
            layout.capabilities.derived_supported().as_slice(),
            layout.supported_components(),
            "{} supported_components must be derived from capabilities",
            layout.kind.as_str()
        );
    }
}

#[test]
fn layout_kind_covers_all_target_kinds() {
    let kinds: Vec<_> = all_layouts().into_iter().map(|l| l.kind).collect();
    assert_eq!(
        kinds,
        vec![
            TargetKind::Antigravity,
            TargetKind::Codex,
            TargetKind::Copilot,
            TargetKind::Cursor,
            TargetKind::GeminiCli,
            TargetKind::OpenCode,
        ]
    );
}

#[test]
fn gemini_env_root_and_flattened_skill() {
    let layout = all_layouts()
        .into_iter()
        .find(|l| l.kind == TargetKind::GeminiCli)
        .unwrap();
    let root = Path::new("/proj");
    assert_eq!(layout.env_root(Scope::Project, root), root.join(".gemini"));
    assert!(layout.allows(ComponentKind::Skill, Scope::Personal));
    assert!(!layout.allows(ComponentKind::Agent, Scope::Project));
}

#[test]
fn copilot_instruction_stays_under_env_root() {
    use crate::component::{ComponentRef, PlacementScope, ProjectContext};
    use crate::target::PluginOrigin;

    let layout = all_layouts()
        .into_iter()
        .find(|l| l.kind == TargetKind::Copilot)
        .unwrap();
    let origin = PluginOrigin::from_marketplace("official", "plugin");
    let project_root = Path::new("/proj");
    let ctx = PlacementContext {
        component: ComponentRef::new(ComponentKind::Instruction, "test"),
        origin: &origin,
        scope: PlacementScope::new(Scope::Project),
        project: ProjectContext::new(project_root),
    };
    let loc = layout.placement_location(&ctx).unwrap();
    assert_eq!(
        loc.as_path(),
        Path::new("/proj/.github/copilot-instructions.md")
    );
}

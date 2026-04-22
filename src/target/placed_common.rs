//! `list_placed` の Instruction 分岐で共有されるユーティリティ

use crate::component::{
    ComponentIdentity, ComponentKind, PlacementContext, PlacementScope, ProjectContext, Scope,
};
use crate::target::{PluginOrigin, Target};
use std::path::Path;

/// Instruction コンポーネントが配置済みの場合にファイル名を返す
pub(crate) fn list_instruction(
    target: &dyn Target,
    scope: Scope,
    project_root: &Path,
    instruction_filename: &str,
) -> Vec<String> {
    let dummy_origin = PluginOrigin::from_marketplace("", "");
    let ctx = PlacementContext {
        component: ComponentIdentity::new(ComponentKind::Instruction, ""),
        origin: &dummy_origin,
        scope: PlacementScope::new(scope),
        project: ProjectContext::new(project_root),
    };
    let Some(location) = target.placement_location(&ctx) else {
        return vec![];
    };
    if location.as_path().exists() {
        vec![instruction_filename.to_string()]
    } else {
        vec![]
    }
}

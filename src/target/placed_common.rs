//! `list_placed` の Instruction 分岐で共有されるユーティリティ

use crate::component::{
    ComponentKind, ComponentRef, PlacementContext, PlacementScope, ProjectContext, Scope,
};
use crate::target::{PluginOrigin, Target};
use std::path::Path;

/// Instruction コンポーネントが配置済みの場合にファイル名を返す
///
/// 現状の 4 target はいずれも Instruction の placement_location を
/// origin / component name に依存させていないため、dummy 値を渡しても
/// 支障はない。ただし空文字列だと将来 origin / name を使う target が
/// `<base>//...` のような奇妙なパスを生成する恐れがあるため、
/// 非空の placeholder (`"test"`) で固定する（`Target::supports_scope` の
/// dummy context と揃える方針）。
pub(crate) fn list_instruction(
    target: &dyn Target,
    scope: Scope,
    project_root: &Path,
    instruction_filename: &str,
) -> Vec<String> {
    let dummy_origin = PluginOrigin::from_marketplace("test", "test");
    let ctx = PlacementContext {
        component: ComponentRef::new(ComponentKind::Instruction, "test"),
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

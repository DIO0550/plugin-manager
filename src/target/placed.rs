//! 全ターゲットから配置済みプラグインを収集するユーティリティ
//!
//! Target trait と ComponentKind に依存するため、ドメイン非依存の `src/scan/` ではなく
//! `src/target/` 配下に配置する。

use super::all_targets;
use crate::component::{ComponentKind, Scope};
use crate::scan::list_placed_plugins;
use std::collections::HashSet;
use std::path::Path;

/// 全ターゲットから配置済みプラグインを収集
///
/// 全ターゲット・全コンポーネント種別のデプロイ済みコンポーネントを走査し、
/// プラグインの (marketplace, plugin_name) の集合を返す。
pub(crate) fn list_all_placed(project_root: &Path) -> HashSet<(String, String)> {
    let targets = all_targets();
    let mut all_items = Vec::new();

    for target in &targets {
        for kind in ComponentKind::all() {
            if !target.supports(*kind) {
                continue;
            }
            // エラー時は黙殺（保守的に deployed とみなさない）
            if let Ok(placed) = target.list_placed(*kind, Scope::Project, project_root) {
                all_items.extend(placed);
            }
        }
    }

    list_placed_plugins(&all_items)
}

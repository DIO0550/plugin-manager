//! 全ターゲットから配置済みプラグインを収集するユーティリティ
//!
//! Target trait と ComponentKind に依存するため、ドメイン非依存の `src/scan/` ではなく
//! `src/target/` 配下に配置する。

use crate::component::{ComponentKind, Scope};
use crate::scan::list_placed_components;
use crate::target::all_targets;
use std::collections::HashSet;
use std::path::Path;

/// 全ターゲットから配置済みコンポーネントの `flattened_name` 集合を収集する。
///
/// 全ターゲット・全コンポーネント種別の **Project scope のみ** を走査し、
/// Instruction ファイル名を除いた `flattened_name` の重複なし集合を返す。
/// Personal scope（`~/.codex/`, `~/.copilot/` 等）は走査せず、Personal の
/// 配置検知は `.plm-meta.json` の `statusByTarget` 経由（`is_enabled` の優先
/// パス）に委ねる。フラット 2 階層構造へ移行後は `(marketplace, plugin)`
/// ペアでは識別できないため、戻り値の型を `HashSet<String>` に変更している。
///
/// # Arguments
///
/// * `project_root` - Project root directory used for project-scope lookups.
pub(crate) fn list_all_placed(project_root: &Path) -> HashSet<String> {
    let targets = all_targets();
    let mut all_items = Vec::new();

    for target in &targets {
        for kind in ComponentKind::all() {
            if !target.supports(*kind) {
                continue;
            }
            // Instruction の戻り値は後段の `list_placed_components` で
            // 除外されるため、ファイルシステム探査自体をスキップする。
            if matches!(kind, ComponentKind::Instruction) {
                continue;
            }
            // エラー時は黙殺（保守的に deployed とみなさない）
            if let Ok(placed) = target.list_placed(*kind, Scope::Project, project_root) {
                all_items.extend(placed);
            }
        }
    }

    list_placed_components(&all_items)
}

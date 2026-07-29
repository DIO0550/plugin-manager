//! Plugin 付属リソースの除外集合合成
//!
//! `placement_names` のリテラルと `PluginManifest` 解決パスを合成する。

use crate::component::ComponentKind;
use crate::placement_names::COPILOT_COMMAND_SUBDIR;
use crate::plugin::meta::resolve_manifest_path;
use crate::plugin::PluginManifest;
use crate::scan::{list_plugin_attached_resources, AttachedEntry};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

/// マニフェスト解決済みパスを含む除外相対パス集合を構築する。
pub fn attached_exclusion_paths(manifest: &PluginManifest, plugin_root: &Path) -> HashSet<PathBuf> {
    let mut set = HashSet::new();

    for path in [
        manifest.skills_dir(plugin_root),
        manifest.agents_dir(plugin_root),
        manifest.commands_dir(plugin_root),
        manifest.hooks_dir(plugin_root),
        manifest.instructions_dir(plugin_root),
        manifest.instructions_path(plugin_root),
        plugin_root.join(COPILOT_COMMAND_SUBDIR),
        plugin_root.join(ComponentKind::Instruction.plural()),
    ] {
        if let Ok(rel) = path.strip_prefix(plugin_root) {
            if !rel.as_os_str().is_empty() {
                set.insert(rel.to_path_buf());
            }
        }
    }

    set
}

/// 既定のコンポーネント dir 名だけの除外集合（マニフェスト無しフォールバック用）。
fn default_attached_exclusions() -> HashSet<PathBuf> {
    let mut excluded = HashSet::new();
    for kind in ComponentKind::all() {
        excluded.insert(PathBuf::from(kind.plural()));
    }
    excluded.insert(PathBuf::from(COPILOT_COMMAND_SUBDIR));
    excluded
}

/// プラグインの付属リソースを列挙する（除外合成込み）。
pub fn list_attached_for_plugin(
    manifest: &PluginManifest,
    plugin_root: &Path,
) -> Vec<AttachedEntry> {
    let excluded = attached_exclusion_paths(manifest, plugin_root);
    list_plugin_attached_resources(plugin_root, &excluded)
}

/// プラグインルートから付属リソースを列挙する。
///
/// マニフェストがあれば解決パス込みで除外し、無ければ既定コンポーネント dir 名のみ除外する。
pub fn list_attached_entries(plugin_root: &Path) -> Vec<AttachedEntry> {
    match resolve_manifest_path(plugin_root).and_then(|p| PluginManifest::load(&p).ok()) {
        Some(manifest) => list_attached_for_plugin(&manifest, plugin_root),
        None => list_plugin_attached_resources(plugin_root, &default_attached_exclusions()),
    }
}

#[cfg(test)]
#[path = "attached_test.rs"]
mod tests;

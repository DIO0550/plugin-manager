//! プラグインリソースの除外合成・配置・削除

use crate::component::Scope;
use crate::error::{PlmError, Result};
use crate::fs::FileSystem;
use crate::placement_names::{
    ALL_INSTRUCTION_FILENAMES, CLAUDE_PLUGIN_DIR, PLM_META_FILE, PLUGIN_JSON_FILE,
    PLUGIN_RESOURCES_SUBDIR, PLUGIN_RESOURCE_VCS_NAMES,
};
use crate::plugin::PluginManifest;
use crate::scan::{list_plugin_resources, PluginResourceEntry};
use crate::target::{paths::home_dir, TargetKind};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

/// ターゲット上のプラグインリソースルート: `<base>/plugins/<plugin_name>`
pub fn plugin_resources_root(
    target_kind: TargetKind,
    scope: Scope,
    project_root: &Path,
    plugin_name: &str,
) -> Result<PathBuf> {
    validate_plugin_name(plugin_name)?;
    let base = match scope {
        Scope::Personal => target_kind.personal_base(&home_dir()),
        Scope::Project => target_kind.project_base(project_root),
    };
    Ok(base.join(PLUGIN_RESOURCES_SUBDIR).join(plugin_name))
}

/// manifest + 予約パスから除外絶対パス集合を構築する。
pub fn plugin_resource_exclusion_paths(
    plugin_root: &Path,
    manifest: &PluginManifest,
) -> HashSet<PathBuf> {
    let mut paths = HashSet::new();

    paths.insert(manifest.skills_dir(plugin_root));
    paths.insert(manifest.agents_dir(plugin_root));
    paths.insert(manifest.commands_dir(plugin_root));

    let hooks = manifest.hooks_dir(plugin_root);
    paths.insert(hooks.clone());
    // hooks がファイル宣言のとき親ディレクトリも除外
    if let Some(parent) = hooks.parent() {
        if parent != plugin_root {
            paths.insert(parent.to_path_buf());
        }
    }

    paths.insert(manifest.instructions_path(plugin_root));
    paths.insert(manifest.instructions_dir(plugin_root));

    paths.insert(plugin_root.join(CLAUDE_PLUGIN_DIR));
    paths.insert(plugin_root.join(PLUGIN_JSON_FILE));
    paths.insert(plugin_root.join(PLM_META_FILE));

    for name in ALL_INSTRUCTION_FILENAMES {
        paths.insert(plugin_root.join(name));
    }

    if let Some(ref p) = manifest.mcp_servers {
        paths.insert(plugin_root.join(p));
    }
    if let Some(ref p) = manifest.lsp_servers {
        paths.insert(plugin_root.join(p));
    }

    paths
}

/// VCS 等のトップレベル名除外集合。
pub fn plugin_resource_exclusion_names() -> HashSet<&'static str> {
    PLUGIN_RESOURCE_VCS_NAMES.iter().copied().collect()
}

/// プラグインルートからリソースを列挙する（manifest 境界込み）。
pub fn list_resources_for_plugin(
    plugin_root: &Path,
    manifest: &PluginManifest,
) -> Vec<PluginResourceEntry> {
    let excluded_paths = plugin_resource_exclusion_paths(plugin_root, manifest);
    let excluded_names = plugin_resource_exclusion_names();
    list_plugin_resources(plugin_root, &excluded_paths, &excluded_names)
}

/// プラグインリソース配置のパラメータ。
pub struct DeployPluginResourcesRequest<'a> {
    pub plugin_root: &'a Path,
    pub manifest: &'a PluginManifest,
    pub target_kind: TargetKind,
    pub scope: Scope,
    pub project_root: &'a Path,
}

/// プラグインリソースをターゲットへ構造維持で配置する。
///
/// エントリが空なら既存のリソースルートを削除する（stale 掃除）。
/// 戻り値は配置（または削除）したリソースルート。
pub fn deploy_plugin_resources(
    fs: &dyn FileSystem,
    request: &DeployPluginResourcesRequest<'_>,
) -> Result<Option<PathBuf>> {
    let dest = plugin_resources_root(
        request.target_kind,
        request.scope,
        request.project_root,
        &request.manifest.name,
    )?;
    let entries = list_resources_for_plugin(request.plugin_root, request.manifest);

    if entries.is_empty() {
        if fs.exists(&dest) {
            fs.remove(&dest)?;
        }
        return Ok(None);
    }

    let staging_parent = dest
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| PathBuf::from("."));
    fs.create_dir_all(&staging_parent)?;

    let staging = staging_parent.join(format!(
        ".plm-resources-staging-{}-{}",
        request.target_kind.as_str(),
        request.manifest.name
    ));
    if fs.exists(&staging) {
        fs.remove(&staging)?;
    }
    fs.create_dir_all(&staging)?;

    for entry in &entries {
        let target = staging.join(&entry.name);
        if entry.absolute.is_dir() {
            fs.copy_dir(&entry.absolute, &target)?;
        } else if entry.absolute.is_file() {
            fs.copy_file(&entry.absolute, &target)?;
        }
    }

    fs.replace_dir(&staging, &dest)?;
    if fs.exists(&staging) {
        let _ = fs.remove(&staging);
    }

    Ok(Some(dest))
}

/// プラグインリソースルートを削除する。
pub fn remove_plugin_resources(
    fs: &dyn FileSystem,
    target_kind: TargetKind,
    scope: Scope,
    project_root: &Path,
    plugin_name: &str,
) -> Result<Option<PathBuf>> {
    let dest = plugin_resources_root(target_kind, scope, project_root, plugin_name)?;
    if fs.exists(&dest) {
        fs.remove(&dest)?;
        return Ok(Some(dest));
    }
    Ok(None)
}

fn validate_plugin_name(name: &str) -> Result<()> {
    if name.is_empty()
        || name.contains('/')
        || name.contains('\\')
        || name.contains('\0')
        || name == "."
        || name == ".."
    {
        return Err(PlmError::Validation(format!(
            "plugin name '{name}' is not a safe path segment for plugin resources"
        )));
    }
    Ok(())
}

#[cfg(test)]
#[path = "resources_test.rs"]
mod tests;

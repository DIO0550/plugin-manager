//! プラグインリソースの除外合成・配置・削除

use crate::component::Scope;
use crate::error::{PlmError, Result};
use crate::fs::FileSystem;
use crate::placement_names::{
    ALL_INSTRUCTION_FILENAMES, CLAUDE_PLUGIN_DIR, PLM_META_FILE, PLUGIN_JSON_FILE,
    PLUGIN_RESOURCES_SUBDIR, PLUGIN_RESOURCE_VCS_NAMES,
};
use crate::plugin::{PluginManifest, PluginName};
use crate::scan::{list_plugin_resources, PluginResourceEntry};
use crate::target::{paths::home_dir, TargetKind};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

/// プラグイン直下リソースの列挙・配置・削除。
pub struct PluginResources<'a> {
    root: &'a Path,
    manifest: &'a PluginManifest,
    name: PluginName<'a>,
}

impl<'a> PluginResources<'a> {
    /// プラグイン名がパスセグメントとして安全なときだけ構築する。
    pub fn new(root: &'a Path, manifest: &'a PluginManifest) -> Result<Self> {
        let name = PluginName::new(&manifest.name).ok_or_else(|| {
            PlmError::Validation(format!(
                "plugin name '{}' is not a safe path segment for plugin resources",
                manifest.name
            ))
        })?;
        Ok(Self {
            root,
            manifest,
            name,
        })
    }

    /// ターゲットへ構造維持で配置する。
    ///
    /// エントリが空なら既存のリソースルートを削除する（stale 掃除）。
    /// 戻り値は配置（または削除）したリソースルート。
    pub fn deploy(
        &self,
        fs: &dyn FileSystem,
        target_kind: TargetKind,
        scope: Scope,
        project_root: &Path,
    ) -> Result<Option<PathBuf>> {
        let dest = self.target_root(target_kind, scope, project_root);
        let entries = self.list();

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
            target_kind.as_str(),
            self.name.as_str()
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

    /// ターゲット上のリソースルートを削除する。
    pub fn remove(
        &self,
        fs: &dyn FileSystem,
        target_kind: TargetKind,
        scope: Scope,
        project_root: &Path,
    ) -> Result<Option<PathBuf>> {
        let dest = self.target_root(target_kind, scope, project_root);
        if fs.exists(&dest) {
            fs.remove(&dest)?;
            return Ok(Some(dest));
        }
        Ok(None)
    }

    fn list(&self) -> Vec<PluginResourceEntry> {
        list_plugin_resources(
            self.root,
            &self.exclusion_paths(),
            &PLUGIN_RESOURCE_VCS_NAMES.iter().copied().collect(),
        )
    }

    fn target_root(&self, target_kind: TargetKind, scope: Scope, project_root: &Path) -> PathBuf {
        let base = match scope {
            Scope::Personal => target_kind.personal_base(&home_dir()),
            Scope::Project => target_kind.project_base(project_root),
        };
        base.join(PLUGIN_RESOURCES_SUBDIR).join(self.name.as_str())
    }

    fn exclusion_paths(&self) -> HashSet<PathBuf> {
        let mut paths = HashSet::new();

        paths.insert(self.manifest.skills_dir(self.root));
        paths.insert(self.manifest.agents_dir(self.root));
        paths.insert(self.manifest.commands_dir(self.root));

        let hooks = self.manifest.hooks_dir(self.root);
        paths.insert(hooks.clone());
        if let Some(parent) = hooks.parent() {
            if parent != self.root {
                paths.insert(parent.to_path_buf());
            }
        }

        paths.insert(self.manifest.instructions_path(self.root));
        paths.insert(self.manifest.instructions_dir(self.root));

        paths.insert(self.root.join(CLAUDE_PLUGIN_DIR));
        paths.insert(self.root.join(PLUGIN_JSON_FILE));
        paths.insert(self.root.join(PLM_META_FILE));

        for name in ALL_INSTRUCTION_FILENAMES {
            paths.insert(self.root.join(name));
        }

        if let Some(ref p) = self.manifest.mcp_servers {
            paths.insert(self.root.join(p));
        }
        if let Some(ref p) = self.manifest.lsp_servers {
            paths.insert(self.root.join(p));
        }

        paths
    }
}

#[cfg(test)]
#[path = "resources_test.rs"]
mod tests;

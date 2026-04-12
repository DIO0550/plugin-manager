//! プラグイン内部モデル
//!
//! パッケージ内の個別プラグインを表現する。コンポーネントスキャン・パス解決の責務を持つ。

use crate::component::{Component, ComponentKind};
use crate::plugin::PluginManifest;
use crate::scan::{
    file_stem_name, list_agent_names, list_command_names, list_hook_names, list_markdown_names,
    list_skill_names,
};
use std::path::{Path, PathBuf};

/// パッケージ内の個別プラグイン
///
/// `manifest`, `path` を保持し、構築時にコンポーネントを一度だけスキャンしてキャッシュする。
/// `Plugin` は構築時点の FS スナップショットを保持し、構築後の FS 変更は反映しない。
/// 全フィールドは private とし、`Plugin::new()` 経由でのみ構築可能にすることで
/// スナップショット不変条件（`components` と他フィールドの整合性）を保護する。
///
/// プラグイン名は `manifest.name` に一本化しており、`name()` アクセサで取得する。
#[derive(Debug, Clone)]
pub struct Plugin {
    manifest: PluginManifest,
    path: PathBuf,
    components: Vec<Component>,
}

impl Plugin {
    /// Plugin を構築し、コンポーネントをスキャンしてキャッシュする
    pub fn new(manifest: PluginManifest, path: PathBuf) -> Self {
        let components = Self::build_components(&path, &manifest);
        Self {
            manifest,
            path,
            components,
        }
    }

    /// プラグイン名を取得（`manifest.name` を参照する）
    pub fn name(&self) -> &str {
        &self.manifest.name
    }

    /// マニフェストを取得
    pub fn manifest(&self) -> &PluginManifest {
        &self.manifest
    }

    /// プラグインのルートパスを取得
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// プラグインのコンポーネントをスキャンして Vec<Component> に変換する
    fn build_components(path: &Path, manifest: &PluginManifest) -> Vec<Component> {
        let mut components = Vec::new();

        for (name, p) in list_skill_names(&manifest.skills_dir(path)) {
            components.push(Component {
                kind: ComponentKind::Skill,
                path: p,
                name,
            });
        }

        for (name, p) in list_agent_names(&manifest.agents_dir(path)) {
            components.push(Component {
                kind: ComponentKind::Agent,
                path: p,
                name,
            });
        }

        for (name, p) in list_command_names(&manifest.commands_dir(path)) {
            components.push(Component {
                kind: ComponentKind::Command,
                path: p,
                name,
            });
        }

        Self::build_instructions(path, manifest, &mut components);

        for (name, p) in list_hook_names(&manifest.hooks_dir(path)) {
            components.push(Component {
                kind: ComponentKind::Hook,
                path: p,
                name,
            });
        }

        components
    }

    fn build_instructions(path: &Path, manifest: &PluginManifest, components: &mut Vec<Component>) {
        if let Some(path_str) = &manifest.instructions {
            let instr_path = path.join(path_str);

            if instr_path.is_file() {
                if let Some(name) = file_stem_name(&instr_path) {
                    components.push(Component {
                        kind: ComponentKind::Instruction,
                        path: instr_path,
                        name,
                    });
                }
                return;
            }

            if instr_path.is_dir() {
                for (name, p) in list_markdown_names(&instr_path) {
                    components.push(Component {
                        kind: ComponentKind::Instruction,
                        path: p,
                        name,
                    });
                }
                return;
            }

            return;
        }

        for (name, p) in list_markdown_names(&manifest.instructions_dir(path)) {
            components.push(Component {
                kind: ComponentKind::Instruction,
                path: p,
                name,
            });
        }

        let agents_md = path.join("AGENTS.md");
        if agents_md.exists() {
            components.push(Component {
                kind: ComponentKind::Instruction,
                path: agents_md,
                name: "AGENTS".to_string(),
            });
        }
    }

    // =========================================================================
    // ディレクトリ解決メソッド
    // =========================================================================

    /// スキルディレクトリのパスを解決
    pub fn skills_dir(&self) -> PathBuf {
        self.manifest.skills_dir(&self.path)
    }

    /// エージェントディレクトリのパスを解決
    pub fn agents_dir(&self) -> PathBuf {
        self.manifest.agents_dir(&self.path)
    }

    /// コマンドディレクトリのパスを解決
    pub fn commands_dir(&self) -> PathBuf {
        self.manifest.commands_dir(&self.path)
    }

    /// インストラクションパスを解決
    pub fn instructions_path(&self) -> PathBuf {
        self.manifest.instructions_path(&self.path)
    }

    /// フックディレクトリのパスを解決
    pub fn hooks_dir(&self) -> PathBuf {
        self.manifest.hooks_dir(&self.path)
    }

    // =========================================================================
    // スキャンメソッド
    // =========================================================================

    /// プラグイン内のコンポーネントを取得（構築時のスナップショット）
    pub fn components(&self) -> &[Component] {
        &self.components
    }
}

#[cfg(test)]
#[path = "plugin_content_test.rs"]
mod plugin_content_test;

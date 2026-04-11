//! プラグイン内部モデル
//!
//! パッケージ内の個別プラグインを表現する。コンポーネントスキャン・パス解決の責務を持つ。

use crate::component::{Component, ComponentKind};
use crate::path_ext::PathExt;
use crate::plugin::PluginManifest;
use crate::scan::{scan_components, AGENT_SUFFIX, MARKDOWN_SUFFIX, PROMPT_SUFFIX};
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
        let scan = scan_components(path, manifest);
        let mut components = Vec::new();

        // Skills
        let skills_dir = manifest.skills_dir(path);
        for name in scan.skills {
            components.push(Component {
                kind: ComponentKind::Skill,
                path: skills_dir.join(&name),
                name,
            });
        }

        // Agents
        let agents_dir = manifest.agents_dir(path);
        for name in scan.agents {
            let component_path = Self::resolve_agent_path(&agents_dir, &name);
            components.push(Component {
                kind: ComponentKind::Agent,
                path: component_path,
                name,
            });
        }

        // Commands
        let commands_dir = manifest.commands_dir(path);
        for name in scan.commands {
            let component_path = Self::resolve_command_path(&commands_dir, &name);
            components.push(Component {
                kind: ComponentKind::Command,
                path: component_path,
                name,
            });
        }

        // Instructions
        for name in scan.instructions {
            let component_path = Self::resolve_instruction_path(path, manifest, &name);
            components.push(Component {
                kind: ComponentKind::Instruction,
                path: component_path,
                name,
            });
        }

        // Hooks
        let hooks_dir = manifest.hooks_dir(path);
        for name in scan.hooks {
            if let Some(component_path) = Self::resolve_hook_path(&hooks_dir, &name) {
                components.push(Component {
                    kind: ComponentKind::Hook,
                    path: component_path,
                    name,
                });
            }
        }

        components
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

    // =========================================================================
    // パス解決ヘルパー（名前 → パス）
    // =========================================================================

    fn resolve_agent_path(agents_dir: &Path, name: &str) -> PathBuf {
        if agents_dir.is_file() {
            return agents_dir.to_path_buf();
        }

        let agent_path = agents_dir.join(format!("{}{}", name, AGENT_SUFFIX));
        if agent_path.exists() {
            agent_path
        } else {
            agents_dir.join(format!("{}{}", name, MARKDOWN_SUFFIX))
        }
    }

    fn resolve_command_path(commands_dir: &Path, name: &str) -> PathBuf {
        let prompt_path = commands_dir.join(format!("{}{}", name, PROMPT_SUFFIX));
        if prompt_path.exists() {
            prompt_path
        } else {
            commands_dir.join(format!("{}{}", name, MARKDOWN_SUFFIX))
        }
    }

    fn resolve_instruction_path(
        plugin_path: &Path,
        manifest: &PluginManifest,
        name: &str,
    ) -> PathBuf {
        // manifest.instructions が指定されている場合は、その設定に従って解決する。
        // "AGENTS" という名前もディレクトリ配下のファイル（例: docs/AGENTS.md）として
        // 扱い、ルート AGENTS.md へ誤ってフォールバックしないようにする。
        if let Some(path_str) = &manifest.instructions {
            let path = plugin_path.join(path_str);
            if path.is_file() {
                return path;
            }
            if path.is_dir() {
                return path.join(format!("{}.md", name));
            }
        }

        // デフォルト設定時のみ、ルートの AGENTS.md を特別扱いする。
        // scan_instructions_internal がデフォルト分岐で "AGENTS" を追加するのと整合する。
        if name == "AGENTS" {
            return plugin_path.join("AGENTS.md");
        }

        manifest
            .instructions_dir(plugin_path)
            .join(format!("{}.md", name))
    }

    fn resolve_hook_path(hooks_dir: &Path, name: &str) -> Option<PathBuf> {
        hooks_dir
            .read_dir_entries()
            .into_iter()
            .filter(|p| p.is_file())
            .find(|path| {
                path.file_name()
                    .and_then(|f| f.to_str())
                    .map(|f| f.rsplit_once('.').map(|(n, _)| n).unwrap_or(f) == name)
                    .unwrap_or(false)
            })
    }
}

#[cfg(test)]
#[path = "plugin_content_test.rs"]
mod plugin_content_test;

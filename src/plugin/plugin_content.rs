//! プラグイン内部モデル
//!
//! パッケージ内の個別プラグインを表現する。コンポーネントスキャン・パス解決の責務を持つ。

use crate::component::{Component, ComponentKind};
use crate::error::{PlmError, Result};
use crate::plugin::PluginManifest;
use crate::scan::{
    file_stem_name, list_agent_names, list_command_names, list_hook_names, list_markdown_names,
    list_skill_names,
};
use crate::target::PluginOrigin;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// プラグイン名と元名から平坦化済みの `Component.name` を組み立てる純粋関数。
///
/// 常に `"{plugin_name}_{original_name}"` 形式を返す。サニタイズは行わない。
///
/// # Arguments
///
/// * `plugin_name` - `PluginManifest.name`
/// * `original_name` - スキャン層が返す元名（中間ディレクトリ名は含まない）
pub(crate) fn flatten_name(plugin_name: &str, original_name: &str) -> String {
    format!("{plugin_name}_{original_name}")
}

/// 平坦化に使う名前（plugin_name / original_name）が単一パスセグメントとして
/// 安全に扱えることを検証する。`placement_location` が `base.join(name)` や
/// `format!("{name}.agent.md")` を直接行うため、パス区切り・null 文字・親参照
/// などが含まれているとターゲットディレクトリの外へエスケープされる恐れがある。
///
/// # 拒否する条件
/// - 空文字
/// - `/` `\` `\0` を含む
/// - `.` または `..` 単体
/// - `Path::new(name).components()` が複数コンポーネントを返す
fn validate_path_segment(label: &str, name: &str) -> Result<()> {
    if name.is_empty() {
        return Err(PlmError::Validation(format!(
            "{label} must not be empty for component flattening"
        )));
    }
    if name.contains('/') || name.contains('\\') || name.contains('\0') {
        return Err(PlmError::Validation(format!(
            "{label} '{name}' must not contain path separators or null bytes"
        )));
    }
    if name == "." || name == ".." {
        return Err(PlmError::Validation(format!(
            "{label} '{name}' must not be a parent/current directory reference"
        )));
    }
    let mut components = Path::new(name).components();
    let first = components.next();
    if components.next().is_some() || !matches!(first, Some(std::path::Component::Normal(_))) {
        return Err(PlmError::Validation(format!(
            "{label} '{name}' must be a single normal path segment"
        )));
    }
    Ok(())
}

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
    origin: PluginOrigin,
    components: Vec<Component>,
}

impl Plugin {
    /// Plugin を構築し、コンポーネントをスキャンしてキャッシュする
    ///
    /// # Arguments
    ///
    /// * `manifest` - Plugin manifest describing the plugin metadata and layout.
    /// * `path` - Root directory path of the plugin on disk.
    /// * `origin` - Plugin origin (marketplace and plugin identifier).
    pub fn new(manifest: PluginManifest, path: PathBuf, origin: PluginOrigin) -> Result<Self> {
        let components = Self::build_components(&path, &manifest)?;
        Ok(Self {
            manifest,
            path,
            origin,
            components,
        })
    }

    /// テスト専用コンストラクタ: FS スキャンをバイパスしてコンポーネントを直接注入する
    #[cfg(test)]
    pub(crate) fn new_for_test(
        manifest: PluginManifest,
        path: PathBuf,
        components: Vec<Component>,
    ) -> Self {
        Self {
            manifest,
            path,
            origin: PluginOrigin::from_marketplace("test", "test"),
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

    /// プラグインの出自を取得
    pub fn origin(&self) -> &PluginOrigin {
        &self.origin
    }

    /// プラグインのコンポーネントをスキャンして Vec<Component> に変換する
    ///
    /// # Arguments
    ///
    /// * `path` - Plugin root directory used to resolve component directories.
    /// * `manifest` - Plugin manifest that defines component layout and names.
    fn build_components(path: &Path, manifest: &PluginManifest) -> Result<Vec<Component>> {
        let plugin_name = manifest.name.as_str();
        let mut components = Vec::new();

        for (kind, items) in [
            (
                ComponentKind::Skill,
                list_skill_names(&manifest.skills_dir(path)),
            ),
            (
                ComponentKind::Agent,
                list_agent_names(&manifest.agents_dir(path)),
            ),
            (
                ComponentKind::Command,
                list_command_names(&manifest.commands_dir(path)),
            ),
            (
                ComponentKind::Hook,
                list_hook_names(&manifest.hooks_dir(path)),
            ),
        ] {
            let flattened = flatten_components(kind, plugin_name, items)?;
            detect_name_collisions(&flattened)?;
            components.extend(flattened);
        }

        Self::build_instructions(path, manifest, &mut components);

        Ok(components)
    }

    /// Append instruction components resolved from the manifest into `components`.
    ///
    /// # Arguments
    ///
    /// * `path` - Plugin root directory used to resolve instruction paths.
    /// * `manifest` - Plugin manifest that optionally specifies an instructions path.
    /// * `components` - Output buffer that receives discovered instruction components.
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

    /// プラグイン内のコンポーネントを取得（構築時のスナップショット）
    pub fn components(&self) -> &[Component] {
        &self.components
    }
}

/// 元名のリストを平坦化済み `Component` のリストに変換する。
///
/// `plugin_name` および各 `original_name` を `validate_path_segment` で検証し、
/// `flatten_name` で `"{plugin}_{original}"` 形式に変換する。検証に失敗した
/// 場合は `PlmError::Validation` を返す。重複検出は行わない（呼び出し側で
/// `detect_name_collisions` を使う）。
fn flatten_components(
    kind: ComponentKind,
    plugin_name: &str,
    items: Vec<(String, PathBuf)>,
) -> Result<Vec<Component>> {
    validate_path_segment("plugin name", plugin_name)?;
    items
        .into_iter()
        .map(|(original_name, path)| {
            validate_path_segment("component name", &original_name)?;
            Ok(Component {
                kind,
                name: flatten_name(plugin_name, &original_name),
                path,
            })
        })
        .collect()
}

/// 同一スライス内で `Component.name` が重複していないかを検出する。
///
/// 重複があれば `PlmError::Validation` を返し、衝突した 2 パスをメッセージに
/// 含める。スライスは同一 `kind` を前提とする（kind ごとに別呼び出しすること）。
fn detect_name_collisions(components: &[Component]) -> Result<()> {
    let mut seen: HashMap<&str, &Path> = HashMap::new();
    for component in components {
        if let Some(existing) = seen.get(component.name.as_str()) {
            return Err(PlmError::Validation(format!(
                "duplicate component name '{}' for kind {:?}: \
                 '{}' conflicts with '{}'. \
                 Two source paths produce the same flattened name; \
                 rename one of the source directories/files to disambiguate.",
                component.name,
                component.kind,
                existing.display(),
                component.path.display(),
            )));
        }
        seen.insert(component.name.as_str(), component.path.as_path());
    }
    Ok(())
}

#[cfg(test)]
#[path = "plugin_content_test.rs"]
mod plugin_content_test;

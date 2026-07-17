//! Cursor ターゲット実装（Skills / Agents / Commands / Instructions / Hooks）

use crate::component::{Component, ComponentKind, PlacementContext, PlacementLocation, Scope};
use crate::error::Result;
use crate::target::paths::base_dir;
use crate::target::placed_common;
use crate::target::scanner::{scan_components, ScannedComponent};
use crate::target::{Target, TargetKind};
use std::path::{Path, PathBuf};

const CURSOR_SUBDIR: &str = ".cursor";

/// Cursor ターゲット
pub struct CursorTarget;

impl CursorTarget {
    pub fn new() -> Self {
        Self
    }

    /// スコープに応じたベースディレクトリを取得
    ///
    /// # Arguments
    ///
    /// * `scope` - Scope (`Personal` or `Project`) that selects the base directory.
    /// * `project_root` - Project root directory used for project scope.
    fn base_dir(scope: Scope, project_root: &Path) -> PathBuf {
        base_dir(scope, project_root, CURSOR_SUBDIR, CURSOR_SUBDIR)
    }

    /// この組み合わせで配置できるか
    ///
    /// Instructions は Project スコープのみ（Personal の User Rules は対象外）。
    ///
    /// # Arguments
    ///
    /// * `kind` - Component kind to check.
    /// * `scope` - Scope (`Personal` or `Project`) to check.
    fn can_place(kind: ComponentKind, scope: Scope) -> bool {
        matches!(
            (kind, scope),
            (ComponentKind::Skill, _)
                | (ComponentKind::Agent, _)
                | (ComponentKind::Command, _)
                | (ComponentKind::Instruction, Scope::Project)
                | (ComponentKind::Hook, _)
        )
    }

    /// Cursor が期待するプレーン `.md` ファイルか判定する。
    ///
    /// PLM 内部の `.agent.md` / `.prompt.md` サフィックスは Cursor では
    /// 認識されないため除外する（#359 検証結果）。
    fn is_plain_markdown(name: &str) -> bool {
        name.ends_with(".md") && !name.ends_with(".agent.md") && !name.ends_with(".prompt.md")
    }

    /// Cursor は 1 スコープにつき単一の `hooks.json` を読むため、複数 Hook を
    /// 個別配置すると同じファイルを上書きする。マージ未実装の間は拒否する。
    ///
    /// # Arguments
    ///
    /// * `components` - Components being placed in a single install/import call.
    pub fn hook_component_conflict_error(components: &[Component]) -> Option<String> {
        let hook_count = components
            .iter()
            .filter(|component| component.kind == ComponentKind::Hook)
            .count();

        (hook_count > 1).then(|| {
            format!(
                "Cursor target supports a single hooks.json per scope; {} Hook components would overwrite each other. Select one Hook component or wait for merge support.",
                hook_count
            )
        })
    }

    /// 配置先 `hooks.json` がすでに存在し、`target_path` が現在のプラグインの
    /// 管理下に無い場合にエラー文字列を返す。
    ///
    /// # Arguments
    ///
    /// * `target_path` - Resolved destination path (e.g. `.cursor/hooks.json`).
    /// * `plugin_root` - Cached plugin directory; `.plm-meta.json` lives here.
    pub fn hook_overwrite_error(target_path: &Path, plugin_root: &Path) -> Option<String> {
        if !target_path.exists() {
            return None;
        }

        let already_owned = crate::plugin::meta::load_meta(plugin_root)
            .map(|meta| meta.manages_file("cursor", target_path))
            .unwrap_or(false);

        if already_owned {
            return None;
        }

        Some(format!(
            "{} already exists and is not managed by this plugin. \
             Refusing to overwrite; remove the file or merge it manually before re-installing.",
            target_path.display()
        ))
    }

    /// コンポーネント種別に応じたフィルタリング
    ///
    /// # Arguments
    ///
    /// * `c` - Scanned component entry.
    /// * `kind` - Component kind expected for the entry.
    fn filter_component(c: &ScannedComponent, kind: ComponentKind) -> Option<String> {
        match kind {
            ComponentKind::Skill if c.is_dir => {
                let skill_md = c.path.join("SKILL.md");
                if skill_md.exists() {
                    Some(c.name.clone())
                } else {
                    None
                }
            }
            ComponentKind::Agent if !c.is_dir && Self::is_plain_markdown(&c.name) => {
                Some(c.name.trim_end_matches(".md").to_string())
            }
            ComponentKind::Command if !c.is_dir && Self::is_plain_markdown(&c.name) => {
                Some(c.name.trim_end_matches(".md").to_string())
            }
            ComponentKind::Hook if !c.is_dir && c.name == "hooks.json" => Some("hooks".to_string()),
            _ => None,
        }
    }
}

impl Default for CursorTarget {
    fn default() -> Self {
        Self::new()
    }
}

impl Target for CursorTarget {
    fn display_name(&self) -> &'static str {
        "Cursor"
    }

    fn kind(&self) -> TargetKind {
        TargetKind::Cursor
    }

    fn supported_components(&self) -> &[ComponentKind] {
        &[
            ComponentKind::Skill,
            ComponentKind::Agent,
            ComponentKind::Command,
            ComponentKind::Instruction,
            ComponentKind::Hook,
        ]
    }

    fn placement_location(&self, context: &PlacementContext) -> Option<PlacementLocation> {
        let kind = context.kind();
        let scope = context.scope();
        if !Self::can_place(kind, scope) {
            return None;
        }

        let project_root = context.project_root();
        let base = Self::base_dir(scope, project_root);
        let name = context.name();

        Some(match kind {
            // フラット構造: skills/<flattened_name> (ディレクトリ)
            ComponentKind::Skill => PlacementLocation::dir(base.join("skills").join(name)),
            // フラット構造: agents/<flattened_name>.md (ファイル)
            ComponentKind::Agent => {
                PlacementLocation::file(base.join("agents").join(format!("{name}.md")))
            }
            // フラット構造: commands/<flattened_name>.md (ファイル)
            ComponentKind::Command => {
                PlacementLocation::file(base.join("commands").join(format!("{name}.md")))
            }
            // Project scope: AGENTS.md at project root (shared with Codex)
            ComponentKind::Instruction => PlacementLocation::file(project_root.join("AGENTS.md")),
            ComponentKind::Hook => PlacementLocation::file(base.join("hooks.json")),
        })
    }

    fn list_placed(
        &self,
        kind: ComponentKind,
        scope: Scope,
        project_root: &Path,
    ) -> Result<Vec<String>> {
        if !Self::can_place(kind, scope) {
            return Ok(vec![]);
        }

        if kind == ComponentKind::Instruction {
            return Ok(placed_common::list_instruction(
                self,
                scope,
                project_root,
                "AGENTS.md",
            ));
        }

        let base = Self::base_dir(scope, project_root);
        let dir_path = match kind {
            ComponentKind::Skill => base.join("skills"),
            ComponentKind::Agent => base.join("agents"),
            ComponentKind::Command => base.join("commands"),
            ComponentKind::Hook => base.clone(),
            _ => return Ok(vec![]),
        };

        let names = scan_components(&dir_path)?
            .into_iter()
            .filter_map(|c| Self::filter_component(&c, kind))
            .collect();

        Ok(names)
    }
}

#[cfg(test)]
#[path = "cursor_test.rs"]
mod tests;

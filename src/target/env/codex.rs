//! OpenAI Codex ターゲット実装

use crate::component::{Component, ComponentKind, PlacementContext, PlacementLocation, Scope};
use crate::error::Result;
use crate::target::paths::base_dir;
use crate::target::placed_common;
use crate::target::scanner::{scan_components, ScannedComponent};
use crate::target::{Target, TargetKind};
use std::path::{Path, PathBuf};

const CODEX_SUBDIR: &str = ".codex";

/// OpenAI Codex ターゲット
pub struct CodexTarget;

impl CodexTarget {
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
        base_dir(scope, project_root, CODEX_SUBDIR, CODEX_SUBDIR)
    }

    /// この組み合わせで配置できるか
    ///
    /// # Arguments
    ///
    /// * `kind` - Component kind to check.
    fn can_place(kind: ComponentKind) -> bool {
        kind != ComponentKind::Command
    }

    /// Codex は 1 スコープにつき単一の `hooks.json` を読むため、複数 Hook を
    /// 個別配置すると同じファイルを上書きする。マージ未実装の間は拒否する。
    pub fn hook_component_conflict_error(components: &[Component]) -> Option<String> {
        let hook_count = components
            .iter()
            .filter(|component| component.kind == ComponentKind::Hook)
            .count();

        (hook_count > 1).then(|| {
            format!(
                "Codex target supports a single hooks.json per scope; {} Hook components would overwrite each other. Select one Hook component or wait for merge support.",
                hook_count
            )
        })
    }

    /// 配置先 `hooks.json` がすでに存在し、`target_path` が現在のプラグインの
    /// 管理下に無い場合にエラー文字列を返す。
    ///
    /// 共有パス（`.codex/hooks.json` / `~/.codex/hooks.json`）を上書きして
    /// しまうことで、ユーザーが手書きした hooks や別プラグインが配置した
    /// hooks を黙って消してしまうのを防ぐ。再 install（同プラグイン）の場合は
    /// `.plm-meta.json` の `managedFiles["codex"]` に `target_path` が
    /// 含まれている場合のみ許可する。
    ///
    /// 旧実装は `statusByTarget["codex"] == "enabled"` を所有権の根拠に
    /// していたが、この値は scope 非依存かつコンポーネント種別非依存で、
    /// (a) personal で enable された状態が project 側 `.codex/hooks.json`
    /// の上書きを許してしまう、(b) Skill のみ配置したプラグインでも
    /// 上書きガードを通過してしまう、という不整合があった。
    /// 絶対パス単位の `managedFiles` でこの両方を解消する。
    ///
    /// # Arguments
    ///
    /// * `target_path` - Resolved destination path (e.g. `.codex/hooks.json`).
    /// * `plugin_root` - Cached plugin directory; `.plm-meta.json` lives here.
    pub fn hook_overwrite_error(target_path: &Path, plugin_root: &Path) -> Option<String> {
        if !target_path.exists() {
            return None;
        }

        let already_owned = crate::plugin::meta::load_meta(plugin_root)
            .map(|meta| meta.manages_file("codex", target_path))
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
            // Skill: 直下に SKILL.md が存在するディレクトリのみ採用する。
            // 旧 3 階層 `<plural>/<mp>/<plg>/<skill>/SKILL.md` の中間段
            // (`<mp>` ディレクトリ等) を Skill と誤認しないための二重防御。
            ComponentKind::Skill if c.is_dir && c.path.join("SKILL.md").is_file() => {
                Some(c.name.clone())
            }
            ComponentKind::Agent if !c.is_dir && c.name.ends_with(".agent.md") => {
                Some(c.name.trim_end_matches(".agent.md").to_string())
            }
            ComponentKind::Hook if !c.is_dir && c.name == "hooks.json" => Some("hooks".to_string()),
            _ => None,
        }
    }
}

impl Default for CodexTarget {
    fn default() -> Self {
        Self::new()
    }
}

impl Target for CodexTarget {
    fn name(&self) -> &'static str {
        "codex"
    }

    fn display_name(&self) -> &'static str {
        "OpenAI Codex"
    }

    fn kind(&self) -> TargetKind {
        TargetKind::Codex
    }

    fn supported_components(&self) -> &[ComponentKind] {
        &[
            ComponentKind::Skill,
            ComponentKind::Agent,
            ComponentKind::Instruction,
            ComponentKind::Hook,
        ]
    }

    fn placement_location(&self, context: &PlacementContext) -> Option<PlacementLocation> {
        let kind = context.kind();
        if !Self::can_place(kind) {
            return None;
        }

        let scope = context.scope();
        let project_root = context.project_root();
        let base = Self::base_dir(scope, project_root);
        let name = context.name();

        Some(match kind {
            // フラット構造: skills/<flattened_name> (ディレクトリ)
            ComponentKind::Skill => PlacementLocation::dir(base.join("skills").join(name)),
            // フラット構造: agents/<flattened_name>.agent.md (ファイル)
            ComponentKind::Agent => {
                PlacementLocation::file(base.join("agents").join(format!("{}.agent.md", name)))
            }
            ComponentKind::Instruction => match scope {
                // Project scope: AGENTS.md is at project root, not in .codex
                Scope::Project => PlacementLocation::file(project_root.join("AGENTS.md")),
                Scope::Personal => PlacementLocation::file(base.join("AGENTS.md")),
            },
            ComponentKind::Hook => PlacementLocation::file(base.join("hooks.json")),
            ComponentKind::Command => return None,
        })
    }

    fn list_placed(
        &self,
        kind: ComponentKind,
        scope: Scope,
        project_root: &Path,
    ) -> Result<Vec<String>> {
        if !Self::can_place(kind) {
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
#[path = "codex_test.rs"]
mod tests;

//! コンポーネント種別とスコープの定義

use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// コンポーネント種別
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, ValueEnum, Serialize, Deserialize,
)]
#[serde(rename_all = "lowercase")]
pub enum ComponentKind {
    /// スキル（SKILL.md形式）
    Skill,
    /// エージェント（.agent.md形式）
    Agent,
    /// コマンド（.prompt.md形式）
    Command,
    /// インストラクション（AGENTS.md, copilot-instructions.md形式）
    Instruction,
    /// フック（任意のスクリプト）
    Hook,
}

impl ComponentKind {
    /// 識別子文字列を取得
    pub fn as_str(&self) -> &'static str {
        match self {
            ComponentKind::Skill => "skill",
            ComponentKind::Agent => "agent",
            ComponentKind::Command => "command",
            ComponentKind::Instruction => "instruction",
            ComponentKind::Hook => "hook",
        }
    }

    /// 複数形の文字列を取得
    pub fn plural(&self) -> &'static str {
        match self {
            ComponentKind::Skill => "skills",
            ComponentKind::Agent => "agents",
            ComponentKind::Command => "commands",
            ComponentKind::Instruction => "instructions",
            ComponentKind::Hook => "hooks",
        }
    }

    /// 表示名を取得
    pub fn display_name(&self) -> &'static str {
        match self {
            ComponentKind::Skill => "Skill",
            ComponentKind::Agent => "Agent",
            ComponentKind::Command => "Command",
            ComponentKind::Instruction => "Instruction",
            ComponentKind::Hook => "Hook",
        }
    }

    /// 全コンポーネント種別を取得
    pub fn all() -> &'static [ComponentKind] {
        &[
            ComponentKind::Skill,
            ComponentKind::Agent,
            ComponentKind::Command,
            ComponentKind::Instruction,
            ComponentKind::Hook,
        ]
    }
}

impl std::fmt::Display for ComponentKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

/// プラグイン名と元名から平坦化済み識別子を組み立てる。
///
/// 常に `"{plugin_name}_{original_name}"` 形式を返す。サニタイズは行わない。
///
/// # Arguments
///
/// * `plugin_name` - `PluginManifest.name`
/// * `original_name` - スキャン層が返す元名（中間ディレクトリ名は含まない）
pub fn flatten_name(plugin_name: &str, original_name: &str) -> String {
    format!("{plugin_name}_{original_name}")
}

/// プラグイン内のコンポーネント
///
/// `name` は他ターゲット互換のフラット化識別子（`{plugin}_{original}`）。
/// Cursor Skill 配置では `original_name` をディレクトリ名に使い、frontmatter
/// の `name` と一致させる（Issue #377）。
#[derive(Debug, Clone)]
pub struct Component {
    pub kind: ComponentKind,
    /// フラット化済み識別子（他ターゲット・カタログ用の正）
    pub name: String,
    /// スキャン時の元名（ディレクトリ名 / ファイル stem）。
    /// フラット化経路（[`Component::flattened`]）でのみ `Some`。
    /// [`Component::new`] では `None`（Instruction 等の非フラット化専用）。
    pub original_name: Option<String>,
    /// `PluginManifest.name`。非フラット化では空文字
    pub plugin_name: String,
    pub path: PathBuf,
}

impl Component {
    /// 非フラット化コンポーネント用コンストラクタ。
    ///
    /// `original_name` は `None`。Instruction 等、フラット化しない種別向け。
    /// Skill / Agent / Command / Hook は [`Component::flattened`] を使うこと。
    /// `original_name` 未設定の Skill を Cursor に流すと配置がスキップされる
    /// （フラット化名への危険なフォールバックはしない — #377）。
    ///
    /// # Arguments
    ///
    /// * `kind` - Component kind.
    /// * `name` - Component identifier.
    /// * `path` - Filesystem path of the component source.
    pub fn new(kind: ComponentKind, name: impl Into<String>, path: impl Into<PathBuf>) -> Self {
        Self {
            kind,
            name: name.into(),
            original_name: None,
            plugin_name: String::new(),
            path: path.into(),
        }
    }

    /// フラット化済みコンポーネントを構築する。
    ///
    /// # Arguments
    ///
    /// * `kind` - Component kind.
    /// * `plugin_name` - Plugin manifest name used as flatten prefix.
    /// * `original_name` - Pre-flatten directory / file stem name.
    /// * `path` - Filesystem path of the component source.
    pub fn flattened(
        kind: ComponentKind,
        plugin_name: impl Into<String>,
        original_name: impl Into<String>,
        path: impl Into<PathBuf>,
    ) -> Self {
        let plugin_name = plugin_name.into();
        let original_name = original_name.into();
        Self {
            kind,
            name: flatten_name(&plugin_name, &original_name),
            original_name: Some(original_name),
            plugin_name,
            path: path.into(),
        }
    }
}

/// デプロイスコープ
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, ValueEnum, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Scope {
    /// ユーザーレベル（~/.codex/, ~/.copilot/）
    Personal,
    /// プロジェクトレベル（.codex/, .github/）
    Project,
}

impl Scope {
    /// 識別子文字列を取得
    pub fn as_str(&self) -> &'static str {
        match self {
            Scope::Personal => "personal",
            Scope::Project => "project",
        }
    }

    /// 表示名を取得
    pub fn display_name(&self) -> &'static str {
        match self {
            Scope::Personal => "Personal",
            Scope::Project => "Project",
        }
    }

    /// 説明文を取得
    pub fn description(&self) -> &'static str {
        match self {
            Scope::Personal => "~/.codex/, ~/.copilot/",
            Scope::Project => ".codex/, .github/",
        }
    }
}

impl std::fmt::Display for Scope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

#[cfg(test)]
#[path = "kind_test.rs"]
mod tests;

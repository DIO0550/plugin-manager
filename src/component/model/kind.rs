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

/// プラグイン内のコンポーネント
#[derive(Debug, Clone)]
pub struct Component {
    pub kind: ComponentKind,
    pub name: String,
    pub path: PathBuf,
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

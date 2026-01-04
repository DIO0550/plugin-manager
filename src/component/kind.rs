//! コンポーネント種別とスコープの定義

use clap::ValueEnum;
use std::path::PathBuf;

/// コンポーネント種別
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, ValueEnum)]
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
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
mod tests {
    use super::*;

    #[test]
    fn test_component_kind_as_str() {
        assert_eq!(ComponentKind::Skill.as_str(), "skill");
        assert_eq!(ComponentKind::Agent.as_str(), "agent");
        assert_eq!(ComponentKind::Command.as_str(), "command");
        assert_eq!(ComponentKind::Instruction.as_str(), "instruction");
        assert_eq!(ComponentKind::Hook.as_str(), "hook");
    }

    #[test]
    fn test_component_kind_plural() {
        assert_eq!(ComponentKind::Skill.plural(), "skills");
        assert_eq!(ComponentKind::Agent.plural(), "agents");
    }

    #[test]
    fn test_component_kind_all() {
        assert_eq!(ComponentKind::all().len(), 5);
    }

    #[test]
    fn test_scope_as_str() {
        assert_eq!(Scope::Personal.as_str(), "personal");
        assert_eq!(Scope::Project.as_str(), "project");
    }

    #[test]
    fn test_component_kind_all_elements_unique() {
        let all = ComponentKind::all();
        let mut seen = std::collections::HashSet::new();
        for kind in all {
            assert!(seen.insert(kind), "Duplicate ComponentKind found: {:?}", kind);
        }
    }

    #[test]
    fn test_component_kind_as_str_not_empty() {
        for kind in ComponentKind::all() {
            assert!(!kind.as_str().is_empty(), "{:?}.as_str() is empty", kind);
        }
    }

    #[test]
    fn test_component_kind_plural_not_empty() {
        for kind in ComponentKind::all() {
            assert!(!kind.plural().is_empty(), "{:?}.plural() is empty", kind);
        }
    }

    #[test]
    fn test_component_kind_display_name_not_empty() {
        for kind in ComponentKind::all() {
            assert!(
                !kind.display_name().is_empty(),
                "{:?}.display_name() is empty",
                kind
            );
        }
    }

    #[test]
    fn test_component_kind_as_str_unique() {
        let all = ComponentKind::all();
        let mut seen = std::collections::HashSet::new();
        for kind in all {
            let s = kind.as_str();
            assert!(seen.insert(s), "Duplicate as_str found: {}", s);
        }
    }

    #[test]
    fn test_component_kind_plural_unique() {
        let all = ComponentKind::all();
        let mut seen = std::collections::HashSet::new();
        for kind in all {
            let s = kind.plural();
            assert!(seen.insert(s), "Duplicate plural found: {}", s);
        }
    }

    #[test]
    fn test_scope_as_str_not_empty() {
        assert!(!Scope::Personal.as_str().is_empty());
        assert!(!Scope::Project.as_str().is_empty());
    }

    #[test]
    fn test_scope_display_name_not_empty() {
        assert!(!Scope::Personal.display_name().is_empty());
        assert!(!Scope::Project.display_name().is_empty());
    }

    #[test]
    fn test_scope_description_not_empty() {
        assert!(!Scope::Personal.description().is_empty());
        assert!(!Scope::Project.description().is_empty());
    }
}

//! スキャン関連の定数定義
//!
//! ターゲット非依存の配置名は [`crate::component::ComponentKind`] を正とし、
//! 本モジュールは互換のための薄い re-export を提供する。

use crate::component::ComponentKind;

/// スキルマニフェストファイル名
pub const SKILL_MANIFEST: &str = ComponentKind::skill_manifest();

/// コンポーネント検出用のファイルサフィックス
pub const AGENT_SUFFIX: &str = match ComponentKind::Agent.file_suffix() {
    Some(s) => s,
    None => ".agent.md",
};
pub const PROMPT_SUFFIX: &str = match ComponentKind::Command.file_suffix() {
    Some(s) => s,
    None => ".prompt.md",
};
pub const MARKDOWN_SUFFIX: &str = ".md";

/// デフォルトのコンポーネントディレクトリパス（プラグイン内相対・表示用 plural と同一）
pub const DEFAULT_SKILLS_DIR: &str = ComponentKind::Skill.plural();
pub const DEFAULT_AGENTS_DIR: &str = ComponentKind::Agent.plural();
pub const DEFAULT_COMMANDS_DIR: &str = ComponentKind::Command.plural();
pub const DEFAULT_HOOKS_DIR: &str = ComponentKind::Hook.plural();

/// デフォルトのインストラクション設定（プラグインパッケージ内。ターゲット配置名とは別）
pub const DEFAULT_INSTRUCTIONS_FILE: &str = "instructions.md";
pub const DEFAULT_INSTRUCTIONS_DIR: &str = ComponentKind::Instruction.plural();

//! ターゲット環境の抽象化
//!
//! 各AI開発環境（Antigravity, Codex, Copilot）への配置を抽象化する。
//! 使う側は具体的なターゲットを意識せず、`Target` traitを通じて操作する。
//!
//! ## 使い方
//!
//! ```ignore
//! use crate::component::{ComponentRef, PlacementContext, PlacementScope, ProjectContext};
//!
//! let target = parse_target("codex")?;
//! let origin = PluginOrigin::from_marketplace("official", "my-plugin");
//! let ctx = PlacementContext {
//!     component: ComponentRef::new(ComponentKind::Skill, "my-skill"),
//!     origin: &origin,
//!     scope: PlacementScope(Scope::Project),
//!     project: ProjectContext::new(&project_root),
//! };
//! let location = target.placement_location(&ctx);
//! ```

mod antigravity;
mod codex;
mod copilot;
mod effect;
mod gemini_cli;
mod registry;
pub mod scanner;

pub use registry::{AddResult, RemoveResult, TargetRegistry};

pub use antigravity::AntigravityTarget;
pub use codex::CodexTarget;
pub use copilot::CopilotTarget;
pub use effect::{AffectedTargets, OperationResult};
pub use gemini_cli::GeminiCliTarget;
// PluginOrigin はモジュール内で定義されているのでここでは再エクスポート不要

use crate::component::{AgentFormat, CommandFormat, ComponentKind};
// componentモジュールから再エクスポート
pub use crate::component::Scope;
use crate::component::{
    ComponentRef, PlacementContext, PlacementLocation, PlacementScope, ProjectContext,
};
use crate::error::{PlmError, Result};
use crate::fs::{FileSystem, RealFs};
use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// プラグインの出自情報
///
/// コンポーネントがどのマーケットプレイス・プラグインから来たかを追跡する。
/// デプロイ時に `<marketplace>/<plugin>/<component>` 階層を構築するために使用。
#[derive(Debug, Clone)]
pub struct PluginOrigin {
    /// マーケットプレイス名（直接GitHubの場合は "github"）
    pub marketplace: String,
    /// プラグイン名（直接GitHubの場合は "owner--repo" 形式）
    pub plugin: String,
}

impl PluginOrigin {
    /// マーケットプレイス経由のプラグイン
    pub fn from_marketplace(marketplace: &str, plugin: &str) -> Self {
        Self {
            marketplace: marketplace.to_string(),
            plugin: plugin.to_string(),
        }
    }

    /// 直接GitHub経由のプラグイン
    pub fn from_github(owner: &str, repo: &str) -> Self {
        Self {
            marketplace: "github".to_string(),
            plugin: format!("{}--{}", owner, repo),
        }
    }

    /// CachedPlugin から PluginOrigin を生成
    pub fn from_cached_plugin(marketplace: Option<&str>, plugin_name: &str) -> Self {
        Self {
            marketplace: marketplace.unwrap_or("github").to_string(),
            plugin: plugin_name.to_string(),
        }
    }
}

/// ターゲット種別（CLIオプション用）
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TargetKind {
    Antigravity,
    Codex,
    Copilot,
    GeminiCli,
}

impl TargetKind {
    /// ターゲット名を取得
    pub fn as_str(&self) -> &'static str {
        match self {
            TargetKind::Antigravity => "antigravity",
            TargetKind::Codex => "codex",
            TargetKind::Copilot => "copilot",
            TargetKind::GeminiCli => "gemini",
        }
    }

    /// Command コンポーネントのフォーマットを取得
    ///
    /// ターゲット環境が期待する Command ファイル形式を返す。
    pub fn command_format(&self) -> CommandFormat {
        match self {
            TargetKind::Antigravity => CommandFormat::ClaudeCode, // Antigravity は Skills のみ
            TargetKind::Codex => CommandFormat::Codex,
            TargetKind::Copilot => CommandFormat::Copilot,
            TargetKind::GeminiCli => CommandFormat::ClaudeCode, // Gemini CLI は Command 非サポート
        }
    }

    /// Agent コンポーネントのフォーマットを取得
    ///
    /// ターゲット環境が期待する Agent ファイル形式を返す。
    pub fn agent_format(&self) -> AgentFormat {
        match self {
            TargetKind::Antigravity => AgentFormat::ClaudeCode, // Antigravity は Agent 非サポート
            TargetKind::Codex => AgentFormat::Codex,
            TargetKind::Copilot => AgentFormat::Copilot,
            TargetKind::GeminiCli => AgentFormat::ClaudeCode, // Gemini CLI は Agent 非サポート
        }
    }
}

/// ターゲット環境の抽象化trait
///
/// 各ターゲット（Codex, Copilot）がこのtraitを実装する。
/// 使う側は具体的なターゲットを意識せずに配置操作を行える。
pub trait Target: Send + Sync {
    /// ターゲット識別子（"codex", "copilot"）
    fn name(&self) -> &'static str;

    /// 表示名
    fn display_name(&self) -> &'static str;

    /// ターゲット種別
    fn kind(&self) -> TargetKind;

    /// Command コンポーネントのフォーマットを取得
    fn command_format(&self) -> CommandFormat {
        self.kind().command_format()
    }

    /// Agent コンポーネントのフォーマットを取得
    fn agent_format(&self) -> AgentFormat {
        self.kind().agent_format()
    }

    /// サポートするコンポーネント種別
    fn supported_components(&self) -> &[ComponentKind];

    /// 指定コンポーネント種別をサポートするか
    fn supports(&self, kind: ComponentKind) -> bool {
        self.supported_components().contains(&kind)
    }

    /// 指定コンポーネント・スコープの組み合わせをサポートするか
    fn supports_scope(&self, kind: ComponentKind, scope: Scope) -> bool {
        let dummy_origin = PluginOrigin::from_marketplace("test", "test");
        let ctx = PlacementContext {
            component: ComponentRef::new(kind, "test"),
            origin: &dummy_origin,
            scope: PlacementScope(scope),
            project: ProjectContext::new(Path::new(".")),
        };
        self.placement_location(&ctx).is_some()
    }

    /// 配置先ロケーションを取得
    ///
    /// `PlacementContext` を受け取り、`PlacementLocation` を返す。
    /// サポートしていない組み合わせの場合は `None` を返す。
    fn placement_location(&self, context: &PlacementContext) -> Option<PlacementLocation>;

    /// コンポーネントを削除
    fn remove(&self, context: &PlacementContext) -> Result<()> {
        let fs = RealFs;
        let location = self.placement_location(context).ok_or_else(|| {
            PlmError::Deployment(format!(
                "{} is not supported on {} with {} scope",
                context.kind(),
                self.display_name(),
                context.scope().as_str()
            ))
        })?;

        let path = location.as_path();
        if fs.exists(path) {
            if location.is_dir() {
                fs.remove_dir_all(path)?;
            } else {
                fs.remove_file(path)?;
            }
        }

        Ok(())
    }

    /// 配置済みコンポーネント一覧を取得
    fn list_placed(
        &self,
        kind: ComponentKind,
        scope: Scope,
        project_root: &Path,
    ) -> Result<Vec<String>>;
}

/// ターゲット名をパースしてTarget traitオブジェクトを返す
///
/// parse_sourceと同じパターンで、使う側は具体的なターゲットを意識しない。
pub fn parse_target(name: &str) -> Result<Box<dyn Target>> {
    match name {
        "antigravity" => Ok(Box::new(AntigravityTarget::new())),
        "codex" => Ok(Box::new(CodexTarget::new())),
        "copilot" => Ok(Box::new(CopilotTarget::new())),
        "gemini" => Ok(Box::new(GeminiCliTarget::new())),
        _ => Err(PlmError::TargetNotFound(name.to_string())),
    }
}

/// 全ターゲットを取得
pub fn all_targets() -> Vec<Box<dyn Target>> {
    vec![
        Box::new(AntigravityTarget::new()),
        Box::new(CodexTarget::new()),
        Box::new(CopilotTarget::new()),
        Box::new(GeminiCliTarget::new()),
    ]
}

#[cfg(test)]
#[path = "target_test.rs"]
mod tests;

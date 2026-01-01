//! ターゲット環境の抽象化
//!
//! 各AI開発環境（Codex, Copilot）への配置を抽象化する。
//! 使う側は具体的なターゲットを意識せず、`Target` traitを通じて操作する。
//!
//! ## 使い方
//!
//! ```ignore
//! let target = parse_target("codex")?;
//! let origin = PluginOrigin::from_marketplace("official", "my-plugin");
//! target.place(ComponentKind::Skill, Scope::Project, &source, "my-skill", &origin, &project_root)?;
//! ```

mod codex;
mod copilot;

pub use codex::CodexTarget;
pub use copilot::CopilotTarget;
// PluginOrigin はモジュール内で定義されているのでここでは再エクスポート不要

use crate::component::ComponentKind;
// componentモジュールから再エクスポート
pub use crate::component::{ComponentPlacement, Scope};
use crate::error::{PlmError, Result};
use clap::ValueEnum;
use std::fs;
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
#[derive(Debug, Clone, ValueEnum)]
pub enum TargetKind {
    Codex,
    Copilot,
}

impl TargetKind {
    /// ターゲット名を取得
    pub fn as_str(&self) -> &'static str {
        match self {
            TargetKind::Codex => "codex",
            TargetKind::Copilot => "copilot",
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

    /// サポートするコンポーネント種別
    fn supported_components(&self) -> &[ComponentKind];

    /// 指定コンポーネント種別をサポートするか
    fn supports(&self, kind: ComponentKind) -> bool {
        self.supported_components().contains(&kind)
    }

    /// 指定コンポーネント・スコープの組み合わせをサポートするか
    fn supports_scope(&self, kind: ComponentKind, scope: Scope) -> bool {
        let dummy_origin = PluginOrigin::from_marketplace("test", "test");
        self.placement_path(kind, scope, "test", &dummy_origin, Path::new("."))
            .is_some()
    }

    /// コンポーネントの配置先ベースパスを取得
    ///
    /// サポートしていない組み合わせの場合は `None` を返す。
    /// Agent/Promptの場合はディレクトリを返す。完全なファイルパスが必要な場合は
    /// `full_placement_path` を使用する。
    ///
    /// 階層構造: `<base>/<kind>/<marketplace>/<plugin>/<component>`
    fn placement_path(
        &self,
        kind: ComponentKind,
        scope: Scope,
        component_name: &str,
        origin: &PluginOrigin,
        project_root: &Path,
    ) -> Option<std::path::PathBuf>;

    /// コンポーネントの完全な配置先パスを取得
    ///
    /// Agent/Promptの場合はファイル名を含めた完全なパスを返す。
    fn full_placement_path(
        &self,
        kind: ComponentKind,
        scope: Scope,
        component_name: &str,
        origin: &PluginOrigin,
        project_root: &Path,
    ) -> Option<std::path::PathBuf> {
        let base = self.placement_path(kind, scope, component_name, origin, project_root)?;
        Some(match kind {
            ComponentKind::Agent => base.join(format!("{}.agent.md", component_name)),
            ComponentKind::Prompt => base.join(format!("{}.prompt.md", component_name)),
            _ => base,
        })
    }

    /// コンポーネントを削除
    fn remove(
        &self,
        kind: ComponentKind,
        scope: Scope,
        component_name: &str,
        origin: &PluginOrigin,
        project_root: &Path,
    ) -> Result<()> {
        let path = self
            .full_placement_path(kind, scope, component_name, origin, project_root)
            .ok_or_else(|| {
                PlmError::Deployment(format!(
                    "{} is not supported on {} with {} scope",
                    kind,
                    self.display_name(),
                    scope.as_str()
                ))
            })?;

        if path.exists() {
            if path.is_dir() {
                fs::remove_dir_all(&path)?;
            } else {
                fs::remove_file(&path)?;
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
        "codex" => Ok(Box::new(CodexTarget::new())),
        "copilot" => Ok(Box::new(CopilotTarget::new())),
        _ => Err(PlmError::TargetNotFound(name.to_string())),
    }
}

/// 全ターゲットを取得
pub fn all_targets() -> Vec<Box<dyn Target>> {
    vec![
        Box::new(CodexTarget::new()),
        Box::new(CopilotTarget::new()),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_target_codex() {
        let target = parse_target("codex").unwrap();
        assert_eq!(target.name(), "codex");
    }

    #[test]
    fn test_parse_target_copilot() {
        let target = parse_target("copilot").unwrap();
        assert_eq!(target.name(), "copilot");
    }

    #[test]
    fn test_parse_target_unknown() {
        let result = parse_target("unknown");
        assert!(result.is_err());
    }

    #[test]
    fn test_all_targets() {
        let targets = all_targets();
        assert_eq!(targets.len(), 2);
        assert!(targets.iter().any(|t| t.name() == "codex"));
        assert!(targets.iter().any(|t| t.name() == "copilot"));
    }
}

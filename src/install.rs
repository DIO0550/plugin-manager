use std::path::{Path, PathBuf};

use crate::component::{AgentFormat, CommandFormat, ComponentKind, Scope};
use crate::component::{Component, ComponentDeployment, ConversionConfig, DeploymentOutput};
use crate::component::{ComponentRef, PlacementContext, PlacementScope, ProjectContext};
use crate::plugin::{
    cleanup_legacy_hierarchy, MarketplaceContent, PackageCache, PackageCacheAccess,
};
use crate::source::parse_source;
use crate::target::{CodexTarget, PluginOrigin, Target, TargetKind};

pub mod format;
pub use crate::hooks::converter::{ConversionWarning, SourceFormat};

/// スキャン済みプラグイン
///
/// ダウンロード済みプラグインからコンポーネントをスキャンした結果。
#[derive(Debug)]
pub struct ScannedPlugin {
    package: MarketplaceContent,
    pub components: Vec<Component>,
}

impl ScannedPlugin {
    /// プラグイン名を取得
    pub fn name(&self) -> &str {
        self.package.name()
    }

    /// キャッシュディレクトリ名を返す（`id` が `None` の場合は `name` にフォールバック）
    pub fn id(&self) -> &str {
        crate::plugin::resolve_id(self.package.id(), self.package.name())
    }

    /// マーケットプレイス名を取得
    pub fn marketplace(&self) -> Option<&str> {
        self.package.marketplace()
    }

    /// Command コンポーネントのソースフォーマットを取得
    pub fn command_format(&self) -> CommandFormat {
        self.package.command_format()
    }

    /// Agent コンポーネントのソースフォーマットを取得
    pub fn agent_format(&self) -> AgentFormat {
        self.package.agent_format()
    }

    /// パッケージキャッシュのルートパスを取得
    pub fn plugin_root(&self) -> &Path {
        self.package.path()
    }
}

/// 配置リクエスト
pub struct PlaceRequest<'a> {
    pub scanned: &'a ScannedPlugin,
    pub targets: &'a [Box<dyn Target>],
    pub scope: Scope,
    pub project_root: &'a Path,
}

/// 配置結果
#[derive(Debug)]
pub struct PlaceResult {
    pub plugin_name: String,
    pub successes: Vec<PlaceSuccess>,
    pub failures: Vec<PlaceFailure>,
}

/// 配置成功
#[derive(Debug)]
pub struct PlaceSuccess {
    pub target: String,
    pub target_kind: TargetKind,
    pub component_name: String,
    pub component_kind: ComponentKind,
    pub target_path: PathBuf,
    pub source_format: Option<String>,
    pub dest_format: Option<String>,
    /// Hook 変換時の警告（`HookConverted` 以外は空）。
    pub hook_warnings: Vec<ConversionWarning>,
    /// `HookConverted` 時に生成されたスクリプト数（それ以外は 0）。
    pub script_count: usize,
    /// `HookConverted` 時に変換後 JSON に残った hook 定義数（それ以外は 0）。
    pub hook_count: usize,
    /// Hook 変換時の入力形式。`Some(SourceFormat::ClaudeCode)` のときのみ
    /// `(converted from Claude Code format)` サフィックスを表示する。
    ///
    /// `None` になるケース:
    /// - **Hook 以外**（Skill / Agent / Command / Instruction）
    /// - **Hook だが `DeploymentOutput::Copied` 経路**を通った場合（version 付き
    ///   Copilot 形式の完全 passthrough。`HookConvertOutput` を経由しないため
    ///   `source_format` を保持しない）
    pub hook_source_format: Option<SourceFormat>,
}

/// 配置失敗の段階
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlaceFailureStage {
    Resolution,
    Deployment,
}

/// 配置失敗
#[derive(Debug)]
pub struct PlaceFailure {
    pub target: String,
    pub component_name: String,
    pub component_kind: ComponentKind,
    pub error: String,
    pub stage: PlaceFailureStage,
}

/// 汎用プラグインダウンロード
///
/// `source_str` をパースし、GitHub またはマーケットプレイスからプラグインをダウンロードする。
/// デフォルトの `PackageCache` を使用する CLI/TUI 向け便利関数。
///
/// # Arguments
///
/// * `source_str` - Plugin source locator (GitHub or marketplace format).
/// * `force` - When true, ignore the existing cache entry and re-download.
pub async fn download_plugin(
    source_str: &str,
    force: bool,
) -> crate::error::Result<MarketplaceContent> {
    let cache = PackageCache::new()?;
    download_plugin_with_cache(source_str, force, &cache).await
}

/// キャッシュを注入可能な汎用プラグインダウンロード
///
/// テストや DI が必要な場面で使用する。
///
/// # Arguments
///
/// * `source_str` - Plugin source locator (GitHub or marketplace format).
/// * `force` - When true, ignore the existing cache entry and re-download.
/// * `cache` - Package cache implementation used to resolve and store downloads.
pub async fn download_plugin_with_cache(
    source_str: &str,
    force: bool,
    cache: &dyn PackageCacheAccess,
) -> crate::error::Result<MarketplaceContent> {
    let source = parse_source(source_str)?;
    let cached = source.download(cache, force).await?;
    MarketplaceContent::try_from(cached)
}

/// プラグインのコンポーネントをスキャン
///
/// `type_filter` が指定された場合、該当する種別のコンポーネントのみを返す。
///
/// # Arguments
///
/// * `package` - Downloaded plugin content to scan.
/// * `type_filter` - Optional list of component kinds to retain; `None` keeps all kinds.
pub fn scan_plugin(
    package: &MarketplaceContent,
    type_filter: Option<&[ComponentKind]>,
) -> Result<ScannedPlugin, String> {
    let mut components = package.components();

    if let Some(filter) = type_filter {
        components.retain(|c| filter.contains(&c.kind));
    }

    Ok(ScannedPlugin {
        package: package.clone(),
        components,
    })
}

/// プラグインのコンポーネントをターゲットに配置
///
/// # Arguments
///
/// * `request` - Placement request describing the scanned plugin, targets, scope, and project root.
pub fn place_plugin(request: &PlaceRequest) -> PlaceResult {
    let mut successes = Vec::new();
    let mut failures = Vec::new();

    let origin =
        PluginOrigin::from_cached_plugin(request.scanned.marketplace(), request.scanned.id());

    for target in request.targets {
        let failures_before = failures.len();
        let codex_hook_conflict = if target.kind() == TargetKind::Codex {
            CodexTarget::hook_component_conflict_error(&request.scanned.components)
        } else {
            None
        };

        for component in &request.scanned.components {
            if !target.supports(component.kind) {
                continue;
            }

            if component.kind == ComponentKind::Hook {
                if let Some(error) = &codex_hook_conflict {
                    failures.push(PlaceFailure {
                        target: target.name().to_string(),
                        component_name: component.name.clone(),
                        component_kind: component.kind,
                        error: error.clone(),
                        stage: PlaceFailureStage::Resolution,
                    });
                    continue;
                }
            }

            let ctx = PlacementContext {
                component: ComponentRef::new(component.kind, &component.name),
                origin: &origin,
                scope: PlacementScope::new(request.scope),
                project: ProjectContext::new(request.project_root),
            };

            let target_path = match target.placement_location(&ctx) {
                Some(location) => location.into_path(),
                None => continue,
            };

            if component.kind == ComponentKind::Hook && target.kind() == TargetKind::Codex {
                if let Some(error) =
                    CodexTarget::hook_overwrite_error(&target_path, request.scanned.plugin_root())
                {
                    failures.push(PlaceFailure {
                        target: target.name().to_string(),
                        component_name: component.name.clone(),
                        component_kind: component.kind,
                        error,
                        stage: PlaceFailureStage::Resolution,
                    });
                    continue;
                }
            }

            let conversion = match component.kind {
                ComponentKind::Command => ConversionConfig::Command {
                    source: request.scanned.command_format(),
                    dest: target.command_format(),
                },
                ComponentKind::Agent => ConversionConfig::Agent {
                    source: request.scanned.agent_format(),
                    dest: target.agent_format(),
                },
                ComponentKind::Hook
                    if matches!(target.kind(), TargetKind::Codex | TargetKind::Copilot) =>
                {
                    ConversionConfig::Hook {
                        target_kind: target.kind(),
                        plugin_root: Some(request.scanned.plugin_root().to_path_buf()),
                    }
                }
                _ => ConversionConfig::None,
            };

            let deployment = match ComponentDeployment::builder()
                .component(component.clone())
                .scope(request.scope)
                .target_path(&target_path)
                .conversion(conversion)
                .build()
            {
                Ok(d) => d,
                Err(e) => {
                    failures.push(PlaceFailure {
                        target: target.name().to_string(),
                        component_name: component.name.clone(),
                        component_kind: component.kind,
                        error: e.to_string(),
                        stage: PlaceFailureStage::Resolution,
                    });
                    continue;
                }
            };

            match deployment.execute() {
                Ok(result) => {
                    let (source_format, dest_format) = match &result {
                        DeploymentOutput::CommandConverted(conv) if conv.converted => (
                            Some(conv.source_format.to_string()),
                            Some(conv.dest_format.to_string()),
                        ),
                        DeploymentOutput::AgentConverted(conv) if conv.converted => (
                            Some(conv.source_format.to_string()),
                            Some(conv.dest_format.to_string()),
                        ),
                        _ => (None, None),
                    };

                    let (hook_warnings, script_count, hook_count, hook_source_format) =
                        match &result {
                            DeploymentOutput::HookConverted(hr) => (
                                hr.warnings.clone(),
                                hr.script_count,
                                hr.hook_count,
                                Some(hr.source_format),
                            ),
                            _ => (Vec::new(), 0, 0, None),
                        };

                    successes.push(PlaceSuccess {
                        target: target.name().to_string(),
                        target_kind: target.kind(),
                        component_name: component.name.clone(),
                        component_kind: component.kind,
                        target_path: deployment.path().to_path_buf(),
                        source_format,
                        dest_format,
                        hook_warnings,
                        script_count,
                        hook_count,
                        hook_source_format,
                    });
                }
                Err(e) => {
                    failures.push(PlaceFailure {
                        target: target.name().to_string(),
                        component_name: component.name.clone(),
                        component_kind: component.kind,
                        error: e.to_string(),
                        stage: PlaceFailureStage::Deployment,
                    });
                }
            }
        }

        // 旧 3 階層構造 (<base>/<plural>/<mp>/<plg>) のクリーンアップは
        // 当該 target 内で failure が発生しなかった場合に実行する。
        // 途中で failure があった場合に旧階層を消すと「新旧どちらも残らない」
        // 状態を招きうるため、ロールバック相当として旧階層を温存する。
        // 一方、配置件数が 0 件（target がサポートしないコンポーネントだけ
        // の場合など）でも failure がなければ、install 実行時の自動クリーン
        // アップ対象とし旧階層が永続するのを避ける。
        let target_had_failure = failures.len() > failures_before;
        if !target_had_failure {
            cleanup_legacy_hierarchy(target.kind(), &origin, request.project_root);
        }
    }

    PlaceResult {
        plugin_name: request.scanned.name().to_string(),
        successes,
        failures,
    }
}

#[cfg(test)]
#[path = "install_test.rs"]
mod tests;

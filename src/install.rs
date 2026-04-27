use std::path::{Path, PathBuf};

use crate::component::{AgentFormat, CommandFormat, ComponentKind, Scope};
use crate::component::{Component, ComponentDeployment, ConversionConfig, DeploymentOutput};
use crate::component::{ComponentRef, PlacementContext, PlacementScope, ProjectContext};
use crate::plugin::{
    cleanup_legacy_hierarchy, MarketplaceContent, PackageCache, PackageCacheAccess,
};
use crate::source::parse_source;
use crate::target::{PluginOrigin, Target, TargetKind};

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
    pub component_name: String,
    pub component_kind: ComponentKind,
    pub target_path: PathBuf,
    pub source_format: Option<String>,
    pub dest_format: Option<String>,
    pub hook_warnings: Vec<String>,
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
        let successes_before = successes.len();
        let failures_before = failures.len();

        for component in &request.scanned.components {
            if !target.supports(component.kind) {
                continue;
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

            let conversion = match component.kind {
                ComponentKind::Command => ConversionConfig::Command {
                    source: request.scanned.command_format(),
                    dest: target.command_format(),
                },
                ComponentKind::Agent => ConversionConfig::Agent {
                    source: request.scanned.agent_format(),
                    dest: target.agent_format(),
                },
                ComponentKind::Hook if target.kind() == TargetKind::Copilot => {
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

                    let hook_warnings = match &result {
                        DeploymentOutput::HookConverted(hr) => {
                            hr.warnings.iter().map(|w| w.to_string()).collect()
                        }
                        _ => vec![],
                    };

                    successes.push(PlaceSuccess {
                        target: target.name().to_string(),
                        component_name: component.name.clone(),
                        component_kind: component.kind,
                        target_path: deployment.path().to_path_buf(),
                        source_format,
                        dest_format,
                        hook_warnings,
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
        // 当該 target 内の配置がすべて成功し、かつ少なくとも 1 件配置できた
        // 場合にのみ実行する。途中で failure があった場合に旧階層を消すと
        // 「新旧どちらも残らない」状態を招きうるため、ロールバック相当として
        // 旧階層を温存する。
        let target_had_failure = failures.len() > failures_before;
        let target_had_success = successes.len() > successes_before;
        if target_had_success && !target_had_failure {
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

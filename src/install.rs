use std::path::{Path, PathBuf};

use crate::component::{AgentFormat, CommandFormat, ComponentKind, Scope};
use crate::component::{Component, ComponentDeployment, DeploymentResult};
use crate::component::{ComponentRef, PlacementContext, PlacementScope, ProjectContext};
use crate::plugin::{MarketplacePackage, PluginCache, PluginCacheAccess};
use crate::source::{parse_source, MarketplaceSource, PluginSource};
use crate::target::{PluginOrigin, Target, TargetKind};

/// スキャン済みプラグイン
///
/// ダウンロード済みプラグインからコンポーネントをスキャンした結果。
#[derive(Debug)]
pub struct ScannedPlugin {
    pub name: String,
    pub marketplace: Option<String>,
    package: MarketplacePackage,
    pub components: Vec<ScannedComponent>,
}

impl ScannedPlugin {
    /// Command コンポーネントのソースフォーマットを取得
    pub fn command_format(&self) -> CommandFormat {
        self.package.command_format()
    }

    /// Agent コンポーネントのソースフォーマットを取得
    pub fn agent_format(&self) -> AgentFormat {
        self.package.agent_format()
    }

    /// プラグインキャッシュのルートパスを取得
    pub fn plugin_root(&self) -> &Path {
        &self.package.path
    }
}

/// スキャン済みコンポーネント
#[derive(Debug, Clone)]
pub struct ScannedComponent {
    pub kind: ComponentKind,
    pub name: String,
    pub source_path: PathBuf,
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
/// デフォルトの `PluginCache` を使用する CLI/TUI 向け便利関数。
pub async fn download_plugin(source_str: &str, force: bool) -> Result<MarketplacePackage, String> {
    let cache = PluginCache::new().map_err(|e| format!("Failed to access cache: {e}"))?;
    download_plugin_with_cache(source_str, force, &cache).await
}

/// キャッシュを注入可能な汎用プラグインダウンロード
///
/// テストや DI が必要な場面で使用する。
pub async fn download_plugin_with_cache(
    source_str: &str,
    force: bool,
    cache: &dyn PluginCacheAccess,
) -> Result<MarketplacePackage, String> {
    let source = parse_source(source_str).map_err(|e| e.to_string())?;
    let cached = source
        .download(cache, force)
        .await
        .map_err(|e| e.to_string())?;
    Ok(MarketplacePackage::from(cached))
}

/// マーケットプレイス経由のプラグインダウンロード
///
/// デフォルトの `PluginCache` を使用する CLI/TUI 向け便利関数。
pub async fn download_marketplace_plugin(
    plugin_name: &str,
    marketplace_name: &str,
    force: bool,
) -> Result<MarketplacePackage, String> {
    let cache = PluginCache::new().map_err(|e| format!("Failed to access cache: {e}"))?;
    download_marketplace_plugin_with_cache(plugin_name, marketplace_name, force, &cache).await
}

/// キャッシュを注入可能なマーケットプレイス経由のプラグインダウンロード
///
/// テストや DI が必要な場面で使用する。
pub async fn download_marketplace_plugin_with_cache(
    plugin_name: &str,
    marketplace_name: &str,
    force: bool,
    cache: &dyn PluginCacheAccess,
) -> Result<MarketplacePackage, String> {
    let source = MarketplaceSource::new(plugin_name, marketplace_name);
    let cached = source
        .download(cache, force)
        .await
        .map_err(|e| e.to_string())?;
    Ok(MarketplacePackage::from(cached))
}

/// プラグインのコンポーネントをスキャン
///
/// `type_filter` が指定された場合、該当する種別のコンポーネントのみを返す。
pub fn scan_plugin(
    package: &MarketplacePackage,
    type_filter: Option<&[ComponentKind]>,
) -> Result<ScannedPlugin, String> {
    let mut components = package.components();

    if let Some(filter) = type_filter {
        components.retain(|c| filter.contains(&c.kind));
    }

    let scanned_components = components
        .into_iter()
        .map(|c| ScannedComponent {
            kind: c.kind,
            name: c.name,
            source_path: c.path,
        })
        .collect();

    Ok(ScannedPlugin {
        name: package.name.clone(),
        marketplace: package.marketplace.clone(),
        package: package.clone(),
        components: scanned_components,
    })
}

/// プラグインのコンポーネントをターゲットに配置
pub fn place_plugin(request: &PlaceRequest) -> PlaceResult {
    let mut successes = Vec::new();
    let mut failures = Vec::new();

    let origin = PluginOrigin::from_cached_plugin(
        request.scanned.marketplace.as_deref(),
        &request.scanned.name,
    );

    for target in request.targets {
        for component in &request.scanned.components {
            if !target.supports(component.kind) {
                continue;
            }

            let comp = Component {
                kind: component.kind,
                name: component.name.clone(),
                path: component.source_path.clone(),
            };

            let ctx = PlacementContext {
                component: ComponentRef::new(component.kind, &component.name),
                origin: &origin,
                scope: PlacementScope(request.scope),
                project: ProjectContext::new(request.project_root),
            };

            let target_path = match target.placement_location(&ctx) {
                Some(location) => location.into_path(),
                None => continue,
            };

            let mut builder = ComponentDeployment::builder()
                .component(&comp)
                .scope(request.scope)
                .target_path(&target_path);

            if component.kind == ComponentKind::Command {
                builder = builder
                    .source_format(request.scanned.command_format())
                    .dest_format(target.command_format());
            }

            if component.kind == ComponentKind::Agent {
                builder = builder
                    .source_agent_format(request.scanned.agent_format())
                    .dest_agent_format(target.agent_format());
            }

            if component.kind == ComponentKind::Hook && target.kind() == TargetKind::Copilot {
                builder = builder
                    .hook_convert(true)
                    .target_kind(target.kind())
                    .plugin_root(request.scanned.plugin_root());
            }

            let deployment = match builder.build() {
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
                        DeploymentResult::Converted(conv) if conv.converted => (
                            Some(conv.source_format.to_string()),
                            Some(conv.dest_format.to_string()),
                        ),
                        DeploymentResult::AgentConverted(conv) if conv.converted => (
                            Some(conv.source_format.to_string()),
                            Some(conv.dest_format.to_string()),
                        ),
                        _ => (None, None),
                    };

                    let hook_warnings = match &result {
                        DeploymentResult::HookConverted(hr) => {
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
    }

    PlaceResult {
        plugin_name: request.scanned.name.clone(),
        successes,
        failures,
    }
}

#[cfg(test)]
#[path = "install_test.rs"]
mod tests;

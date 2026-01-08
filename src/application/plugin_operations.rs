//! プラグイン操作ユースケース
//!
//! Enable/Disable/Uninstall などのプラグインライフサイクル操作を提供する。

use crate::component::{ComponentDeployment, ComponentKind, Scope};
use crate::domain::placement::{ComponentRef, PlacementContext, PlacementScope, ProjectContext};
use crate::plugin::{PluginCache, PluginManifest};
use crate::target::{all_targets, AffectedTargets, OperationResult, PluginOrigin, Target};
use std::fs;
use std::path::{Path, PathBuf};

/// プラグインを Disable（デプロイ先から削除、キャッシュは残す）
///
/// # Arguments
/// * `plugin_name` - プラグイン名
/// * `marketplace` - マーケットプレイス名（任意）
/// * `project_root` - プロジェクトルートパス
pub fn disable_plugin(
    plugin_name: &str,
    marketplace: Option<&str>,
    project_root: &Path,
) -> OperationResult {
    let cache = match PluginCache::new() {
        Ok(c) => c,
        Err(e) => return OperationResult::error(format!("Failed to access cache: {}", e)),
    };

    // プラグインがキャッシュに存在するか確認
    if !cache.is_cached(marketplace, plugin_name) {
        return OperationResult::error(format!("Plugin '{}' not found in cache", plugin_name));
    }

    // マニフェストを読み込んでコンポーネント情報を取得
    let manifest = match cache.load_manifest(marketplace, plugin_name) {
        Ok(m) => m,
        Err(e) => return OperationResult::error(format!("Failed to load manifest: {}", e)),
    };

    // プラグインのオリジン情報を作成
    let origin = match marketplace {
        Some(mp) => PluginOrigin::from_marketplace(mp, &manifest.name),
        None => PluginOrigin::from_marketplace("github", &manifest.name),
    };

    let plugin_path = cache.plugin_path(marketplace, plugin_name);

    // 値オブジェクトで結果を記録
    let mut affected = AffectedTargets::new();
    let targets = all_targets();

    for target in &targets {
        match remove_plugin_from_target(
            target.as_ref(),
            &origin,
            &manifest,
            &plugin_path,
            project_root,
        ) {
            Ok(removed_count) => {
                affected.record_success(target.name(), removed_count);
            }
            Err(e) => {
                affected.record_error(target.name(), e);
            }
        }
    }

    // 値オブジェクトが結果を生成
    affected.into_result()
}

/// プラグインを Enable（キャッシュからデプロイ先に配置）
///
/// # Arguments
/// * `plugin_name` - プラグイン名
/// * `marketplace` - マーケットプレイス名（任意）
/// * `project_root` - プロジェクトルートパス
pub fn enable_plugin(
    plugin_name: &str,
    marketplace: Option<&str>,
    project_root: &Path,
) -> OperationResult {
    let cache = match PluginCache::new() {
        Ok(c) => c,
        Err(e) => return OperationResult::error(format!("Failed to access cache: {}", e)),
    };

    // プラグインがキャッシュに存在するか確認
    if !cache.is_cached(marketplace, plugin_name) {
        return OperationResult::error(format!("Plugin '{}' not found in cache", plugin_name));
    }

    // マニフェストを読み込んでコンポーネント情報を取得
    let manifest = match cache.load_manifest(marketplace, plugin_name) {
        Ok(m) => m,
        Err(e) => return OperationResult::error(format!("Failed to load manifest: {}", e)),
    };

    // プラグインのオリジン情報を作成
    let origin = match marketplace {
        Some(mp) => PluginOrigin::from_marketplace(mp, &manifest.name),
        None => PluginOrigin::from_marketplace("github", &manifest.name),
    };

    let plugin_path = cache.plugin_path(marketplace, plugin_name);

    // PluginDeployment を構築
    let plugin = PluginDeployment {
        origin,
        manifest,
        path: plugin_path,
    };

    // 値オブジェクトで結果を記録
    let mut affected = AffectedTargets::new();
    let targets = all_targets();

    for target in &targets {
        let ctx = TargetContext {
            target: target.as_ref(),
            project_root,
        };
        match deploy_plugin_to_target(&ctx, &plugin) {
            Ok(deployed_count) => {
                affected.record_success(target.name(), deployed_count);
            }
            Err(e) => {
                affected.record_error(target.name(), e);
            }
        }
    }

    // 値オブジェクトが結果を生成
    affected.into_result()
}

/// プラグインを Uninstall（デプロイ先 + キャッシュ削除）
///
/// # Arguments
/// * `plugin_name` - プラグイン名
/// * `marketplace` - マーケットプレイス名（任意）
/// * `project_root` - プロジェクトルートパス
pub fn uninstall_plugin(
    plugin_name: &str,
    marketplace: Option<&str>,
    project_root: &Path,
) -> OperationResult {
    // まずデプロイ先から削除
    let disable_result = disable_plugin(plugin_name, marketplace, project_root);
    if !disable_result.success {
        return disable_result;
    }

    // キャッシュから削除
    let cache = match PluginCache::new() {
        Ok(c) => c,
        Err(e) => return OperationResult::error(format!("Failed to access cache: {}", e)),
    };

    if let Err(e) = cache.remove(marketplace, plugin_name) {
        return OperationResult::error(format!("Failed to remove from cache: {}", e));
    }

    disable_result
}

/// 特定のターゲットからプラグインのコンポーネントを削除
///
/// 削除したコンポーネント数を返す。
fn remove_plugin_from_target(
    target: &dyn Target,
    origin: &PluginOrigin,
    manifest: &PluginManifest,
    plugin_path: &Path,
    project_root: &Path,
) -> std::result::Result<usize, String> {
    let mut removed_count = 0usize;

    // スキルを削除
    if target.supports(ComponentKind::Skill) {
        let skills_dir = manifest.skills_dir(plugin_path);
        if skills_dir.exists() {
            for entry in skills_dir.read_dir().map_err(|e| e.to_string())? {
                let entry = entry.map_err(|e| e.to_string())?;
                if entry.path().is_dir() {
                    let name = entry.file_name().to_string_lossy().to_string();
                    if remove_component(target, origin, ComponentKind::Skill, &name, project_root)?
                    {
                        removed_count += 1;
                    }
                }
            }
        }
    }

    // エージェントを削除
    if target.supports(ComponentKind::Agent) {
        let agents_path = manifest.agents_dir(plugin_path);
        if agents_path.exists() {
            if agents_path.is_file() {
                // 単一ファイルの場合
                if let Some(name) = agents_path.file_stem() {
                    let name = name.to_string_lossy().to_string();
                    if remove_component(target, origin, ComponentKind::Agent, &name, project_root)?
                    {
                        removed_count += 1;
                    }
                }
            } else {
                // ディレクトリの場合
                for entry in agents_path.read_dir().map_err(|e| e.to_string())? {
                    let entry = entry.map_err(|e| e.to_string())?;
                    let path = entry.path();
                    if path.is_file() {
                        if let Some(name) = path.file_stem() {
                            let name = name.to_string_lossy().to_string();
                            if remove_component(
                                target,
                                origin,
                                ComponentKind::Agent,
                                &name,
                                project_root,
                            )? {
                                removed_count += 1;
                            }
                        }
                    }
                }
            }
        }
    }

    // コマンドを削除
    if target.supports(ComponentKind::Command) {
        let commands_dir = manifest.commands_dir(plugin_path);
        if commands_dir.exists() {
            for entry in commands_dir.read_dir().map_err(|e| e.to_string())? {
                let entry = entry.map_err(|e| e.to_string())?;
                let path = entry.path();
                if path.is_file() {
                    if let Some(name) = path.file_stem() {
                        let name = name.to_string_lossy().to_string();
                        if remove_component(
                            target,
                            origin,
                            ComponentKind::Command,
                            &name,
                            project_root,
                        )? {
                            removed_count += 1;
                        }
                    }
                }
            }
        }
    }

    // インストラクションを削除
    if target.supports(ComponentKind::Instruction) {
        let instructions_dir = manifest.instructions_dir(plugin_path);
        if instructions_dir.exists() {
            for entry in instructions_dir.read_dir().map_err(|e| e.to_string())? {
                let entry = entry.map_err(|e| e.to_string())?;
                let path = entry.path();
                if path.is_file() {
                    if let Some(name) = path.file_stem() {
                        let name = name.to_string_lossy().to_string();
                        if remove_component(
                            target,
                            origin,
                            ComponentKind::Instruction,
                            &name,
                            project_root,
                        )? {
                            removed_count += 1;
                        }
                    }
                }
            }
        }
    }

    // プラグインディレクトリをクリーンアップ
    cleanup_plugin_directories(target.name(), origin, project_root);

    Ok(removed_count)
}

/// プラグインディレクトリをクリーンアップ
///
/// コンポーネント削除後に空になったプラグインディレクトリを削除する。
fn cleanup_plugin_directories(target_name: &str, origin: &PluginOrigin, project_root: &Path) {
    // ターゲットごとのディレクトリ構造
    let dirs_to_check: Vec<(&str, &str)> = match target_name {
        "codex" => vec![("agents", ".codex"), ("skills", ".codex")],
        "copilot" => vec![
            ("agents", ".github"),
            ("prompts", ".github"),
            ("skills", ".github"),
        ],
        _ => vec![],
    };

    for (kind_dir, base_dir) in dirs_to_check {
        // プラグインディレクトリのパス: <base>/<kind>/<marketplace>/<plugin>/
        let plugin_dir = project_root
            .join(base_dir)
            .join(kind_dir)
            .join(&origin.marketplace)
            .join(&origin.plugin);

        // ディレクトリが存在して空なら削除
        if plugin_dir.is_dir() {
            if let Ok(entries) = fs::read_dir(&plugin_dir) {
                if entries.count() == 0 {
                    let _ = fs::remove_dir(&plugin_dir);
                }
            }
        }

        // マーケットプレイスディレクトリも空なら削除
        let marketplace_dir = project_root
            .join(base_dir)
            .join(kind_dir)
            .join(&origin.marketplace);

        if marketplace_dir.is_dir() {
            if let Ok(entries) = fs::read_dir(&marketplace_dir) {
                if entries.count() == 0 {
                    let _ = fs::remove_dir(&marketplace_dir);
                }
            }
        }

        // kind ディレクトリも空なら削除
        let kind_dir_path = project_root.join(base_dir).join(kind_dir);

        if kind_dir_path.is_dir() {
            if let Ok(entries) = fs::read_dir(&kind_dir_path) {
                if entries.count() == 0 {
                    let _ = fs::remove_dir(&kind_dir_path);
                }
            }
        }
    }
}

/// 単一コンポーネントを削除
///
/// 削除に成功したら true、存在しなかった場合は false を返す。
fn remove_component(
    target: &dyn Target,
    origin: &PluginOrigin,
    kind: ComponentKind,
    name: &str,
    project_root: &Path,
) -> std::result::Result<bool, String> {
    let context = PlacementContext {
        component: ComponentRef::new(kind, name),
        origin,
        scope: PlacementScope(Scope::Project),
        project: ProjectContext::new(project_root),
    };

    // 削除を試みる（存在しなくてもエラーにしない）
    match target.remove(&context) {
        Ok(()) => Ok(true),
        Err(e) => {
            let err_str = e.to_string();
            // ファイルが存在しない場合は無視
            if err_str.contains("not found") || err_str.contains("No such file") {
                Ok(false)
            } else {
                Err(err_str)
            }
        }
    }
}

/// ターゲット環境のコンテキスト
///
/// デプロイ操作で共通して使用する環境パラメータをまとめる。
struct TargetContext<'a> {
    target: &'a dyn Target,
    project_root: &'a Path,
}

/// デプロイ用プラグイン情報（Application層DTO）
///
/// デプロイ操作に必要な origin, manifest, path を保持。
/// CachedPlugin から変換して使用する。
pub struct PluginDeployment {
    pub origin: PluginOrigin,
    pub manifest: PluginManifest,
    pub path: PathBuf,
}

impl PluginDeployment {
    /// スキルディレクトリのパスを取得
    pub fn skills_dir(&self) -> PathBuf {
        self.manifest.skills_dir(&self.path)
    }

    /// エージェントディレクトリのパスを取得
    pub fn agents_dir(&self) -> PathBuf {
        self.manifest.agents_dir(&self.path)
    }

    /// コマンドディレクトリのパスを取得
    pub fn commands_dir(&self) -> PathBuf {
        self.manifest.commands_dir(&self.path)
    }

    /// インストラクションディレクトリのパスを取得
    pub fn instructions_dir(&self) -> PathBuf {
        self.manifest.instructions_dir(&self.path)
    }
}

/// 特定のターゲットにプラグインのコンポーネントをデプロイ
///
/// デプロイしたコンポーネント数を返す。
fn deploy_plugin_to_target(
    ctx: &TargetContext,
    plugin: &PluginDeployment,
) -> std::result::Result<usize, String> {
    let mut deployed_count = 0usize;

    // スキルをデプロイ
    if ctx.target.supports(ComponentKind::Skill) {
        let skills_dir = plugin.skills_dir();
        if skills_dir.exists() {
            for entry in skills_dir.read_dir().map_err(|e| e.to_string())? {
                let entry = entry.map_err(|e| e.to_string())?;
                if entry.path().is_dir() {
                    let name = entry.file_name().to_string_lossy().to_string();
                    let source_path = entry.path();
                    if deploy_component(ctx, &plugin.origin, ComponentKind::Skill, &name, &source_path)? {
                        deployed_count += 1;
                    }
                }
            }
        }
    }

    // エージェントをデプロイ
    if ctx.target.supports(ComponentKind::Agent) {
        let agents_path = plugin.agents_dir();
        if agents_path.exists() {
            if agents_path.is_file() {
                // 単一ファイルの場合
                if let Some(name) = agents_path.file_stem() {
                    let name = name.to_string_lossy().to_string();
                    if deploy_component(ctx, &plugin.origin, ComponentKind::Agent, &name, &agents_path)? {
                        deployed_count += 1;
                    }
                }
            } else {
                // ディレクトリの場合
                for entry in agents_path.read_dir().map_err(|e| e.to_string())? {
                    let entry = entry.map_err(|e| e.to_string())?;
                    let path = entry.path();
                    if path.is_file() {
                        if let Some(name) = path.file_stem() {
                            let name = name.to_string_lossy().to_string();
                            if deploy_component(ctx, &plugin.origin, ComponentKind::Agent, &name, &path)? {
                                deployed_count += 1;
                            }
                        }
                    }
                }
            }
        }
    }

    // コマンドをデプロイ
    if ctx.target.supports(ComponentKind::Command) {
        let commands_dir = plugin.commands_dir();
        if commands_dir.exists() {
            for entry in commands_dir.read_dir().map_err(|e| e.to_string())? {
                let entry = entry.map_err(|e| e.to_string())?;
                let path = entry.path();
                if path.is_file() {
                    if let Some(name) = path.file_stem() {
                        let name = name.to_string_lossy().to_string();
                        if deploy_component(ctx, &plugin.origin, ComponentKind::Command, &name, &path)? {
                            deployed_count += 1;
                        }
                    }
                }
            }
        }
    }

    // インストラクションをデプロイ
    if ctx.target.supports(ComponentKind::Instruction) {
        let instructions_dir = plugin.instructions_dir();
        if instructions_dir.exists() {
            for entry in instructions_dir.read_dir().map_err(|e| e.to_string())? {
                let entry = entry.map_err(|e| e.to_string())?;
                let path = entry.path();
                if path.is_file() {
                    if let Some(name) = path.file_stem() {
                        let name = name.to_string_lossy().to_string();
                        if deploy_component(ctx, &plugin.origin, ComponentKind::Instruction, &name, &path)?
                        {
                            deployed_count += 1;
                        }
                    }
                }
            }
        }
    }

    Ok(deployed_count)
}

/// 単一コンポーネントをデプロイ
///
/// デプロイに成功したら true を返す。
fn deploy_component(
    ctx: &TargetContext,
    origin: &PluginOrigin,
    kind: ComponentKind,
    name: &str,
    source_path: &Path,
) -> std::result::Result<bool, String> {
    let context = PlacementContext {
        component: ComponentRef::new(kind, name),
        origin,
        scope: PlacementScope(Scope::Project),
        project: ProjectContext::new(ctx.project_root),
    };

    // 配置先を取得
    let target_path = match ctx.target.placement_location(&context) {
        Some(location) => location.into_path(),
        None => return Ok(false), // サポートしていない場合はスキップ
    };

    // デプロイを実行
    let deployment = ComponentDeployment::builder()
        .kind(kind)
        .name(name)
        .scope(Scope::Project)
        .source_path(source_path)
        .target_path(target_path)
        .build()
        .map_err(|e| e.to_string())?;

    deployment.execute().map_err(|e| e.to_string())?;
    Ok(true)
}

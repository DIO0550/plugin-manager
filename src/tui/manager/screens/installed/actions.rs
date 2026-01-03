//! Installed タブのアクション実行
//!
//! Disable/Uninstall などのプラグイン操作を実行する。

use crate::component::ComponentKind;
use crate::component::Scope;
use crate::domain::placement::{ComponentRef, PlacementContext, PlacementScope, ProjectContext};
use crate::plugin::PluginCache;
use crate::target::{all_targets, PluginOrigin, Target};
use std::env;
use std::path::Path;

/// アクション実行結果
#[derive(Debug)]
pub enum ActionResult {
    /// 成功
    Success,
    /// エラー
    Error(String),
}

/// プラグインを Disable（デプロイ先から削除、キャッシュは残す）
pub fn disable_plugin(plugin_name: &str, marketplace: Option<&str>) -> ActionResult {
    let cache = match PluginCache::new() {
        Ok(c) => c,
        Err(e) => return ActionResult::Error(format!("Failed to access cache: {}", e)),
    };

    // プラグインがキャッシュに存在するか確認
    if !cache.is_cached(marketplace, plugin_name) {
        return ActionResult::Error(format!("Plugin '{}' not found in cache", plugin_name));
    }

    // マニフェストを読み込んでコンポーネント情報を取得
    let manifest = match cache.load_manifest(marketplace, plugin_name) {
        Ok(m) => m,
        Err(e) => return ActionResult::Error(format!("Failed to load manifest: {}", e)),
    };

    // プラグインのオリジン情報を作成
    // 重要: デプロイ時はマニフェストの name を使うので、削除時も同じ値を使う
    let origin = match marketplace {
        Some(mp) => PluginOrigin::from_marketplace(mp, &manifest.name),
        None => PluginOrigin::from_marketplace("github", &manifest.name),
    };

    // プロジェクトルート（カレントディレクトリ）
    let project_root = env::current_dir().unwrap_or_else(|_| ".".into());

    // 各ターゲットに対してコンポーネントを削除
    let mut errors = Vec::new();
    let targets = all_targets();

    for target in &targets {
        if let Err(e) = remove_plugin_from_target(
            target.as_ref(),
            &origin,
            &manifest,
            &cache,
            marketplace,
            plugin_name,
            &project_root,
        ) {
            errors.push(format!("{}: {}", target.name(), e));
        }
    }

    if errors.is_empty() {
        ActionResult::Success
    } else {
        ActionResult::Error(errors.join("; "))
    }
}

/// プラグインを Uninstall（デプロイ先 + キャッシュ削除）
pub fn uninstall_plugin(plugin_name: &str, marketplace: Option<&str>) -> ActionResult {
    // まずデプロイ先から削除
    if let ActionResult::Error(e) = disable_plugin(plugin_name, marketplace) {
        return ActionResult::Error(e);
    }

    // キャッシュから削除
    let cache = match PluginCache::new() {
        Ok(c) => c,
        Err(e) => return ActionResult::Error(format!("Failed to access cache: {}", e)),
    };

    if let Err(e) = cache.remove(marketplace, plugin_name) {
        return ActionResult::Error(format!("Failed to remove from cache: {}", e));
    }

    ActionResult::Success
}

/// 特定のターゲットからプラグインのコンポーネントを削除
fn remove_plugin_from_target(
    target: &dyn Target,
    origin: &PluginOrigin,
    manifest: &crate::plugin::PluginManifest,
    cache: &PluginCache,
    marketplace: Option<&str>,
    plugin_name: &str,
    project_root: &Path,
) -> Result<(), String> {
    let plugin_path = cache.plugin_path(marketplace, plugin_name);

    // スキルを削除
    if target.supports(ComponentKind::Skill) {
        let skills_dir = manifest.skills_dir(&plugin_path);
        if skills_dir.exists() {
            for entry in skills_dir.read_dir().map_err(|e| e.to_string())? {
                let entry = entry.map_err(|e| e.to_string())?;
                if entry.path().is_dir() {
                    let name = entry.file_name().to_string_lossy().to_string();
                    remove_component(target, origin, ComponentKind::Skill, &name, project_root)?;
                }
            }
        }
    }

    // エージェントを削除
    if target.supports(ComponentKind::Agent) {
        let agents_path = manifest.agents_dir(&plugin_path);
        if agents_path.exists() {
            if agents_path.is_file() {
                // 単一ファイルの場合
                if let Some(name) = agents_path.file_stem() {
                    let name = name.to_string_lossy().to_string();
                    remove_component(target, origin, ComponentKind::Agent, &name, project_root)?;
                }
            } else {
                // ディレクトリの場合
                for entry in agents_path.read_dir().map_err(|e| e.to_string())? {
                    let entry = entry.map_err(|e| e.to_string())?;
                    let path = entry.path();
                    if path.is_file() {
                        if let Some(name) = path.file_stem() {
                            let name = name.to_string_lossy().to_string();
                            remove_component(
                                target,
                                origin,
                                ComponentKind::Agent,
                                &name,
                                project_root,
                            )?;
                        }
                    }
                }
            }
        }
    }

    // コマンドを削除
    if target.supports(ComponentKind::Command) {
        let commands_dir = manifest.commands_dir(&plugin_path);
        if commands_dir.exists() {
            for entry in commands_dir.read_dir().map_err(|e| e.to_string())? {
                let entry = entry.map_err(|e| e.to_string())?;
                let path = entry.path();
                if path.is_file() {
                    if let Some(name) = path.file_stem() {
                        let name = name.to_string_lossy().to_string();
                        remove_component(
                            target,
                            origin,
                            ComponentKind::Command,
                            &name,
                            project_root,
                        )?;
                    }
                }
            }
        }
    }

    // インストラクションを削除
    if target.supports(ComponentKind::Instruction) {
        let instructions_dir = manifest.instructions_dir(&plugin_path);
        if instructions_dir.exists() {
            for entry in instructions_dir.read_dir().map_err(|e| e.to_string())? {
                let entry = entry.map_err(|e| e.to_string())?;
                let path = entry.path();
                if path.is_file() {
                    if let Some(name) = path.file_stem() {
                        let name = name.to_string_lossy().to_string();
                        remove_component(
                            target,
                            origin,
                            ComponentKind::Instruction,
                            &name,
                            project_root,
                        )?;
                    }
                }
            }
        }
    }

    Ok(())
}

/// 単一コンポーネントを削除
fn remove_component(
    target: &dyn Target,
    origin: &PluginOrigin,
    kind: ComponentKind,
    name: &str,
    project_root: &Path,
) -> Result<(), String> {
    let context = PlacementContext {
        component: ComponentRef::new(kind, name),
        origin,
        scope: PlacementScope(Scope::Project),
        project: ProjectContext::new(project_root),
    };

    // 削除を試みる（存在しなくてもエラーにしない）
    if let Err(e) = target.remove(&context) {
        // ファイルが存在しない場合は無視
        let err_str = e.to_string();
        if !err_str.contains("not found") && !err_str.contains("No such file") {
            return Err(err_str);
        }
    }

    Ok(())
}

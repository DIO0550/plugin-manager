//! プラグイン更新ユースケース
//!
//! GitHub APIを使用して最新のコミットSHAを取得し、インストール済みSHAと比較して
//! 差分がある場合に再ダウンロード・再デプロイを行う。

use crate::application::enable_plugin;
use crate::error::{PlmError, Result};
use crate::host::{HostClient, HostClientFactory, HostKind};
use crate::http::with_retry;
use crate::marketplace::{MarketplaceCache, MarketplaceRegistry, PluginSource as MpPluginSource};
use crate::plugin::lifecycle::plugin_resolver::{find_by_plugin_name, ResolvedPlugin};
use crate::plugin::version::needs_update;
use crate::plugin::{meta, PackageCacheAccess, PluginMeta};
use crate::repo::{self, Repo};
use std::path::Path;

/// 更新ステータス
#[derive(Debug, Clone)]
pub enum UpdateStatus {
    /// 更新完了
    Updated {
        from_sha: Option<String>,
        to_sha: String,
    },
    /// 既に最新
    AlreadyUpToDate,
    /// 更新失敗
    Failed,
    /// スキップ
    Skipped { reason: String },
}

/// 更新結果
#[derive(Debug, Clone)]
pub struct UpdateResult {
    pub plugin_name: String,
    pub marketplace: String,
    pub status: UpdateStatus,
    pub error: Option<String>,
    /// 再デプロイに成功したターゲット
    pub deployed_targets: Vec<String>,
    /// 再デプロイに失敗したターゲット
    pub failed_targets: Vec<String>,
}

impl UpdateResult {
    /// 更新成功
    ///
    /// # Arguments
    ///
    /// * `name` - Plugin name.
    /// * `from` - Commit SHA before the update, or `None` when unrecorded.
    /// * `to` - Commit SHA after the update.
    /// * `deployed` - Targets that were redeployed successfully.
    /// * `failed` - Targets that failed to redeploy.
    pub fn updated(
        name: &str,
        from: Option<String>,
        to: String,
        deployed: Vec<String>,
        failed: Vec<String>,
    ) -> Self {
        Self {
            plugin_name: name.to_string(),
            marketplace: "github".to_string(),
            status: UpdateStatus::Updated {
                from_sha: from,
                to_sha: to,
            },
            error: None,
            deployed_targets: deployed,
            failed_targets: failed,
        }
    }

    /// 既に最新
    ///
    /// # Arguments
    ///
    /// * `name` - Plugin name.
    pub fn up_to_date(name: &str) -> Self {
        Self {
            plugin_name: name.to_string(),
            marketplace: "github".to_string(),
            status: UpdateStatus::AlreadyUpToDate,
            error: None,
            deployed_targets: vec![],
            failed_targets: vec![],
        }
    }

    /// 更新失敗
    ///
    /// # Arguments
    ///
    /// * `name` - Plugin name.
    /// * `error` - Error message describing the failure.
    pub fn failed(name: &str, error: String) -> Self {
        Self {
            plugin_name: name.to_string(),
            marketplace: "github".to_string(),
            status: UpdateStatus::Failed,
            error: Some(error),
            deployed_targets: vec![],
            failed_targets: vec![],
        }
    }

    /// スキップ
    ///
    /// # Arguments
    ///
    /// * `name` - Plugin name.
    /// * `reason` - Reason the update was skipped.
    pub fn skipped(name: &str, reason: String) -> Self {
        Self {
            plugin_name: name.to_string(),
            marketplace: "github".to_string(),
            status: UpdateStatus::Skipped { reason },
            error: None,
            deployed_targets: vec![],
            failed_targets: vec![],
        }
    }
}

/// リポジトリ情報を復元
///
/// 優先順位:
/// 1. meta.source_repo（owner/repo形式）
/// 2. プラグイン名からフォールバック（owner--repo形式）
///
/// # Arguments
///
/// * `meta` - Plugin metadata used as the primary source of repository info.
/// * `plugin_name` - Plugin name, interpreted as `owner--repo` during fallback.
/// * `git_ref` - Target Git reference to associate with the restored repo.
fn restore_repo(
    meta: &PluginMeta,
    plugin_name: &str,
    git_ref: &str,
) -> std::result::Result<Repo, String> {
    if let Some((owner, name)) = meta.get_source_repo() {
        let repo = Repo::new(HostKind::GitHub, owner, name, Some(git_ref.to_string()));
        return Ok(repo);
    }

    let parts: Vec<&str> = plugin_name.split("--").collect();
    if parts.len() == 2 {
        let repo = Repo::new(
            HostKind::GitHub,
            parts[0],
            parts[1],
            Some(git_ref.to_string()),
        );
        Ok(repo)
    } else {
        Err(format!(
            "Cannot determine repository from plugin name: {}",
            plugin_name
        ))
    }
}

/// `MarketplaceCache.source` (`"github:{owner}/{name}"` 形式) と `PluginSource` から
/// archive 取得用の `Repo` を再構築する。
///
/// `git_ref` は呼び出し側で必ず渡す（ローカル `PluginMeta.git_ref` を `unwrap_or("HEAD")` 等で解決）。
/// `MarketplaceCache.source` は ref を持たないため、`Repo` に ref を詰めることで
/// `download_archive_with_sha` がデフォルトブランチへフォールバックするのを防ぐ。
///
/// # Arguments
///
/// * `mp_source` - Marketplace source string in `github:owner/repo` form.
/// * `plugin_source` - Plugin source descriptor from the marketplace cache entry.
/// * `git_ref` - Git reference to embed into the resulting `Repo`.
fn parse_repo_from_mp_source(
    mp_source: &str,
    plugin_source: &MpPluginSource,
    git_ref: Option<&str>,
) -> Result<Repo> {
    // External は `repo` 文字列を `repo::from_url` で解析（`owner/repo@tag` や HTTPS URL に対応）。
    // 取り出した owner / name を使って、ローカル `PluginMeta.git_ref` で上書きした `Repo` を返す。
    // Local は marketplace 自体の repo を参照（mp_source の "github:owner/name" を strip して再構築）。
    let parsed = match plugin_source {
        MpPluginSource::External { repo, .. } => repo::from_url(repo)?,
        MpPluginSource::Local(_) => {
            let stripped = mp_source.strip_prefix("github:").unwrap_or(mp_source);
            repo::from_url(stripped)?
        }
    };
    Ok(Repo::new(
        parsed.host(),
        parsed.owner().to_string(),
        parsed.name().to_string(),
        git_ref
            .map(String::from)
            .or_else(|| parsed.git_ref().map(String::from)),
    ))
}

/// `update_plugin` の入力 (`plugin_input`, `marketplace_hint`) を `ResolvedPlugin` に解決する。
///
/// 解決順序:
/// 1. `marketplace_hint` がある場合: `cache.is_cached(Some(hint), plugin_input)` で確認
/// 2. `marketplace_hint` がない場合: `cache.list()` を全走査して `cache_id == plugin_input` を集める
///    - 1 件 → 確定 / 複数件 → AmbiguousPluginName
/// 3. cache_id でヒットしなかった場合のみ display name 解決にフォールバック
///
/// # Arguments
///
/// * `cache` - Package cache accessor.
/// * `plugin_input` - User-supplied plugin identifier (cache_id or display name).
/// * `marketplace_hint` - Optional marketplace hint to disambiguate.
fn resolve_update_target(
    cache: &dyn PackageCacheAccess,
    plugin_input: &str,
    marketplace_hint: Option<&str>,
) -> Result<ResolvedPlugin> {
    // (1)/(2) cache_id 完全一致
    let cache_id_matches: Vec<(Option<String>, String)> = match marketplace_hint {
        Some(hint) => {
            if cache.is_cached(Some(hint), plugin_input) {
                vec![(Some(hint.to_string()), plugin_input.to_string())]
            } else {
                vec![]
            }
        }
        None => cache
            .list()?
            .into_iter()
            .filter(|(_m, cid)| cid == plugin_input)
            .collect(),
    };

    if cache_id_matches.len() == 1 {
        let (market, cache_id) = cache_id_matches.into_iter().next().unwrap();
        return load_resolved(cache, market, cache_id);
    } else if cache_id_matches.len() >= 2 {
        let mut candidates: Vec<String> = cache_id_matches
            .iter()
            .map(|(m, cid)| match m {
                Some(mk) => format!("{}@{}", cid, mk),
                None => cid.clone(),
            })
            .collect();
        candidates.sort();
        return Err(PlmError::AmbiguousPluginName {
            name: plugin_input.to_string(),
            candidates,
        });
    }

    // (3) display name フォールバック
    match find_by_plugin_name(cache, plugin_input, marketplace_hint)? {
        Some(r) => Ok(r),
        None => match marketplace_hint {
            Some(hint) => Err(PlmError::PluginNotFound(format!(
                "{}@{}",
                plugin_input, hint
            ))),
            None => Err(PlmError::PluginNotFound(plugin_input.to_string())),
        },
    }
}

fn load_resolved(
    cache: &dyn PackageCacheAccess,
    market: Option<String>,
    cache_id: String,
) -> Result<ResolvedPlugin> {
    let pkg = cache.load_package(market.as_deref(), &cache_id)?;
    let plugin_path = cache.plugin_path(market.as_deref(), &cache_id);
    let package_meta = meta::load_meta(&plugin_path).unwrap_or_default();
    Ok(ResolvedPlugin {
        marketplace: market,
        cache_id,
        display_name: pkg.name.clone(),
        package: pkg,
        package_meta,
    })
}

/// 単一プラグインの更新
///
/// # Arguments
///
/// * `cache` - Package cache accessor for the plugin.
/// * `plugin_input` - Cache ID or display name of the plugin.
/// * `marketplace_hint` - Optional marketplace hint to disambiguate.
/// * `project_root` - Project root path used for redeployment.
/// * `target_filter` - When `Some`, only redeploy to this single target.
pub async fn update_plugin(
    cache: &dyn PackageCacheAccess,
    plugin_input: &str,
    marketplace_hint: Option<&str>,
    project_root: &Path,
    target_filter: Option<&str>,
) -> UpdateResult {
    let resolved = match resolve_update_target(cache, plugin_input, marketplace_hint) {
        Ok(r) => r,
        Err(PlmError::AmbiguousPluginName { name, candidates }) => {
            return UpdateResult::failed(
                &name,
                format!(
                    "Ambiguous plugin name (matches: {}). \
                     Specify with marketplace, e.g. `{}@<marketplace>`.",
                    candidates.join(", "),
                    name,
                ),
            );
        }
        Err(PlmError::PluginNotFound(_)) => {
            return UpdateResult::failed(plugin_input, "Plugin not found in cache".to_string());
        }
        Err(e) => return UpdateResult::failed(plugin_input, e.to_string()),
    };

    if let Some(market) = resolved.marketplace.clone() {
        update_marketplace_plugin(cache, &market, &resolved, project_root, target_filter).await
    } else {
        update_github_plugin(cache, &resolved, project_root, target_filter).await
    }
}

/// 直接 GitHub 経路の安全更新
async fn update_github_plugin(
    cache: &dyn PackageCacheAccess,
    resolved: &ResolvedPlugin,
    project_root: &Path,
    target_filter: Option<&str>,
) -> UpdateResult {
    let plugin_meta = &resolved.package_meta;
    let cache_id = resolved.cache_id.as_str();
    let display_name = resolved.display_name.as_str();

    if !plugin_meta.is_github() {
        return UpdateResult::skipped(display_name, "Not a GitHub plugin".to_string());
    }

    let git_ref = plugin_meta.git_ref.as_deref().unwrap_or("HEAD");
    let current_sha = plugin_meta.commit_sha.clone();

    let repo = match restore_repo(plugin_meta, cache_id, git_ref) {
        Ok(r) => r,
        Err(e) => return UpdateResult::failed(display_name, e),
    };

    let factory = HostClientFactory::with_defaults();
    let client = factory.create(HostKind::GitHub);

    let latest_sha = match with_retry(|| client.get_commit_sha(&repo, git_ref), 3).await {
        Ok(sha) => sha,
        Err(e) => {
            return UpdateResult::failed(display_name, format!("Failed to get latest SHA: {}", e));
        }
    };

    if current_sha.as_deref() == Some(&latest_sha) {
        return UpdateResult::up_to_date(display_name);
    }

    if current_sha.is_none() {
        eprintln!(
            "Warning: No commit SHA recorded for '{}'. Forcing update.",
            display_name
        );
    }

    do_safe_update(
        cache,
        None,
        cache_id,
        display_name,
        plugin_meta,
        client.as_ref(),
        &repo,
        None,
        project_root,
        target_filter,
    )
    .await
}

/// marketplace 経路の安全更新
async fn update_marketplace_plugin(
    cache: &dyn PackageCacheAccess,
    marketplace: &str,
    resolved: &ResolvedPlugin,
    project_root: &Path,
    target_filter: Option<&str>,
) -> UpdateResult {
    let display_name = resolved.display_name.as_str();
    let cache_id = resolved.cache_id.as_str();
    let plugin_meta = &resolved.package_meta;

    let registry = match MarketplaceRegistry::new() {
        Ok(r) => r,
        Err(e) => return UpdateResult::failed(display_name, e.to_string()),
    };
    let mp_cache = match registry.get(marketplace) {
        Ok(Some(c)) => c,
        Ok(None) => {
            return UpdateResult::failed(
                display_name,
                format!("Marketplace not found: {}", marketplace),
            );
        }
        Err(e) => return UpdateResult::failed(display_name, e.to_string()),
    };
    let entry = match mp_cache.plugins.iter().find(|p| p.name == cache_id) {
        Some(e) => e.clone(),
        None => {
            return UpdateResult::failed(
                display_name,
                format!("Plugin entry not found in marketplace: {}", cache_id),
            );
        }
    };

    let git_ref_owned = plugin_meta
        .git_ref
        .clone()
        .unwrap_or_else(|| "HEAD".to_string());
    let git_ref = git_ref_owned.as_str();

    let repo = match parse_repo_from_mp_source(&mp_cache.source, &entry.source, Some(git_ref)) {
        Ok(r) => r,
        Err(e) => return UpdateResult::failed(display_name, e.to_string()),
    };

    let factory = HostClientFactory::with_defaults();
    let client = factory.create(HostKind::GitHub);

    let latest_sha = match with_retry(|| client.get_commit_sha(&repo, git_ref), 3).await {
        Ok(sha) => sha,
        Err(e) => {
            return UpdateResult::failed(display_name, format!("Failed to get latest SHA: {}", e));
        }
    };

    let current_sha = plugin_meta.commit_sha.clone();
    if current_sha.as_deref() == Some(&latest_sha) {
        return UpdateResult::up_to_date(display_name);
    }
    if current_sha.is_none() {
        eprintln!(
            "Warning: No commit SHA recorded for '{}'. Forcing update.",
            display_name
        );
    }

    let source_path = match &entry.source {
        MpPluginSource::Local(p) => Some(p.clone()),
        MpPluginSource::External { .. } => None,
    };

    do_safe_update(
        cache,
        Some(marketplace),
        cache_id,
        display_name,
        plugin_meta,
        client.as_ref(),
        &repo,
        source_path.as_deref(),
        project_root,
        target_filter,
    )
    .await
}

/// 安全更新フローの共通実装
///
/// `backup → fetch → atomic_update_with_source_path → redeploy → meta merge → write_meta → remove_backup`
/// の順で実行し、各失敗ポイントで `cache.restore` による rollback を行う。
#[allow(clippy::too_many_arguments)]
async fn do_safe_update(
    cache: &dyn PackageCacheAccess,
    marketplace: Option<&str>,
    cache_id: &str,
    display_name: &str,
    old_meta: &PluginMeta,
    client: &dyn HostClient,
    repo: &Repo,
    source_path: Option<&str>,
    project_root: &Path,
    target_filter: Option<&str>,
) -> UpdateResult {
    let current_sha = old_meta.commit_sha.clone();
    let git_ref_owned = old_meta
        .git_ref
        .clone()
        .unwrap_or_else(|| "HEAD".to_string());
    let git_ref = git_ref_owned.as_str();

    println!("  Creating backup...");
    if let Err(e) = cache.backup(marketplace, cache_id) {
        return UpdateResult::failed(display_name, format!("Backup failed: {}", e));
    }

    println!("  Downloading...");
    let (archive, _archive_git_ref, archive_sha) =
        match with_retry(|| client.download_archive_with_sha(repo), 3).await {
            Ok(triple) => triple,
            Err(e) => {
                let _ = cache.restore(marketplace, cache_id);
                return UpdateResult::failed(display_name, format!("Download failed: {}", e));
            }
        };

    println!("  Extracting...");
    let plugin_path =
        match cache.atomic_update_with_source_path(marketplace, cache_id, &archive, source_path) {
            Ok(p) => p,
            Err(e) => {
                let _ = cache.restore(marketplace, cache_id);
                return UpdateResult::failed(display_name, format!("Extraction failed: {}", e));
            }
        };

    println!("  Deploying...");
    let enabled = old_meta.enabled_targets();
    let targets: Vec<&str> = match target_filter {
        Some(f) => enabled.into_iter().filter(|t| *t == f).collect(),
        None => enabled,
    };
    let (deployed, failed) =
        redeploy_to_targets(cache, cache_id, marketplace, &targets, project_root);

    let mut new_meta = old_meta.clone();
    new_meta.set_git_info(git_ref, &archive_sha);
    for t in &failed {
        new_meta.set_status(t, "disabled");
    }
    if let Err(e) = meta::write_meta(&plugin_path, &new_meta) {
        let _ = cache.restore(marketplace, cache_id);
        return UpdateResult::failed(display_name, format!("Failed to write metadata: {}", e));
    }

    let _ = cache.remove_backup(marketplace, cache_id);

    UpdateResult::updated(display_name, current_sha, archive_sha, deployed, failed)
}

/// ターゲットへの再デプロイ
///
/// # Arguments
///
/// * `cache` - Package cache accessor for the plugin.
/// * `plugin_name` - Plugin name being redeployed.
/// * `marketplace` - Marketplace name (`None` falls back to `"github"`).
/// * `targets` - Target names to redeploy to.
/// * `project_root` - Project root path used for redeployment.
fn redeploy_to_targets(
    cache: &dyn PackageCacheAccess,
    plugin_name: &str,
    marketplace: Option<&str>,
    targets: &[&str],
    project_root: &Path,
) -> (Vec<String>, Vec<String>) {
    let mut deployed = Vec::new();
    let mut failed = Vec::new();

    for target in targets {
        let result = enable_plugin(
            cache,
            plugin_name,
            marketplace.or(Some("github")),
            project_root,
            Some(target),
        );
        if result.success {
            deployed.push(target.to_string());
        } else {
            failed.push(target.to_string());
        }
    }

    (deployed, failed)
}

/// 全プラグインの一括更新
///
/// キャッシュ内の全プラグイン（直接 GitHub / marketplace 経由）を走査し、
/// 各プラグインの commit SHA を比較して差分があるものを更新する。
///
/// # Arguments
///
/// * `cache` - Package cache accessor used to enumerate and update plugins.
/// * `project_root` - Project root path used for redeployment.
/// * `target_filter` - When `Some`, only redeploy to this single target.
pub async fn update_all_plugins(
    cache: &dyn PackageCacheAccess,
    project_root: &Path,
    target_filter: Option<&str>,
) -> Vec<UpdateResult> {
    let plugins = match cache.list() {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Error: Failed to list plugins: {}", e);
            return vec![];
        }
    };

    if plugins.is_empty() {
        println!("No plugins installed.");
        return vec![];
    }

    println!("Checking for updates...");
    let factory = HostClientFactory::with_defaults();
    let client = factory.create(HostKind::GitHub);

    // marketplace 別に MarketplaceCache を 1 度だけロード
    let registry = match MarketplaceRegistry::new() {
        Ok(r) => Some(r),
        Err(e) => {
            eprintln!("Warning: Failed to load marketplace registry: {}", e);
            None
        }
    };

    let mut mp_caches: std::collections::HashMap<String, MarketplaceCache> =
        std::collections::HashMap::new();
    if let Some(reg) = &registry {
        let unique_markets: std::collections::HashSet<String> = plugins
            .iter()
            .filter_map(|(m, _)| m.clone())
            .filter(|m| m != "github")
            .collect();
        for m in unique_markets {
            if let Ok(Some(c)) = reg.get(&m) {
                mp_caches.insert(m, c);
            }
        }
    }

    let mut results = Vec::new();
    let mut up_to_date_count = 0usize;
    let mut error_count = 0usize;
    let mut updates_to_do: Vec<(Option<String>, String, String, ResolvedPlugin)> = Vec::new();

    for (marketplace, cache_id) in &plugins {
        let plugin_path = cache.plugin_path(marketplace.as_deref(), cache_id);
        let pkg = match cache.load_package(marketplace.as_deref(), cache_id) {
            Ok(p) => p,
            Err(e) => {
                error_count += 1;
                eprintln!("  {}: Failed to load ({})", cache_id, e);
                continue;
            }
        };
        let display_name = pkg.name.clone();
        let plugin_meta = meta::load_meta(&plugin_path).unwrap_or_default();

        // marketplace 経由 plugin
        if let Some(market) = marketplace.as_deref() {
            if market == "github" {
                // 直接 GitHub install
                let result =
                    check_github_remote(&*client, &plugin_meta, cache_id, &display_name).await;
                match result {
                    RemoteCheck::UpToDate => {
                        up_to_date_count += 1;
                        results.push(UpdateResult::up_to_date(&display_name));
                    }
                    RemoteCheck::NeedsUpdate(latest) => {
                        let resolved = ResolvedPlugin {
                            marketplace: marketplace.clone(),
                            cache_id: cache_id.clone(),
                            display_name: display_name.clone(),
                            package: pkg,
                            package_meta: plugin_meta,
                        };
                        updates_to_do.push((
                            marketplace.clone(),
                            cache_id.clone(),
                            latest,
                            resolved,
                        ));
                    }
                    RemoteCheck::Failed(msg) => {
                        error_count += 1;
                        eprintln!("  {}: Failed to check ({})", display_name, msg);
                    }
                    RemoteCheck::Skipped(_reason) => {
                        // GitHub 以外はあり得ないが念のため
                    }
                }
                continue;
            }

            let mp_cache = match mp_caches.get(market) {
                Some(c) => c,
                None => {
                    error_count += 1;
                    eprintln!("  {}: Marketplace not found: {}", display_name, market);
                    continue;
                }
            };
            let entry = match mp_cache.plugins.iter().find(|p| p.name == *cache_id) {
                Some(e) => e,
                None => {
                    // marketplace から消えたものは up_to_date 扱い
                    up_to_date_count += 1;
                    results.push(UpdateResult::up_to_date(&display_name));
                    continue;
                }
            };
            let git_ref = plugin_meta.git_ref.as_deref().unwrap_or("HEAD");
            let repo =
                match parse_repo_from_mp_source(&mp_cache.source, &entry.source, Some(git_ref)) {
                    Ok(r) => r,
                    Err(e) => {
                        error_count += 1;
                        eprintln!("  {}: {}", display_name, e);
                        continue;
                    }
                };
            let latest_sha = match with_retry(|| client.get_commit_sha(&repo, git_ref), 3).await {
                Ok(s) => s,
                Err(e) => {
                    error_count += 1;
                    eprintln!("  {}: Failed to check ({})", display_name, e);
                    continue;
                }
            };

            if !needs_update(plugin_meta.commit_sha.as_deref(), &latest_sha) {
                up_to_date_count += 1;
                results.push(UpdateResult::up_to_date(&display_name));
                continue;
            }

            let resolved = ResolvedPlugin {
                marketplace: marketplace.clone(),
                cache_id: cache_id.clone(),
                display_name: display_name.clone(),
                package: pkg,
                package_meta: plugin_meta,
            };
            updates_to_do.push((marketplace.clone(), cache_id.clone(), latest_sha, resolved));
            continue;
        }

        // marketplace == None 経路
        let result = check_github_remote(&*client, &plugin_meta, cache_id, &display_name).await;
        match result {
            RemoteCheck::UpToDate => {
                up_to_date_count += 1;
                results.push(UpdateResult::up_to_date(&display_name));
            }
            RemoteCheck::NeedsUpdate(latest) => {
                let resolved = ResolvedPlugin {
                    marketplace: None,
                    cache_id: cache_id.clone(),
                    display_name: display_name.clone(),
                    package: pkg,
                    package_meta: plugin_meta,
                };
                updates_to_do.push((None, cache_id.clone(), latest, resolved));
            }
            RemoteCheck::Skipped(reason) => {
                results.push(UpdateResult::skipped(&display_name, reason));
            }
            RemoteCheck::Failed(msg) => {
                error_count += 1;
                eprintln!("  {}: Failed to check ({})", display_name, msg);
            }
        }
    }

    let update_count = updates_to_do.len();

    if update_count == 0 {
        println!(
            "\nAll {} plugin(s) are up to date.{}",
            up_to_date_count,
            if error_count > 0 {
                format!(" ({} could not be checked)", error_count)
            } else {
                String::new()
            }
        );
        return results;
    }

    println!(
        "\nUpdating {} plugin(s)... ({} up to date{})",
        update_count,
        up_to_date_count,
        if error_count > 0 {
            format!(", {} errors", error_count)
        } else {
            String::new()
        }
    );

    for (idx, (marketplace, _cache_id, _latest, resolved)) in updates_to_do.into_iter().enumerate()
    {
        println!(
            "\n[{}/{}] Updating {}...",
            idx + 1,
            update_count,
            resolved.display_name
        );

        let result = if let Some(market) = marketplace.as_deref() {
            update_marketplace_plugin(cache, market, &resolved, project_root, target_filter).await
        } else {
            update_github_plugin(cache, &resolved, project_root, target_filter).await
        };

        match &result.status {
            UpdateStatus::Updated { from_sha, to_sha } => {
                let from = from_sha.as_deref().unwrap_or("unknown");
                println!("  Updated: {} -> {}", from, to_sha);
            }
            UpdateStatus::Failed => {
                if let Some(e) = &result.error {
                    eprintln!("  Error: {}", e);
                }
            }
            _ => {}
        }
        results.push(result);
    }

    results
}

enum RemoteCheck {
    UpToDate,
    NeedsUpdate(String),
    Skipped(String),
    Failed(String),
}

async fn check_github_remote(
    client: &dyn HostClient,
    plugin_meta: &PluginMeta,
    cache_id: &str,
    display_name: &str,
) -> RemoteCheck {
    if !plugin_meta.is_github() {
        return RemoteCheck::Skipped("Not a GitHub plugin".to_string());
    }
    let git_ref = plugin_meta.git_ref.as_deref().unwrap_or("HEAD");
    let repo = match restore_repo(plugin_meta, cache_id, git_ref) {
        Ok(r) => r,
        Err(e) => return RemoteCheck::Failed(e),
    };
    let latest_sha = match with_retry(|| client.get_commit_sha(&repo, git_ref), 3).await {
        Ok(s) => s,
        Err(e) => return RemoteCheck::Failed(e.to_string()),
    };
    if !needs_update(plugin_meta.commit_sha.as_deref(), &latest_sha) {
        let _ = display_name;
        return RemoteCheck::UpToDate;
    }
    RemoteCheck::NeedsUpdate(latest_sha)
}

#[cfg(test)]
#[path = "update_test.rs"]
mod tests;

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
    /// 一括更新で他プラグインの失敗により更新前へロールバックされた
    /// (本来は更新成功していたが、batch all-or-nothing により巻き戻された)
    RolledBack,
}

/// 更新結果
#[derive(Debug, Clone)]
pub struct UpdateOutcome {
    pub plugin_name: String,
    pub marketplace: String,
    pub status: UpdateStatus,
    pub error: Option<String>,
    /// 再デプロイに成功したターゲット
    pub deployed_targets: Vec<String>,
    /// 再デプロイに失敗したターゲット
    pub failed_targets: Vec<String>,
}

impl UpdateOutcome {
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

    /// ロールバック済み（バッチ失敗により更新前へ巻き戻された）
    ///
    /// # Arguments
    ///
    /// * `name` - Plugin name.
    /// * `note` - 任意の補足（restore 失敗時の警告など）。None なら error なし。
    pub fn rolled_back(name: &str, note: Option<String>) -> Self {
        Self {
            plugin_name: name.to_string(),
            marketplace: "github".to_string(),
            status: UpdateStatus::RolledBack,
            error: note,
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
) -> UpdateOutcome {
    let resolved = match resolve_update_target(cache, plugin_input, marketplace_hint) {
        Ok(r) => r,
        Err(PlmError::AmbiguousPluginName { name, candidates }) => {
            return UpdateOutcome::failed(
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
            return UpdateOutcome::failed(plugin_input, "Plugin not found in cache".to_string());
        }
        Err(e) => return UpdateOutcome::failed(plugin_input, e.to_string()),
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
) -> UpdateOutcome {
    let plugin_meta = &resolved.package_meta;
    let cache_id = resolved.cache_id.as_str();
    let display_name = resolved.display_name.as_str();

    if !plugin_meta.is_github() {
        return UpdateOutcome::skipped(display_name, "Not a GitHub plugin".to_string());
    }

    let git_ref = plugin_meta.git_ref.as_deref().unwrap_or("HEAD");
    let current_sha = plugin_meta.commit_sha.clone();

    let repo = match restore_repo(plugin_meta, cache_id, git_ref) {
        Ok(r) => r,
        Err(e) => return UpdateOutcome::failed(display_name, e),
    };

    let factory = HostClientFactory::with_defaults();
    let client = factory.create(HostKind::GitHub);

    let latest_sha = match with_retry(|| client.get_commit_sha(&repo, git_ref), 3).await {
        Ok(sha) => sha,
        Err(e) => {
            return UpdateOutcome::failed(display_name, format!("Failed to get latest SHA: {}", e));
        }
    };

    if current_sha.as_deref() == Some(&latest_sha) {
        return UpdateOutcome::up_to_date(display_name);
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
) -> UpdateOutcome {
    let display_name = resolved.display_name.as_str();
    let cache_id = resolved.cache_id.as_str();
    let plugin_meta = &resolved.package_meta;

    let registry = match MarketplaceRegistry::new() {
        Ok(r) => r,
        Err(e) => return UpdateOutcome::failed(display_name, e.to_string()),
    };
    let mp_cache = match registry.get(marketplace) {
        Ok(Some(c)) => c,
        Ok(None) => {
            return UpdateOutcome::failed(
                display_name,
                format!("Marketplace not found: {}", marketplace),
            );
        }
        Err(e) => return UpdateOutcome::failed(display_name, e.to_string()),
    };
    let entry = match mp_cache.plugins.iter().find(|p| p.name == cache_id) {
        Some(e) => e.clone(),
        None => {
            return UpdateOutcome::failed(
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
        Err(e) => return UpdateOutcome::failed(display_name, e.to_string()),
    };

    let factory = HostClientFactory::with_defaults();
    let client = factory.create(HostKind::GitHub);

    let latest_sha = match with_retry(|| client.get_commit_sha(&repo, git_ref), 3).await {
        Ok(sha) => sha,
        Err(e) => {
            return UpdateOutcome::failed(display_name, format!("Failed to get latest SHA: {}", e));
        }
    };

    let current_sha = plugin_meta.commit_sha.clone();
    if current_sha.as_deref() == Some(&latest_sha) {
        return UpdateOutcome::up_to_date(display_name);
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
) -> UpdateOutcome {
    let current_sha = old_meta.commit_sha.clone();
    let git_ref_owned = old_meta
        .git_ref
        .clone()
        .unwrap_or_else(|| "HEAD".to_string());
    let git_ref = git_ref_owned.as_str();

    println!("  Creating backup...");
    if let Err(e) = cache.backup(marketplace, cache_id) {
        return UpdateOutcome::failed(display_name, format!("Backup failed: {}", e));
    }

    println!("  Downloading...");
    let (archive, _archive_git_ref, archive_sha) =
        match with_retry(|| client.download_archive_with_sha(repo), 3).await {
            Ok(triple) => triple,
            Err(e) => {
                let _ = cache.restore(marketplace, cache_id);
                return UpdateOutcome::failed(display_name, format!("Download failed: {}", e));
            }
        };

    println!("  Extracting...");
    let plugin_path =
        match cache.atomic_update_with_source_path(marketplace, cache_id, &archive, source_path) {
            Ok(p) => p,
            Err(e) => {
                let _ = cache.restore(marketplace, cache_id);
                return UpdateOutcome::failed(display_name, format!("Extraction failed: {}", e));
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
        return UpdateOutcome::failed(display_name, format!("Failed to write metadata: {}", e));
    }

    let _ = cache.remove_backup(marketplace, cache_id);

    UpdateOutcome::updated(display_name, current_sha, archive_sha, deployed, failed)
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

/// marketplace 解決の抽象（テストで差し替え可能にする）
pub(crate) trait MarketplaceResolver {
    /// 指定 marketplace の MarketplaceCache を返す（無ければ None）
    fn resolve(&self, marketplace: &str) -> Result<Option<MarketplaceCache>>;
}

/// 本番実装: registry を 1 度だけロードして保持し、以降の resolve はキャッシュ参照のみ。
///
/// `MarketplaceRegistry::new()` 失敗時は registry を None とし、resolve は常に None を返す
/// （= 現行 `update_all_plugins` の「registry ロード失敗は警告して続行」挙動と同一）。
pub(crate) struct RegistryResolver {
    registry: Option<MarketplaceRegistry>,
}

impl RegistryResolver {
    pub(crate) fn new() -> Self {
        match MarketplaceRegistry::new() {
            Ok(r) => Self { registry: Some(r) },
            Err(e) => {
                eprintln!("Warning: Failed to load marketplace registry: {}", e);
                Self { registry: None }
            }
        }
    }
}

impl MarketplaceResolver for RegistryResolver {
    fn resolve(&self, marketplace: &str) -> Result<Option<MarketplaceCache>> {
        match &self.registry {
            Some(r) => r.get(marketplace),
            None => Ok(None),
        }
    }
}

/// CHECK フェーズで確定した「更新すべき対象」1 件分
struct UpdateTarget {
    marketplace: Option<String>,
    cache_id: String,
    display_name: String,
    repo: Repo,
    source_path: Option<String>,
    old_meta: PluginMeta,
    git_ref: String,
}

/// PREPARE 成功後の staging 済み 1 件分
struct StagedUpdate {
    target: UpdateTarget,
    archive_sha: String,
}

/// prepare の集約結果
enum PrepareOutcome {
    /// 全件 staging + backup 成功
    AllStaged(Vec<StagedUpdate>),
    /// 1 件失敗。本番は非改変、staging/backup は破棄済み。
    /// 失敗・スキップした全 plugin の UpdateOutcome を含む。
    Aborted(Vec<UpdateOutcome>),
}

/// 全プラグインの一括更新（all-or-nothing バッチアトミック）
///
/// キャッシュ内の全プラグイン（直接 GitHub / marketplace 経由）を走査し、
/// 各プラグインの commit SHA を比較して差分があるものを 2 フェーズ（prepare → commit）で更新する。
/// prepare 中に 1 件でも失敗したら本番を一切改変せず、commit(swap) 中に失敗したら
/// swap 済みを含め全件を更新前へロールバックする。
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
) -> Vec<UpdateOutcome> {
    let factory = HostClientFactory::with_defaults();
    let client = factory.create(HostKind::GitHub);
    let resolver = RegistryResolver::new();
    update_all_plugins_with_deps(
        cache,
        client.as_ref(),
        &resolver,
        project_root,
        target_filter,
    )
    .await
}

/// 依存注入版本体（テストから直接呼ぶ）
pub(crate) async fn update_all_plugins_with_deps(
    cache: &dyn PackageCacheAccess,
    client: &dyn HostClient,
    resolver: &dyn MarketplaceResolver,
    project_root: &Path,
    target_filter: Option<&str>,
) -> Vec<UpdateOutcome> {
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

    // marketplace 別に MarketplaceCache を 1 度だけ解決
    let mut mp_caches: std::collections::HashMap<String, MarketplaceCache> =
        std::collections::HashMap::new();
    let unique_markets: std::collections::HashSet<String> = plugins
        .iter()
        .filter_map(|(m, _)| m.clone())
        .filter(|m| m != "github")
        .collect();
    for m in unique_markets {
        if let Ok(Some(c)) = resolver.resolve(&m) {
            mp_caches.insert(m, c);
        }
    }

    // CHECK フェーズ: 更新が必要な対象（targets）と、確定済み結果（up_to_date/skipped）を構築
    let CheckOutcome {
        targets,
        mut results,
        up_to_date_count,
        error_count,
    } = check_all(cache, client, &plugins, &mp_caches).await;

    let update_count = targets.len();

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

    // PREPARE フェーズ（本番非破壊）
    match prepare_all(cache, client, targets).await {
        PrepareOutcome::AllStaged(staged) => {
            // COMMIT フェーズ（swap → redeploy → meta、swap 失敗で ROLLBACK）
            results.extend(commit_all(cache, staged, project_root, target_filter));
        }
        PrepareOutcome::Aborted(failures) => {
            // prepare 失敗: 本番非改変。staged/未到達は RolledBack、当該は Failed。
            results.extend(failures);
        }
    }

    results
}

/// CHECK フェーズの集約結果
struct CheckOutcome {
    /// 更新が必要な対象
    targets: Vec<UpdateTarget>,
    /// 確定済み結果（up_to_date / skipped）。CHECK で弾かれた対象を含む。
    results: Vec<UpdateOutcome>,
    /// 最新だった件数（メッセージ表示用）
    up_to_date_count: usize,
    /// CHECK 中に load/解決/SHA 取得が失敗しスキップした件数（結果には含めない＝従来挙動）
    error_count: usize,
}

/// CHECK: 全 plugin を走査し、SHA 比較で更新が必要な対象を `UpdateTarget` に確定する。
///
/// marketplace 経由（`market != "github"`）と直接 GitHub（`None` または `"github"`）の 2 経路で
/// `repo` / `source_path` の解決方法のみが異なり、SHA 比較・`UpdateTarget` 構築の末尾処理は共通化する。
/// CHECK で `get_commit_sha` 等が失敗した plugin は従来どおり `error_count` でスキップし、
/// 他対象のアトミック処理の引き金にはしない。
async fn check_all(
    cache: &dyn PackageCacheAccess,
    client: &dyn HostClient,
    plugins: &[(Option<String>, String)],
    mp_caches: &std::collections::HashMap<String, MarketplaceCache>,
) -> CheckOutcome {
    let mut results = Vec::new();
    let mut up_to_date_count = 0usize;
    let mut error_count = 0usize;
    let mut targets: Vec<UpdateTarget> = Vec::new();

    for (marketplace, cache_id) in plugins {
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
        let git_ref = plugin_meta.git_ref.as_deref().unwrap_or("HEAD");

        // 経路ごとに repo / source_path を解決（早期 continue で skip / up_to_date / error を確定）
        let (repo, source_path) = if let Some(market) =
            marketplace.as_deref().filter(|m| *m != "github")
        {
            // marketplace 経由
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
                    results.push(UpdateOutcome::up_to_date(&display_name));
                    continue;
                }
            };
            let repo =
                match parse_repo_from_mp_source(&mp_cache.source, &entry.source, Some(git_ref)) {
                    Ok(r) => r,
                    Err(e) => {
                        error_count += 1;
                        eprintln!("  {}: {}", display_name, e);
                        continue;
                    }
                };
            let source_path = match &entry.source {
                MpPluginSource::Local(p) => Some(p.clone()),
                MpPluginSource::External { .. } => None,
            };
            (repo, source_path)
        } else {
            // 直接 GitHub 経路（marketplace == None もしくは "github"）
            if !plugin_meta.is_github() {
                results.push(UpdateOutcome::skipped(
                    &display_name,
                    "Not a GitHub plugin".to_string(),
                ));
                continue;
            }
            let repo = match restore_repo(&plugin_meta, cache_id, git_ref) {
                Ok(r) => r,
                Err(e) => {
                    error_count += 1;
                    eprintln!("  {}: Failed to check ({})", display_name, e);
                    continue;
                }
            };
            (repo, None)
        };

        // 共通末尾: SHA 取得 → needs_update 判定 → UpdateTarget 構築
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
            results.push(UpdateOutcome::up_to_date(&display_name));
            continue;
        }
        let git_ref_owned = git_ref.to_string();
        targets.push(UpdateTarget {
            marketplace: marketplace.clone(),
            cache_id: cache_id.clone(),
            display_name,
            repo,
            source_path,
            old_meta: plugin_meta,
            git_ref: git_ref_owned,
        });
    }

    CheckOutcome {
        targets,
        results,
        up_to_date_count,
        error_count,
    }
}

/// best-effort で backup を削除する。失敗してもバッチは継続するが、
/// `.backup` 残骸を放置して無言にしないよう警告を出す（運用者が部分クリーンアップを検知できる）。
fn cleanup_backup(cache: &dyn PackageCacheAccess, marketplace: Option<&str>, name: &str) {
    if let Err(e) = cache.remove_backup(marketplace, name) {
        eprintln!("Warning: Failed to remove backup for '{}': {}", name, e);
    }
}

/// best-effort で staging temp を破棄する。失敗時は `.temp` 残骸を無言にしないよう警告を出す。
fn cleanup_staged(cache: &dyn PackageCacheAccess, marketplace: Option<&str>, name: &str) {
    if let Err(e) = cache.discard_staged(marketplace, name) {
        eprintln!(
            "Warning: Failed to discard staged update for '{}': {}",
            name, e
        );
    }
}

/// PREPARE: 全件 backup + download + stage + 検証（本番非破壊）
///
/// 1 件でも失敗したら、それまでの staging/backup を全破棄して Aborted を返す。
async fn prepare_all(
    cache: &dyn PackageCacheAccess,
    client: &dyn HostClient,
    targets: Vec<UpdateTarget>,
) -> PrepareOutcome {
    let mut staged: Vec<StagedUpdate> = Vec::new();
    // 失敗時に「未到達の残り対象」を拾えるよう iterator を保持する。
    let mut iter = targets.into_iter();
    while let Some(target) = iter.next() {
        let mp = target.marketplace.as_deref();
        // 1. backup
        if let Err(e) = cache.backup(mp, &target.cache_id) {
            let failure = failed_for(&target, format!("Backup failed: {}", e));
            return abort(cache, staged, failure, iter);
        }
        // 2. download（本番非破壊）
        let (archive, _ref, archive_sha) =
            match with_retry(|| client.download_archive_with_sha(&target.repo), 3).await {
                Ok(t) => t,
                Err(e) => {
                    cleanup_backup(cache, mp, &target.cache_id);
                    let failure = failed_for(&target, format!("Download failed: {}", e));
                    return abort(cache, staged, failure, iter);
                }
            };
        // 3. stage（temp 展開 + plugin.json 検証, swap しない）
        if let Err(e) = cache.stage_from_archive(
            mp,
            &target.cache_id,
            &archive,
            target.source_path.as_deref(),
        ) {
            cleanup_backup(cache, mp, &target.cache_id);
            let failure = failed_for(&target, format!("Staging failed: {}", e));
            return abort(cache, staged, failure, iter);
        }
        staged.push(StagedUpdate {
            target,
            archive_sha,
        });
    }
    PrepareOutcome::AllStaged(staged)
}

/// 単一 UpdateTarget から失敗の UpdateOutcome を作る
fn failed_for(target: &UpdateTarget, msg: String) -> UpdateOutcome {
    UpdateOutcome::failed(&target.display_name, msg)
}

/// abort: それまでの staged 全件の staging/backup を破棄し、本番は触らずに失敗結果を返す。
///
/// `remaining`（失敗点より後ろの未到達対象）も「本番未改変のまま据え置き」＝ RolledBack として
/// 結果に必ず含める。これにより「1 プラグイン = 1 UpdateOutcome」「全件が結果に出る」不変条件を保つ。
fn abort(
    cache: &dyn PackageCacheAccess,
    staged: Vec<StagedUpdate>,
    failure: UpdateOutcome,
    remaining: impl Iterator<Item = UpdateTarget>,
) -> PrepareOutcome {
    let mut results = Vec::new();
    // すでに staging 済み（本来成功予定）→ RolledBack
    for s in &staged {
        let mp = s.target.marketplace.as_deref();
        cleanup_staged(cache, mp, &s.target.cache_id);
        cleanup_backup(cache, mp, &s.target.cache_id);
        results.push(UpdateOutcome::rolled_back(&s.target.display_name, None));
    }
    // 失敗した当該プラグイン → Failed
    results.push(failure);
    // 未到達の残り対象（本番未改変・更新前のまま）→ RolledBack
    for t in remaining {
        results.push(UpdateOutcome::rolled_back(&t.display_name, None));
    }
    PrepareOutcome::Aborted(results)
}

/// COMMIT: 全件 swap（Phase 1）→ 全件 redeploy + meta（Phase 2）。
///
/// swap（`commit_staged`）が唯一のロールバック可能フェーズであり、ここで失敗したら
/// 即 ROLLBACK する。**redeploy / meta 書き込みは全件の swap が成功してから**まとめて行う。
/// こうすることで、後続プラグインの swap 失敗による ROLLBACK が「いずれかのプラグインの
/// redeploy 副作用がデプロイ先に残ったまま」発生することを防ぐ（rollback はキャッシュ復元のみで
/// 整合する）。redeploy / write_meta 失敗は従来どおり非アトミック（disabled / 警告）扱い。
fn commit_all(
    cache: &dyn PackageCacheAccess,
    staged: Vec<StagedUpdate>,
    project_root: &Path,
    target_filter: Option<&str>,
) -> Vec<UpdateOutcome> {
    // Phase 1: 全件 swap（本番差し替え）。1 件でも失敗したら、まだ redeploy は一切
    // 行っていないので ROLLBACK はキャッシュ復元だけで完結する。
    for (idx, s) in staged.iter().enumerate() {
        let mp = s.target.marketplace.as_deref();
        if let Err(e) = cache.commit_staged(mp, &s.target.cache_id) {
            // swap 失敗 → ROLLBACK（既 swap 分 + 当該 + 未 swap 残り全件）。
            // 当該は staged 内インデックスで識別する（cache_id 文字列は
            // 別 marketplace 間で衝突し得るため使わない）。
            return rollback_all(cache, &staged, idx, e);
        }
    }

    // Phase 2: 全件 swap 成功。redeploy + meta を行う（非アトミック: ここから先は
    // 失敗しても ROLLBACK しない）。
    let mut results: Vec<UpdateOutcome> = Vec::new();
    for s in &staged {
        let mp = s.target.marketplace.as_deref();
        let plugin_path = cache.plugin_path(mp, &s.target.cache_id);

        // redeploy（非アトミック: 失敗 target は disabled）
        let enabled = s.target.old_meta.enabled_targets();
        let targets: Vec<&str> = match target_filter {
            Some(f) => enabled.into_iter().filter(|t| *t == f).collect(),
            None => enabled,
        };
        let (deployed, failed) =
            redeploy_to_targets(cache, &s.target.cache_id, mp, &targets, project_root);

        // meta 更新（best-effort。アトミック境界は swap までのため失敗しても巻き戻さない）
        let mut new_meta = s.target.old_meta.clone();
        new_meta.set_git_info(&s.target.git_ref, &s.archive_sha);
        for t in &failed {
            new_meta.set_status(t, "disabled");
        }
        // best-effort: 失敗してもロールバックしない（アトミック境界は swap まで）が、
        // commit SHA がメタに反映されない不整合を無言にしないよう警告する。
        // check_all は meta の commit_sha とリモート SHA を比較して更新要否を判定するため、
        // ここで write_meta が失敗すると commit_sha が旧値のまま残り、次回 update で
        // 当該プラグインは再び更新対象として処理される（=自動収束ではなく再ダウンロード・再 swap）。
        if let Err(e) = meta::write_meta(&plugin_path, &new_meta) {
            eprintln!(
                "Warning: Failed to write metadata for '{}': {} \
                 (cache already updated, but commit_sha was not persisted; \
                 the plugin will be re-updated on the next run)",
                s.target.display_name, e
            );
        }

        results.push(UpdateOutcome::updated(
            &s.target.display_name,
            s.target.old_meta.commit_sha.clone(),
            s.archive_sha.clone(),
            deployed,
            failed,
        ));
    }

    // 全件成功 → backup 確定削除
    for s in &staged {
        let mp = s.target.marketplace.as_deref();
        cleanup_backup(cache, mp, &s.target.cache_id);
    }
    results
}

/// ROLLBACK: swap 済み全件 + 未 swap 残り全件を backup から restore する。
///
/// restore 失敗は警告として該当 outcome の error に載せる（部分復元失敗）。
/// `failed_idx` は swap に失敗した当該プラグインの `staged` 内インデックス。当該は restore
/// （部分 swap の巻き戻しのため restore 自体は必要）しつつ、結果は `Failed` のみを 1 件 push する。
/// `staged` 全件を単一パスで走査し、インデックス一致で当該を識別することで、
/// 別 marketplace 間の同名 cache_id 衝突による二重計上を防ぐ。
fn rollback_all(
    cache: &dyn PackageCacheAccess,
    staged: &[StagedUpdate],
    failed_idx: usize,
    cause: PlmError,
) -> Vec<UpdateOutcome> {
    let mut results = Vec::new();
    for (idx, s) in staged.iter().enumerate() {
        let mp = s.target.marketplace.as_deref();
        let restored = cache.restore(mp, &s.target.cache_id);
        // 未 swap 分に残る staging temp も掃除（既 swap 分は commit_staged で temp 消費済み）
        cleanup_staged(cache, mp, &s.target.cache_id);

        if idx == failed_idx {
            // 当該（swap 失敗）は Failed のみ。restore 結果は警告として error に併記。
            let msg = match restored {
                Ok(()) => format!("Commit (swap) failed: {}", cause),
                Err(e) => format!(
                    "Commit (swap) failed: {} (WARNING: restore also failed: {})",
                    cause, e
                ),
            };
            results.push(UpdateOutcome::failed(&s.target.display_name, msg));
        } else {
            results.push(match restored {
                Ok(()) => UpdateOutcome::rolled_back(&s.target.display_name, None),
                Err(e) => UpdateOutcome::rolled_back(
                    &s.target.display_name,
                    Some(format!("WARNING: restore failed: {}", e)),
                ),
            });
        }
    }
    results
}

#[cfg(test)]
#[path = "update_test.rs"]
mod tests;

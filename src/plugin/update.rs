//! プラグイン更新ユースケース
//!
//! GitHub APIを使用して最新のコミットSHAを取得し、インストール済みSHAと比較して
//! 差分がある場合に再ダウンロード・再デプロイを行う。

use crate::application::enable_plugin;
use crate::host::{HostClientFactory, HostKind};
use crate::http::with_retry;
use crate::plugin::version::{fetch_remote_versions, needs_update, VersionQueryResult};
use crate::plugin::{meta, PluginCacheAccess, PluginMeta};
use crate::repo::Repo;
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
fn restore_repo(
    meta: &PluginMeta,
    plugin_name: &str,
    git_ref: &str,
) -> std::result::Result<Repo, String> {
    if let Some((owner, name)) = meta.get_source_repo() {
        let repo = Repo::new(HostKind::GitHub, owner, name, Some(git_ref.to_string()));
        return Ok(repo);
    }

    // フォールバック: owner--repo 形式からパース
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

/// 単一プラグインの更新
pub async fn update_plugin(
    cache: &dyn PluginCacheAccess,
    plugin_name: &str,
    project_root: &Path,
    target_filter: Option<&str>,
) -> UpdateResult {
    // プラグインがキャッシュに存在するか確認
    if !cache.is_cached(Some("github"), plugin_name) {
        return UpdateResult::failed(plugin_name, "Plugin not found in cache".to_string());
    }

    let plugin_path = cache.plugin_path(Some("github"), plugin_name);
    let plugin_meta = meta::load_meta(&plugin_path).unwrap_or_default();

    // GitHub プラグインかどうか確認
    if !plugin_meta.is_github() {
        return UpdateResult::skipped(plugin_name, "Not a GitHub plugin".to_string());
    }

    // Git情報取得（未保存時はHEAD）
    let git_ref = plugin_meta.git_ref.as_deref().unwrap_or("HEAD");
    let current_sha = plugin_meta.commit_sha.clone();

    // リポジトリ情報復元
    let repo = match restore_repo(&plugin_meta, plugin_name, git_ref) {
        Ok(r) => r,
        Err(e) => return UpdateResult::failed(plugin_name, e),
    };

    // GitHubクライアント作成
    let factory = HostClientFactory::with_defaults();
    let client = factory.create(HostKind::GitHub);

    // 最新SHAを取得（リトライ付き）
    let latest_sha = match with_retry(|| client.get_commit_sha(&repo, git_ref), 3).await {
        Ok(sha) => sha,
        Err(e) => {
            return UpdateResult::failed(plugin_name, format!("Failed to get latest SHA: {}", e))
        }
    };

    // 比較判定
    if current_sha.as_deref() == Some(&latest_sha) {
        return UpdateResult::up_to_date(plugin_name);
    }

    // commit_sha 未保存時は警告表示
    if current_sha.is_none() {
        eprintln!(
            "Warning: No commit SHA recorded for '{}'. Forcing update.",
            plugin_name
        );
    }

    // 更新処理実行
    do_update(
        plugin_name,
        &latest_sha,
        cache,
        &*client,
        &repo,
        &plugin_meta,
        project_root,
        target_filter,
    )
    .await
}

/// 更新処理の実行
#[allow(clippy::too_many_arguments)]
async fn do_update(
    plugin_name: &str,
    latest_sha: &str,
    cache: &dyn PluginCacheAccess,
    client: &dyn crate::host::HostClient,
    repo: &Repo,
    plugin_meta: &PluginMeta,
    project_root: &Path,
    target_filter: Option<&str>,
) -> UpdateResult {
    let current_sha = plugin_meta.commit_sha.clone();
    let git_ref = plugin_meta.git_ref.as_deref().unwrap_or("HEAD");

    // バックアップ作成
    println!("  Creating backup...");
    if let Err(e) = cache.backup(Some("github"), plugin_name) {
        return UpdateResult::failed(plugin_name, format!("Backup failed: {}", e));
    }

    // ダウンロード（リトライ付き）
    println!("  Downloading...");
    let archive = match with_retry(|| client.download_archive(repo), 3).await {
        Ok(a) => a,
        Err(e) => {
            // ロールバック
            let _ = cache.restore(Some("github"), plugin_name);
            return UpdateResult::failed(plugin_name, format!("Download failed: {}", e));
        }
    };

    // アトミック更新
    println!("  Extracting...");
    let plugin_path = match cache.atomic_update(Some("github"), plugin_name, &archive) {
        Ok(p) => p,
        Err(e) => {
            // ロールバック
            let _ = cache.restore(Some("github"), plugin_name);
            return UpdateResult::failed(plugin_name, format!("Extraction failed: {}", e));
        }
    };

    // 再デプロイ
    println!("  Deploying...");
    let enabled = plugin_meta.enabled_targets();
    let targets: Vec<&str> = match target_filter {
        Some(f) => enabled.into_iter().filter(|t| *t == f).collect(),
        None => enabled,
    };

    let (deployed, failed) = redeploy_to_targets(cache, plugin_name, &targets, project_root);

    // メタデータ更新
    let mut new_meta = plugin_meta.clone();
    new_meta.set_git_info(git_ref, latest_sha);
    for t in &failed {
        new_meta.set_status(t, "disabled");
    }
    if let Err(e) = meta::write_meta(&plugin_path, &new_meta) {
        eprintln!("Warning: Failed to update metadata: {}", e);
    }

    // バックアップ削除
    let _ = cache.remove_backup(Some("github"), plugin_name);

    UpdateResult::updated(
        plugin_name,
        current_sha,
        latest_sha.to_string(),
        deployed,
        failed,
    )
}

/// ターゲットへの再デプロイ
fn redeploy_to_targets(
    cache: &dyn PluginCacheAccess,
    plugin_name: &str,
    targets: &[&str],
    project_root: &Path,
) -> (Vec<String>, Vec<String>) {
    let mut deployed = Vec::new();
    let mut failed = Vec::new();

    for target in targets {
        let result = enable_plugin(
            cache,
            plugin_name,
            Some("github"),
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
/// キャッシュ内の全プラグインを走査し、各プラグインのメタデータから
/// sourceRepo を取得して更新を実行する。
/// GitHub以外のプラグインはSkippedとして扱う。
/// 一部失敗しても後続の処理を継続する。
pub async fn update_all_plugins(
    cache: &dyn PluginCacheAccess,
    project_root: &Path,
    target_filter: Option<&str>,
) -> Vec<UpdateResult> {
    // キャッシュ内の全プラグインを取得
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

    // 更新チェック用のメタデータを収集
    let mut plugin_metas: Vec<(String, PluginMeta)> = Vec::new();
    for (marketplace, name) in &plugins {
        // GitHub プラグインのみ対象
        if marketplace.is_some() && marketplace.as_deref() != Some("github") {
            continue;
        }
        let plugin_path = cache.plugin_path(marketplace.as_deref(), name);
        let meta = meta::load_meta(&plugin_path).unwrap_or_default();
        plugin_metas.push((name.clone(), meta));
    }

    // 一括リモートバージョン取得（version モジュールを使用）
    println!("Checking for updates...");
    let factory = HostClientFactory::with_defaults();
    let client = factory.create(HostKind::GitHub);
    let remote_versions = fetch_remote_versions(&plugin_metas, client.as_ref()).await;

    // 更新対象を集計
    let mut updates_to_do: Vec<(&str, String, &PluginMeta)> = Vec::new();
    let mut error_count = 0;

    for ((name, meta), (_plugin_name, result)) in plugin_metas.iter().zip(remote_versions.iter()) {
        match result {
            VersionQueryResult::Failed { message } => {
                error_count += 1;
                eprintln!("  {}: Failed to check ({})", name, message);
            }
            VersionQueryResult::Found(remote) => {
                if needs_update(meta.commit_sha.as_deref(), &remote.sha) {
                    let current_short = meta
                        .commit_sha
                        .as_ref()
                        .map(|s| &s[..7.min(s.len())])
                        .unwrap_or("unknown");
                    let latest_short = &remote.sha[..7.min(remote.sha.len())];
                    println!("  {}: {} -> {}", name, current_short, latest_short);
                    updates_to_do.push((name, remote.sha.clone(), meta));
                } else {
                    println!("  {}: Already up to date", name);
                }
            }
        }
    }

    let update_count = updates_to_do.len();
    let up_to_date_count = plugin_metas.len() - update_count - error_count;

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
        return plugin_metas
            .iter()
            .map(|(name, _)| UpdateResult::up_to_date(name))
            .collect();
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

    // 各プラグインを更新
    let mut results = Vec::new();
    let mut update_idx = 0;

    for (name, latest_sha, meta) in &updates_to_do {
        update_idx += 1;
        println!("\n[{}/{}] Updating {}...", update_idx, update_count, name);

        let git_ref = meta.git_ref.as_deref().unwrap_or("HEAD");
        let repo = match restore_repo(meta, name, git_ref) {
            Ok(r) => r,
            Err(e) => {
                results.push(UpdateResult::failed(name, e));
                continue;
            }
        };

        let update_factory = HostClientFactory::with_defaults();
        let update_client = update_factory.create(HostKind::GitHub);

        let result = do_update(
            name,
            latest_sha,
            cache,
            &*update_client,
            &repo,
            meta,
            project_root,
            target_filter,
        )
        .await;

        // 結果表示
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

    // 更新対象外のプラグインも結果に追加
    for (name, _) in &plugin_metas {
        if !updates_to_do.iter().any(|(n, _, _)| n == name) {
            results.push(UpdateResult::up_to_date(name));
        }
    }

    results
}

#[cfg(test)]
#[path = "update_test.rs"]
mod tests;

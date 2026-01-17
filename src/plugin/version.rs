//! リモートバージョン取得
//!
//! GitHub APIを使用して最新のコミットSHAを取得する。
//! `plm update` と `plm list --outdated` の両方で使用される。

use crate::error::PlmError;
use crate::host::{HostClient, HostKind};
use crate::plugin::PluginMeta;
use crate::repo::Repo;
use serde::Serialize;

/// リモート（GitHub等）から取得したバージョン情報
#[derive(Debug, Clone, Serialize)]
pub struct RemoteVersion {
    /// コミットSHA
    pub sha: String,
    /// ブランチ/タグ名
    pub git_ref: String,
}

/// リモートバージョン取得の結果
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum VersionQueryResult {
    /// 取得成功
    Found(RemoteVersion),
    /// 取得失敗
    Failed {
        /// エラーメッセージ
        message: String,
    },
}

impl VersionQueryResult {
    /// 取得成功かどうか
    pub fn is_found(&self) -> bool {
        matches!(self, Self::Found(_))
    }

    /// 取得失敗かどうか
    pub fn is_failed(&self) -> bool {
        matches!(self, Self::Failed { .. })
    }

    /// 成功時のRemoteVersionを取得
    pub fn as_found(&self) -> Option<&RemoteVersion> {
        match self {
            Self::Found(v) => Some(v),
            Self::Failed { .. } => None,
        }
    }

    /// 失敗時のエラーメッセージを取得
    pub fn error_message(&self) -> Option<&str> {
        match self {
            Self::Found(_) => None,
            Self::Failed { message } => Some(message),
        }
    }
}

/// ローカルとリモートのバージョンを比較し、更新が必要かどうかを判定
///
/// - ローカルSHAがNoneの場合: 更新が必要（インストール時にSHAが記録されていない）
/// - ローカルSHAとリモートSHAが異なる場合: 更新が必要
/// - ローカルSHAとリモートSHAが同じ場合: 更新不要
pub fn needs_update(local_sha: Option<&str>, remote_sha: &str) -> bool {
    match local_sha {
        Some(local) => local != remote_sha,
        None => true,
    }
}

/// PlmErrorからエラーメッセージを生成
fn error_message(error: &PlmError) -> String {
    match error {
        PlmError::RepoApi {
            status: 403,
            message,
            ..
        } => {
            let msg_lower = message.to_lowercase();
            if msg_lower.contains("rate limit") || msg_lower.contains("ratelimit") {
                "Rate limited".to_string()
            } else {
                "Access denied".to_string()
            }
        }
        PlmError::RepoApi { status: 404, .. } => "Repository or ref not found".to_string(),
        PlmError::Network(_) => "Network error".to_string(),
        _ => error.to_string(),
    }
}

/// 単一プラグインのリモートバージョンを取得
///
/// 1. `meta.git_ref` を取得（未記録時は `client.get_default_branch()` を使用）
/// 2. `client.get_commit_sha()` で最新 SHA を取得
pub async fn fetch_remote_version(
    meta: &PluginMeta,
    client: &dyn HostClient,
) -> VersionQueryResult {
    // リポジトリ情報がなければエラー
    let (owner, name) = match meta.get_source_repo() {
        Some(repo) => repo,
        None => {
            return VersionQueryResult::Failed {
                message: "No repository info".to_string(),
            }
        }
    };

    // git_ref を取得（未記録時はデフォルトブランチを取得）
    let repo_for_default = Repo::new(HostKind::GitHub, owner, name, None);
    let git_ref = match &meta.git_ref {
        Some(r) => r.clone(),
        None => match client.get_default_branch(&repo_for_default).await {
            Ok(branch) => branch,
            Err(e) => {
                return VersionQueryResult::Failed {
                    message: error_message(&e),
                }
            }
        },
    };

    // リポジトリ情報を構築
    let repo = Repo::new(HostKind::GitHub, owner, name, Some(git_ref.clone()));

    // 最新 SHA を取得
    match client.get_commit_sha(&repo, &git_ref).await {
        Ok(sha) => VersionQueryResult::Found(RemoteVersion { sha, git_ref }),
        Err(e) => VersionQueryResult::Failed {
            message: error_message(&e),
        },
    }
}

/// 複数プラグインのリモートバージョンを一括取得
///
/// 各プラグインに対して `fetch_remote_version()` を呼び出し、結果を集約する。
/// エラーが発生しても後続の処理を継続する。
pub async fn fetch_remote_versions(
    plugins: &[(String, PluginMeta)],
    client: &dyn HostClient,
) -> Vec<(String, VersionQueryResult)> {
    let mut results = Vec::with_capacity(plugins.len());

    for (name, meta) in plugins {
        let result = fetch_remote_version(meta, client).await;
        results.push((name.clone(), result));
    }

    results
}

#[cfg(test)]
#[path = "version_test.rs"]
mod tests;

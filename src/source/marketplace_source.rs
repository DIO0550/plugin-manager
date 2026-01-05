//! Marketplace 経由のダウンロード

use crate::error::{PlmError, Result};
use crate::marketplace::{MarketplaceRegistry, PluginSource as MpPluginSource};
use crate::plugin::CachedPlugin;
use crate::repo;
use crate::source::path_utils::normalize_subdir_path;
use std::future::Future;
use std::path::{Component, Path};
use std::pin::Pin;

use super::{GitHubSource, PluginSource};

/// サブディレクトリパスが {owner}/{repo}/... パターンで始まらないことを検証
///
/// GitHub は大文字小文字非依存なので case-insensitive で比較。
/// 単一要素の一致は実在フォルダ名と偶然一致する可能性があるため拒否しない。
fn validate_subdir_not_owner_repo(subdir: &str, owner: &str, repo_name: &str) -> Result<()> {
    let mut components = Path::new(subdir).components();
    let first = components.next().and_then(|c| match c {
        Component::Normal(s) => s.to_str(),
        _ => None,
    });
    let second = components.next().and_then(|c| match c {
        Component::Normal(s) => s.to_str(),
        _ => None,
    });

    if let (Some(first), Some(second)) = (first, second) {
        if first.eq_ignore_ascii_case(owner) && second.eq_ignore_ascii_case(repo_name) {
            return Err(PlmError::InvalidSource(format!(
                "subdir must not start with '{}/{}'. Use a relative path within the repository.",
                owner, repo_name
            )));
        }
    }

    Ok(())
}

/// 指定した Marketplace からプラグインをダウンロードするソース
pub struct MarketplaceSource {
    plugin: String,
    marketplace: String,
}

impl MarketplaceSource {
    pub fn new(plugin: &str, marketplace: &str) -> Self {
        Self {
            plugin: plugin.to_string(),
            marketplace: marketplace.to_string(),
        }
    }
}

impl PluginSource for MarketplaceSource {
    fn download(
        &self,
        force: bool,
    ) -> Pin<Box<dyn Future<Output = Result<CachedPlugin>> + Send + '_>> {
        Box::pin(async move {
            let registry = MarketplaceRegistry::new()?;

            // マーケットプレイスからプラグイン情報を取得
            let mp_cache = registry
                .get(&self.marketplace)?
                .ok_or_else(|| PlmError::MarketplaceNotFound(self.marketplace.clone()))?;

            let plugin_entry = mp_cache
                .plugins
                .iter()
                .find(|p| p.name == self.plugin)
                .ok_or_else(|| PlmError::PluginNotFound(self.plugin.clone()))?;

            // プラグインソースをRepoに変換してダウンロード
            match &plugin_entry.source {
                MpPluginSource::Local(path) => {
                    let parts: Vec<&str> = mp_cache
                        .source
                        .strip_prefix("github:")
                        .unwrap_or(&mp_cache.source)
                        .split('/')
                        .collect();

                    if parts.len() < 2 {
                        return Err(PlmError::InvalidRepoFormat(mp_cache.source.clone()));
                    }

                    let owner = parts[0];
                    let repo_name = parts[1];
                    let repo = repo::from_url(&format!("{}/{}", owner, repo_name))?;

                    // path を正規化（共通ヘルパーを使用）
                    let subdir = normalize_subdir_path(path)?;

                    // {owner}/{repo}/... パターンを拒否
                    validate_subdir_not_owner_repo(&subdir, owner, repo_name)?;

                    // Git ソースに委譲（marketplace + subdir 情報を渡す）
                    GitHubSource::with_marketplace_and_subdir(
                        repo,
                        self.marketplace.clone(),
                        subdir,
                    )
                    .download(force)
                    .await
                }
                MpPluginSource::External { repo: repo_url, .. } => {
                    let repo = repo::from_url(repo_url)?;
                    // Git ソースに委譲（marketplace 情報を渡す）
                    GitHubSource::with_marketplace(repo, self.marketplace.clone())
                        .download(force)
                        .await
                }
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // テストケース24: path = "DIO0550/d-market/plugins/foo" → InvalidSource エラー
    #[test]
    fn test_validate_subdir_rejects_owner_repo_prefix() {
        let result = validate_subdir_not_owner_repo("DIO0550/d-market/plugins/foo", "DIO0550", "d-market");
        assert!(result.is_err());
        match result.unwrap_err() {
            PlmError::InvalidSource(msg) => {
                assert!(msg.contains("DIO0550/d-market"));
            }
            e => panic!("Expected InvalidSource error, got: {:?}", e),
        }
    }

    // テストケース25: path = "DIO0550/d-market/plugins" (正規化後) → InvalidSource エラー
    // Note: normalize_subdir_path で "./DIO0550/d-market/plugins" は "DIO0550/d-market/plugins" になる
    #[test]
    fn test_validate_subdir_rejects_owner_repo_prefix_after_normalization() {
        // まず正規化
        let normalized = normalize_subdir_path("./DIO0550/d-market/plugins").unwrap();
        assert_eq!(normalized, "DIO0550/d-market/plugins");

        // 検証でエラーになる
        let result = validate_subdir_not_owner_repo(&normalized, "DIO0550", "d-market");
        assert!(result.is_err());
    }

    // テストケース26: path = "DIO0550/plugins/foo" → 正常（単一要素は許可）
    #[test]
    fn test_validate_subdir_allows_single_owner_match() {
        let result = validate_subdir_not_owner_repo("DIO0550/plugins/foo", "DIO0550", "d-market");
        assert!(result.is_ok());
    }

    // テストケース27: path = "d-market/plugins/foo" → 正常（単一要素は許可）
    #[test]
    fn test_validate_subdir_allows_single_repo_match() {
        let result = validate_subdir_not_owner_repo("d-market/plugins/foo", "DIO0550", "d-market");
        assert!(result.is_ok());
    }

    // Case-insensitive テスト
    #[test]
    fn test_validate_subdir_is_case_insensitive() {
        // 大文字小文字が異なっても拒否される
        let result = validate_subdir_not_owner_repo("dio0550/D-Market/plugins", "DIO0550", "d-market");
        assert!(result.is_err());
    }

    // 単一要素のパスは常にOK
    #[test]
    fn test_validate_subdir_allows_single_element_path() {
        let result = validate_subdir_not_owner_repo("plugins", "DIO0550", "d-market");
        assert!(result.is_ok());
    }

    // 通常のパスは問題なし
    #[test]
    fn test_validate_subdir_allows_normal_paths() {
        let result = validate_subdir_not_owner_repo("plugins/my-plugin", "DIO0550", "d-market");
        assert!(result.is_ok());
    }
}

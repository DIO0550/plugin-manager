//! Marketplaces タブのアクション実行
//!
//! マーケットプレイスの追加・削除・更新操作を実行する。

use crate::marketplace::{
    to_display_source, to_internal_source, MarketplaceConfig, MarketplaceFetcher,
    MarketplaceRegistration, MarketplaceRegistry,
};
use crate::repo;
use crate::tui::manager::core::MarketplaceItem;

/// マーケットプレイス追加の結果
pub struct AddResult {
    pub marketplace: MarketplaceItem,
}

/// マーケットプレイスを追加
pub fn add_marketplace(
    source: &str,
    name: &str,
    source_path: Option<&str>,
) -> Result<AddResult, String> {
    let handle = tokio::runtime::Handle::try_current()
        .map_err(|_| "No Tokio runtime available".to_string())?;

    let repo = repo::from_url(source).map_err(|e| e.to_string())?;
    let internal_source = to_internal_source(&repo.full_name());

    let mut config = MarketplaceConfig::load()?;

    if config.exists(name) {
        return Err(format!("Marketplace '{}' already exists", name));
    }

    let entry = MarketplaceRegistration {
        name: name.to_string(),
        source: internal_source.clone(),
        source_path: source_path.map(|s| s.to_string()),
    };

    config.add(entry)?;

    // フェッチしてキャッシュ
    let fetcher = MarketplaceFetcher::new();
    let cache = tokio::task::block_in_place(|| {
        handle.block_on(fetcher.fetch_as_cache(&repo, name, source_path))
    })
    .map_err(|e| e.to_string())?;

    // config.save() を先に実行し、失敗時に孤立キャッシュが残らないようにする
    config.save()?;

    let registry = MarketplaceRegistry::new().map_err(|e| e.to_string())?;
    registry.store(&cache).map_err(|e| e.to_string())?;

    Ok(AddResult {
        marketplace: MarketplaceItem {
            name: name.to_string(),
            source: to_display_source(&internal_source),
            source_path: source_path.map(|s| s.to_string()),
            plugin_count: Some(cache.plugins.len()),
            last_updated: Some(cache.fetched_at.format("%Y-%m-%d %H:%M").to_string()),
        },
    })
}

/// マーケットプレイスを削除
pub fn remove_marketplace(name: &str) -> Result<(), String> {
    let mut config = MarketplaceConfig::load()?;
    config.remove(name)?;
    config.save()?;

    // キャッシュ削除（失敗しても続行）
    if let Ok(registry) = MarketplaceRegistry::new() {
        let _ = registry.remove(name);
    }

    Ok(())
}

/// マーケットプレイスを更新
pub fn update_marketplace(name: &str) -> Result<MarketplaceItem, String> {
    let config = MarketplaceConfig::load()?;
    let entry = config
        .get(name)
        .ok_or_else(|| format!("Marketplace '{}' not found", name))?
        .clone();

    update_marketplace_registration(&entry)
}

/// マーケットプレイス登録情報を更新（config再読み込み不要）
fn update_marketplace_registration(
    entry: &MarketplaceRegistration,
) -> Result<MarketplaceItem, String> {
    let handle = tokio::runtime::Handle::try_current()
        .map_err(|_| "No Tokio runtime available".to_string())?;

    let display_source = to_display_source(&entry.source);
    let repo = repo::from_url(&display_source).map_err(|e| e.to_string())?;
    let fetcher = MarketplaceFetcher::new();
    let cache = tokio::task::block_in_place(|| {
        handle.block_on(fetcher.fetch_as_cache(&repo, &entry.name, entry.source_path.as_deref()))
    })
    .map_err(|e| e.to_string())?;

    let registry = MarketplaceRegistry::new().map_err(|e| e.to_string())?;
    registry.store(&cache).map_err(|e| e.to_string())?;

    Ok(MarketplaceItem {
        name: entry.name.clone(),
        source: display_source,
        source_path: entry.source_path.clone(),
        plugin_count: Some(cache.plugins.len()),
        last_updated: Some(cache.fetched_at.format("%Y-%m-%d %H:%M").to_string()),
    })
}

/// 全マーケットプレイスを更新
pub fn update_all_marketplaces() -> Vec<(String, Result<MarketplaceItem, String>)> {
    let config = match MarketplaceConfig::load() {
        Ok(c) => c,
        Err(e) => return vec![("(config)".to_string(), Err(e))],
    };

    config
        .list()
        .iter()
        .map(|entry| {
            let result = update_marketplace_registration(entry);
            (entry.name.clone(), result)
        })
        .collect()
}

/// マーケットプレイスのプラグイン一覧を取得
pub fn get_marketplace_plugins(name: &str) -> Vec<(String, Option<String>)> {
    let registry = match MarketplaceRegistry::new() {
        Ok(r) => r,
        Err(_) => return Vec::new(),
    };

    match registry.get(name) {
        Ok(Some(cache)) => cache
            .plugins
            .iter()
            .map(|p| (p.name.clone(), p.description.clone()))
            .collect(),
        _ => Vec::new(),
    }
}

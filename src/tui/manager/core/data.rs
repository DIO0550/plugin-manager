//! 共有データストア
//!
//! 全タブで共有されるデータを一元管理する。
//! Application層のDTOのみを保持する。

use crate::application::{list_installed_plugins, PluginSummary};
use crate::component::{ComponentKind, ComponentName, ComponentTypeCount};
use crate::marketplace::{to_display_source, MarketplaceConfig, MarketplaceRegistry};
use crate::plugin::PluginCache;
use std::io;

// ============================================================================
// ID 型（安定したIDでの参照用）
// ============================================================================

/// プラグインID（リポジトリ名で識別）
pub type PluginId = String;

// ============================================================================
// MarketplaceItem（TUI表示用）
// ============================================================================

/// マーケットプレイスアイテム（TUI表示用）
#[derive(Debug, Clone)]
pub struct MarketplaceItem {
    pub name: String,
    pub source: String,
    pub source_path: Option<String>,
    pub plugin_count: Option<usize>,
    pub last_updated: Option<String>,
}

// ============================================================================
// DataStore（共有データストア）
// ============================================================================

/// 共有データストア
pub struct DataStore {
    /// プラグインキャッシュ（再利用のため保持）
    cache: PluginCache,
    /// インストール済みプラグイン一覧
    pub plugins: Vec<PluginSummary>,
    /// マーケットプレイス一覧
    pub marketplaces: Vec<MarketplaceItem>,
    /// 最後のエラー
    pub last_error: Option<String>,
}

impl DataStore {
    /// 新しいデータストアを作成
    pub fn new() -> io::Result<Self> {
        let cache = PluginCache::new().map_err(|e| io::Error::other(e.to_string()))?;
        let plugins =
            list_installed_plugins(&cache).map_err(|e| io::Error::other(e.to_string()))?;
        let LoadMarketplacesResult { items, error } = load_marketplaces();

        Ok(Self {
            cache,
            plugins,
            marketplaces: items,
            last_error: error,
        })
    }

    /// データストアをリロード（list_installed_plugins() で全体再取得）
    pub fn reload(&mut self) -> io::Result<()> {
        self.plugins =
            list_installed_plugins(&self.cache).map_err(|e| io::Error::other(e.to_string()))?;
        let result = load_marketplaces();
        self.marketplaces = result.items;
        // 既存の last_error を上書きせず、マーケットプレイス読み込みエラーを追記/保存する
        self.last_error = merge_errors(self.last_error.take(), result.error);
        Ok(())
    }

    /// プラグインIDでプラグインを検索
    pub fn find_plugin(&self, id: &PluginId) -> Option<&PluginSummary> {
        self.plugins.iter().find(|p| p.name == *id)
    }

    /// プラグインIDでインデックスを検索
    pub fn plugin_index(&self, id: &PluginId) -> Option<usize> {
        self.plugins.iter().position(|p| p.name == *id)
    }

    /// プラグインの空でないコンポーネント種別を取得
    pub fn available_component_kinds(&self, plugin: &PluginSummary) -> Vec<ComponentTypeCount> {
        plugin.component_type_counts()
    }

    /// コンポーネント種別に応じたコンポーネント名一覧を取得
    pub fn component_names(
        &self,
        plugin: &PluginSummary,
        kind: ComponentKind,
    ) -> Vec<ComponentName> {
        plugin.component_names(kind)
    }

    /// プラグインを一覧から削除
    pub fn remove_plugin(&mut self, plugin_id: &PluginId) {
        self.plugins.retain(|p| p.name != *plugin_id);
    }

    /// プラグインの有効状態を更新
    pub fn set_plugin_enabled(&mut self, plugin_id: &PluginId, enabled: bool) {
        if let Some(plugin) = self.plugins.iter_mut().find(|p| &p.name == plugin_id) {
            plugin.enabled = enabled;
        }
    }

    /// マーケットプレイスデータをリロード
    pub fn reload_marketplaces(&mut self) {
        let result = load_marketplaces();
        self.marketplaces = result.items;
        // 既存の last_error を上書きせず、マーケットプレイス読み込みエラーを追記/保存する
        self.last_error = merge_errors(self.last_error.take(), result.error);
    }

    /// マーケットプレイス名で検索
    pub fn find_marketplace(&self, name: &str) -> Option<&MarketplaceItem> {
        self.marketplaces.iter().find(|m| m.name == name)
    }

    /// マーケットプレイス名でインデックスを検索
    pub fn marketplace_index(&self, name: &str) -> Option<usize> {
        self.marketplaces.iter().position(|m| m.name == name)
    }

    /// マーケットプレイスを一覧から削除
    pub fn remove_marketplace(&mut self, name: &str) {
        self.marketplaces.retain(|m| m.name != name);
    }
}

/// マーケットプレイスデータ読み込み結果
struct LoadMarketplacesResult {
    items: Vec<MarketplaceItem>,
    error: Option<String>,
}

/// マーケットプレイスデータを読み込み
fn load_marketplaces() -> LoadMarketplacesResult {
    let config = match MarketplaceConfig::load() {
        Ok(c) => c,
        Err(e) => {
            return LoadMarketplacesResult {
                items: Vec::new(),
                error: Some(format!("Failed to load marketplace config: {}", e)),
            }
        }
    };

    let registry = match MarketplaceRegistry::new() {
        Ok(r) => r,
        Err(e) => {
            // レジストリが作成できない場合はキャッシュなしで一覧だけ返す
            let items = config
                .list()
                .iter()
                .map(|entry| MarketplaceItem {
                    name: entry.name.clone(),
                    source: to_display_source(&entry.source),
                    source_path: entry.source_path.clone(),
                    plugin_count: None,
                    last_updated: None,
                })
                .collect();
            return LoadMarketplacesResult {
                items,
                error: Some(format!("Failed to load marketplace registry: {}", e)),
            };
        }
    };

    let items = config
        .list()
        .iter()
        .map(|entry| {
            let (plugin_count, last_updated) = match registry.get(&entry.name) {
                Ok(Some(cache)) => (
                    Some(cache.plugins.len()),
                    Some(cache.fetched_at.format("%Y-%m-%d %H:%M").to_string()),
                ),
                _ => (None, None),
            };
            MarketplaceItem {
                name: entry.name.clone(),
                source: to_display_source(&entry.source),
                source_path: entry.source_path.clone(),
                plugin_count,
                last_updated,
            }
        })
        .collect();

    LoadMarketplacesResult { items, error: None }
}

/// 2つのエラーをマージする（既存エラーを保持しつつ新しいエラーを追記）
fn merge_errors(existing: Option<String>, new: Option<String>) -> Option<String> {
    match (existing, new) {
        (Some(prev), Some(next)) => Some(format!("{}\n{}", prev, next)),
        (Some(prev), None) => Some(prev),
        (None, next) => next,
    }
}

#[cfg(test)]
impl DataStore {
    /// テスト用コンストラクタ（PluginCache を使わない軽量版）
    pub fn for_test(
        plugins: Vec<PluginSummary>,
        marketplaces: Vec<MarketplaceItem>,
        last_error: Option<String>,
    ) -> Self {
        // テスト用に一時ディレクトリでキャッシュを構築
        let cache_dir = std::env::temp_dir().join("plm-test-cache");
        let cache =
            PluginCache::with_cache_dir(cache_dir).expect("Failed to create test PluginCache");
        Self {
            cache,
            plugins,
            marketplaces,
            last_error,
        }
    }
}

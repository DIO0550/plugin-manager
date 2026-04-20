//! 共有データストア
//!
//! 全タブで共有されるデータを一元管理する。
//! Application層のDTOとパッケージキャッシュを保持する。

use crate::application::{list_installed_plugins, InstalledPlugin};
use crate::component::ComponentKind;
use crate::marketplace::{to_display_source, MarketplaceConfig, MarketplaceRegistry};
use crate::plugin::PackageCache;
use std::io;

/// プラグインID（`InstalledPlugin::id()` の値で識別。リポジトリ名と異なる場合あり）
pub type PluginId = String;

/// マーケットプレイスアイテム（TUI表示用）
#[derive(Debug, Clone)]
pub struct MarketplaceItem {
    pub name: String,
    pub source: String,
    pub source_path: Option<String>,
    pub plugin_count: Option<usize>,
    pub last_updated: Option<String>,
}

/// 共有データストア
pub struct DataStore {
    /// パッケージキャッシュ（再利用のため保持）
    cache: PackageCache,
    /// インストール済みプラグイン一覧
    pub plugins: Vec<InstalledPlugin>,
    /// マーケットプレイス一覧
    pub marketplaces: Vec<MarketplaceItem>,
    /// 最後のエラー
    pub last_error: Option<String>,
}

impl DataStore {
    /// 新しいデータストアを作成
    pub fn new() -> io::Result<Self> {
        let cache = PackageCache::new().map_err(|e| io::Error::other(e.to_string()))?;
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
    ///
    /// # Arguments
    ///
    /// * `id` - the plugin id to look up
    pub fn find_plugin(&self, id: &PluginId) -> Option<&InstalledPlugin> {
        self.plugins.iter().find(|p| p.id() == id.as_str())
    }

    /// `plugin_id` は `InstalledPlugin.id()`（= 操作用キー）と完全一致で比較される。
    /// marketplace の区別はしない（既存の `find_plugin` と同じ設計）。
    /// `enabled` の状態に関わらず、`plugins` に存在すれば `true` を返す。
    ///
    /// # Arguments
    ///
    /// * `plugin_id` - the id to match exactly against `InstalledPlugin.id()`
    pub fn is_plugin_installed(&self, plugin_id: &str) -> bool {
        self.plugins.iter().any(|p| p.id() == plugin_id)
    }

    /// プラグインIDでインデックスを検索
    ///
    /// # Arguments
    ///
    /// * `id` - the plugin id to look up
    pub fn plugin_index(&self, id: &PluginId) -> Option<usize> {
        self.plugins.iter().position(|p| p.id() == id.as_str())
    }

    /// プラグインの空でないコンポーネント種別を取得
    ///
    /// # Arguments
    ///
    /// * `plugin` - the plugin whose non-empty component kinds are reported
    pub fn available_component_kinds(
        &self,
        plugin: &InstalledPlugin,
    ) -> Vec<(ComponentKind, usize)> {
        plugin.component_type_counts()
    }

    /// コンポーネント種別に応じたコンポーネント名一覧を取得
    ///
    /// # Arguments
    ///
    /// * `plugin` - the plugin whose components are enumerated
    /// * `kind` - the component kind to filter by
    pub fn component_names(&self, plugin: &InstalledPlugin, kind: ComponentKind) -> Vec<String> {
        plugin.component_names(kind)
    }

    /// プラグインを一覧から削除
    ///
    /// # Arguments
    ///
    /// * `plugin_id` - the id of the plugin to remove
    pub fn remove_plugin(&mut self, plugin_id: &PluginId) {
        self.plugins.retain(|p| p.id() != plugin_id.as_str());
    }

    /// プラグインの有効状態を更新
    ///
    /// # Arguments
    ///
    /// * `plugin_id` - the id of the plugin to update
    /// * `enabled` - the new enabled state to apply
    pub fn set_plugin_enabled(&mut self, plugin_id: &PluginId, enabled: bool) {
        if let Some(plugin) = self
            .plugins
            .iter_mut()
            .find(|p| p.id() == plugin_id.as_str())
        {
            plugin.set_enabled(enabled);
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
    ///
    /// # Arguments
    ///
    /// * `name` - the marketplace name to look up
    pub fn find_marketplace(&self, name: &str) -> Option<&MarketplaceItem> {
        self.marketplaces.iter().find(|m| m.name == name)
    }

    /// マーケットプレイス名でインデックスを検索
    ///
    /// # Arguments
    ///
    /// * `name` - the marketplace name to look up
    pub fn marketplace_index(&self, name: &str) -> Option<usize> {
        self.marketplaces.iter().position(|m| m.name == name)
    }

    /// マーケットプレイスを一覧から削除
    ///
    /// # Arguments
    ///
    /// * `name` - the marketplace name to remove
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
///
/// # Arguments
///
/// * `existing` - the previously recorded error message, if any
/// * `new` - the newly encountered error message to append, if any
fn merge_errors(existing: Option<String>, new: Option<String>) -> Option<String> {
    match (existing, new) {
        (Some(prev), Some(next)) => Some(format!("{}\n{}", prev, next)),
        (Some(prev), None) => Some(prev),
        (None, next) => next,
    }
}

#[cfg(test)]
impl DataStore {
    /// テスト用コンストラクタ（一時キャッシュ使用）
    ///
    /// `tempfile::TempDir` を使用してユニークな一時ディレクトリを作成する。
    /// 呼び出し側で `TempDir` をスコープに保持し、Drop で自動クリーンアップされる。
    pub fn for_test(
        plugins: Vec<InstalledPlugin>,
        marketplaces: Vec<MarketplaceItem>,
        last_error: Option<String>,
    ) -> (tempfile::TempDir, Self) {
        let temp_dir = tempfile::TempDir::new().expect("Failed to create temp directory for test");
        let cache_dir = temp_dir.path().to_path_buf();
        let cache =
            PackageCache::with_cache_dir(cache_dir).expect("Failed to create test PackageCache");
        (
            temp_dir,
            Self {
                cache,
                plugins,
                marketplaces,
                last_error,
            },
        )
    }
}

//! 共有データストア
//!
//! 全タブで共有されるデータを一元管理する。
//! Application層のDTOのみを保持する。

use crate::application::{list_installed_plugins, PluginSummary};
use crate::component::{ComponentKind, ComponentName, ComponentTypeCount};
use std::io;

// ============================================================================
// ID 型（安定したIDでの参照用）
// ============================================================================

/// プラグインID（リポジトリ名で識別）
pub type PluginId = String;

// ============================================================================
// DataStore（共有データストア）
// ============================================================================

/// 共有データストア
pub struct DataStore {
    /// インストール済みプラグイン一覧
    pub plugins: Vec<PluginSummary>,
    /// 最後のエラー
    pub last_error: Option<String>,
}

impl DataStore {
    /// 新しいデータストアを作成
    pub fn new() -> io::Result<Self> {
        let plugins = list_installed_plugins()
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;

        Ok(Self {
            plugins,
            last_error: None,
        })
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
}

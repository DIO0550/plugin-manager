//! プラグイン管理 TUI の状態管理
//!
//! アプリケーション状態とタブ・画面の定義。

use crate::application::{list_installed_plugins, PluginSummary};
use ratatui::widgets::ListState;
use std::io;

// ============================================================================
// View 用データ型（ドメイン構造を隠蔽）
// ============================================================================

/// インストール済みプラグインの概要
pub(super) struct InstalledPluginSummary {
    pub name: String,
    pub version: String,
    pub marketplace: Option<String>,
}

/// プラグイン詳細（コンポーネント種別画面用）
pub(super) struct PluginDetail {
    pub name: String,
    pub version: String,
    pub marketplace: Option<String>,
}

/// コンポーネント種別と件数
pub(super) struct ComponentTypeCount {
    pub kind: ManagerComponentType,
    pub count: usize,
}

/// コンポーネント名
pub(super) struct ComponentName {
    pub name: String,
}

/// タブ種別
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ManagerTab {
    Discover,
    Installed,
    Marketplaces,
    Errors,
}

impl ManagerTab {
    pub(super) fn all() -> &'static [ManagerTab] {
        &[ManagerTab::Discover, ManagerTab::Installed, ManagerTab::Marketplaces, ManagerTab::Errors]
    }

    pub(super) fn title(&self) -> &'static str {
        match self {
            ManagerTab::Discover => "Discover",
            ManagerTab::Installed => "Installed",
            ManagerTab::Marketplaces => "Marketplaces",
            ManagerTab::Errors => "Errors",
        }
    }

    pub(super) fn index(&self) -> usize {
        match self {
            ManagerTab::Discover => 0,
            ManagerTab::Installed => 1,
            ManagerTab::Marketplaces => 2,
            ManagerTab::Errors => 3,
        }
    }

    pub(super) fn from_index(index: usize) -> Self {
        match index % 4 {
            0 => ManagerTab::Discover,
            1 => ManagerTab::Installed,
            2 => ManagerTab::Marketplaces,
            _ => ManagerTab::Errors,
        }
    }
}

/// コンポーネント種別
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ManagerComponentType {
    Skills,
    Agents,
    Commands,
    Instructions,
    Hooks,
}

impl ManagerComponentType {
    pub(super) fn title(&self) -> &'static str {
        match self {
            ManagerComponentType::Skills => "Skills",
            ManagerComponentType::Agents => "Agents",
            ManagerComponentType::Commands => "Commands",
            ManagerComponentType::Instructions => "Instructions",
            ManagerComponentType::Hooks => "Hooks",
        }
    }
}

/// 画面状態
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum ManagerScreen {
    /// プラグイン一覧
    PluginList,
    /// コンポーネント種別選択（プラグインインデックス）
    ComponentTypes(usize),
    /// コンポーネント一覧（プラグインインデックス、種別インデックス）
    ComponentList(usize, usize),
}

/// アプリケーション状態
pub(super) struct ManagerApp {
    pub(super) current_tab: ManagerTab,
    pub(super) screen: ManagerScreen,
    pub(super) plugins: Vec<PluginSummary>,
    pub(super) list_state: ListState,
    pub(super) component_type_state: ListState,
    pub(super) component_state: ListState,
    pub(super) should_quit: bool,
}

impl ManagerApp {
    /// 新しいアプリケーション状態を作成
    pub(super) fn new() -> io::Result<Self> {
        let plugins = list_installed_plugins()
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;
        let mut list_state = ListState::default();
        if !plugins.is_empty() {
            list_state.select(Some(0));
        }

        let mut component_type_state = ListState::default();
        component_type_state.select(Some(0));

        Ok(Self {
            current_tab: ManagerTab::Installed,
            screen: ManagerScreen::PluginList,
            plugins,
            list_state,
            component_type_state,
            component_state: ListState::default(),
            should_quit: false,
        })
    }

    /// プラグインの空でないコンポーネント種別を取得
    pub(super) fn available_types(&self, plugin: &PluginSummary) -> Vec<ManagerComponentType> {
        let mut types = Vec::new();
        if !plugin.skills.is_empty() {
            types.push(ManagerComponentType::Skills);
        }
        if !plugin.agents.is_empty() {
            types.push(ManagerComponentType::Agents);
        }
        if !plugin.commands.is_empty() {
            types.push(ManagerComponentType::Commands);
        }
        if !plugin.instructions.is_empty() {
            types.push(ManagerComponentType::Instructions);
        }
        if !plugin.hooks.is_empty() {
            types.push(ManagerComponentType::Hooks);
        }
        types
    }

    /// コンポーネント種別に応じたリストを取得
    pub(super) fn get_components<'a>(
        &self,
        plugin: &'a PluginSummary,
        comp_type: ManagerComponentType,
    ) -> &'a Vec<String> {
        match comp_type {
            ManagerComponentType::Skills => &plugin.skills,
            ManagerComponentType::Agents => &plugin.agents,
            ManagerComponentType::Commands => &plugin.commands,
            ManagerComponentType::Instructions => &plugin.instructions,
            ManagerComponentType::Hooks => &plugin.hooks,
        }
    }

    // ========================================================================
    // View 用読み取り関数（ドメイン構造を隠蔽）
    // ========================================================================

    /// インストール済みプラグインの概要一覧を取得
    pub(super) fn installed_plugin_summaries(&self) -> Vec<InstalledPluginSummary> {
        self.plugins
            .iter()
            .map(|p| InstalledPluginSummary {
                name: p.name.clone(),
                version: p.version.clone(),
                marketplace: p.marketplace.clone(),
            })
            .collect()
    }

    /// インストール済みプラグイン数を取得
    pub(super) fn installed_plugin_count(&self) -> usize {
        self.plugins.len()
    }

    /// プラグイン詳細を取得
    pub(super) fn plugin_detail(&self, plugin_idx: usize) -> Option<PluginDetail> {
        self.plugins.get(plugin_idx).map(|p| PluginDetail {
            name: p.name.clone(),
            version: p.version.clone(),
            marketplace: p.marketplace.clone(),
        })
    }

    /// コンポーネント種別と件数の一覧を取得
    pub(super) fn component_type_counts(&self, plugin_idx: usize) -> Vec<ComponentTypeCount> {
        let Some(plugin) = self.plugins.get(plugin_idx) else {
            return Vec::new();
        };

        self.available_types(plugin)
            .into_iter()
            .map(|kind| {
                let count = self.get_components(plugin, kind).len();
                ComponentTypeCount { kind, count }
            })
            .collect()
    }

    /// type_idx から ManagerComponentType を取得
    pub(super) fn component_type_at(&self, plugin_idx: usize, type_idx: usize) -> Option<ManagerComponentType> {
        let plugin = self.plugins.get(plugin_idx)?;
        let types = self.available_types(plugin);
        types.get(type_idx).copied()
    }

    /// コンポーネント名の一覧を取得
    pub(super) fn component_names(
        &self,
        plugin_idx: usize,
        kind: ManagerComponentType,
    ) -> Vec<ComponentName> {
        let Some(plugin) = self.plugins.get(plugin_idx) else {
            return Vec::new();
        };

        self.get_components(plugin, kind)
            .iter()
            .map(|name| ComponentName { name: name.clone() })
            .collect()
    }
}

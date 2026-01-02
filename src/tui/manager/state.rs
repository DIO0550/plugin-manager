//! プラグイン管理 TUI の状態管理
//!
//! アプリケーション状態とタブ・画面の定義。

use crate::application::{list_installed_plugins, PluginSummary};
use ratatui::widgets::ListState;
use std::io;

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
}

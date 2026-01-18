//! Installed タブの Model/Msg 定義
//!
//! 画面状態とメッセージ型を定義。

use crate::component::ComponentKind;
use crate::tui::manager::core::{DataStore, PluginId};
use crossterm::event::KeyCode;
use ratatui::prelude::*;
use ratatui::widgets::ListState;

// ============================================================================
// CacheState（タブ切替時の保持状態）
// ============================================================================

/// キャッシュ状態（タブ切替時に保持）
#[derive(Debug, Default)]
pub struct CacheState {
    pub selected_plugin_id: Option<PluginId>,
}

// ============================================================================
// DetailAction（プラグイン詳細画面のアクション）
// ============================================================================

/// プラグイン詳細画面のアクション
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DetailAction {
    DisablePlugin,
    EnablePlugin,
    MarkForUpdate,
    UpdateNow,
    Uninstall,
    ViewComponents,
    Back,
}

impl DetailAction {
    /// enabled プラグイン用のアクション一覧
    pub fn for_enabled() -> Vec<DetailAction> {
        vec![
            DetailAction::DisablePlugin,
            DetailAction::MarkForUpdate,
            DetailAction::UpdateNow,
            DetailAction::Uninstall,
            DetailAction::ViewComponents,
            DetailAction::Back,
        ]
    }

    /// disabled プラグイン用のアクション一覧
    pub fn for_disabled() -> Vec<DetailAction> {
        vec![
            DetailAction::EnablePlugin,
            DetailAction::Uninstall,
            DetailAction::ViewComponents,
            DetailAction::Back,
        ]
    }

    /// プラグインの enabled 状態に応じたアクション一覧を取得
    pub fn for_plugin(enabled: bool) -> Vec<DetailAction> {
        if enabled {
            Self::for_enabled()
        } else {
            Self::for_disabled()
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            DetailAction::DisablePlugin => "Disable plugin",
            DetailAction::EnablePlugin => "Enable plugin",
            DetailAction::MarkForUpdate => "Mark for update",
            DetailAction::UpdateNow => "Update now",
            DetailAction::Uninstall => "Uninstall",
            DetailAction::ViewComponents => "View components",
            DetailAction::Back => "Back to plugin list",
        }
    }

    pub fn style(&self) -> Style {
        match self {
            DetailAction::UpdateNow => Style::default().fg(Color::Green),
            DetailAction::EnablePlugin => Style::default().fg(Color::Green),
            DetailAction::Uninstall => Style::default().fg(Color::Red),
            _ => Style::default(),
        }
    }
}

// ============================================================================
// Model（画面状態）
// ============================================================================

/// Installed タブの画面状態
pub enum Model {
    /// プラグイン一覧画面
    PluginList {
        selected_id: Option<PluginId>,
        state: ListState,
    },
    /// プラグイン詳細画面
    PluginDetail {
        plugin_id: PluginId,
        state: ListState,
    },
    /// コンポーネント種別選択画面
    ComponentTypes {
        plugin_id: PluginId,
        selected_kind_idx: usize,
        state: ListState,
    },
    /// コンポーネント一覧画面
    ComponentList {
        plugin_id: PluginId,
        kind: ComponentKind,
        selected_idx: usize,
        state: ListState,
    },
}

impl Model {
    /// 新しいモデルを作成
    pub fn new(data: &DataStore) -> Self {
        let selected_id = data.plugins.first().map(|p| p.name.clone());
        let mut state = ListState::default();
        if selected_id.is_some() {
            state.select(Some(0));
        }
        Model::PluginList { selected_id, state }
    }

    /// キャッシュから復元
    pub fn from_cache(data: &DataStore, cache: &CacheState) -> Self {
        let selected_id = cache
            .selected_plugin_id
            .clone()
            .filter(|id| data.find_plugin(id).is_some())
            .or_else(|| data.plugins.first().map(|p| p.name.clone()));

        let index = selected_id
            .as_ref()
            .and_then(|id| data.plugin_index(id))
            .or(if data.plugins.is_empty() {
                None
            } else {
                Some(0)
            });

        let mut state = ListState::default();
        state.select(index);

        Model::PluginList { selected_id, state }
    }

    /// キャッシュ状態を取得
    pub fn to_cache(&self) -> CacheState {
        match self {
            Model::PluginList { selected_id, .. } => CacheState {
                selected_plugin_id: selected_id.clone(),
            },
            Model::PluginDetail { plugin_id, .. } => CacheState {
                selected_plugin_id: Some(plugin_id.clone()),
            },
            Model::ComponentTypes { plugin_id, .. } => CacheState {
                selected_plugin_id: Some(plugin_id.clone()),
            },
            Model::ComponentList { plugin_id, .. } => CacheState {
                selected_plugin_id: Some(plugin_id.clone()),
            },
        }
    }

    /// トップレベル（タブ切替可能）かどうか
    pub fn is_top_level(&self) -> bool {
        matches!(self, Model::PluginList { .. })
    }

    /// 現在選択中の ListState を取得
    pub fn current_state_mut(&mut self) -> &mut ListState {
        match self {
            Model::PluginList { state, .. } => state,
            Model::PluginDetail { state, .. } => state,
            Model::ComponentTypes { state, .. } => state,
            Model::ComponentList { state, .. } => state,
        }
    }
}

// ============================================================================
// Msg（メッセージ）
// ============================================================================

/// Installed タブへのメッセージ
pub enum Msg {
    Up,
    Down,
    Enter,
    Back,
}

/// キーコードをメッセージに変換
pub fn key_to_msg(key: KeyCode) -> Option<Msg> {
    match key {
        KeyCode::Up | KeyCode::Char('k') => Some(Msg::Up),
        KeyCode::Down | KeyCode::Char('j') => Some(Msg::Down),
        KeyCode::Enter => Some(Msg::Enter),
        KeyCode::Esc => Some(Msg::Back),
        _ => None,
    }
}

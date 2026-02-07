//! Installed タブの Model/Msg 定義
//!
//! 画面状態とメッセージ型を定義。

use crate::component::ComponentKind;
use crate::tui::manager::core::{DataStore, PluginId};
use crossterm::event::KeyCode;
use ratatui::prelude::*;
use ratatui::widgets::ListState;
use std::collections::{HashMap, HashSet};

// ============================================================================
// UpdateStatusDisplay（バッチ更新ステータス表示用）
// ============================================================================

/// バッチ更新時の各プラグインの更新ステータス
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UpdateStatusDisplay {
    /// 更新中
    Updating,
    /// 更新完了
    Updated,
    /// 既に最新
    AlreadyUpToDate,
    /// スキップ
    Skipped(String),
    /// 失敗
    Failed(String),
}

// ============================================================================
// CacheState（タブ切替時の保持状態）
// ============================================================================

/// キャッシュ状態（タブ切替時に保持）
#[derive(Debug, Default)]
pub struct CacheState {
    pub selected_plugin_id: Option<PluginId>,
    pub marked_ids: HashSet<PluginId>,
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
            DetailAction::UpdateNow,
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
        marked_ids: HashSet<PluginId>,
        update_statuses: HashMap<PluginId, UpdateStatusDisplay>,
    },
    /// プラグイン詳細画面
    PluginDetail {
        plugin_id: PluginId,
        state: ListState,
        /// PluginList から遷移時のマーク状態（戻るときに復元）
        saved_marked_ids: HashSet<PluginId>,
        /// PluginList から遷移時の更新ステータス（戻るときに復元）
        saved_update_statuses: HashMap<PluginId, UpdateStatusDisplay>,
    },
    /// コンポーネント種別選択画面
    ComponentTypes {
        plugin_id: PluginId,
        selected_kind_idx: usize,
        state: ListState,
        /// PluginList から遷移チェーンで引き継いだマーク状態
        saved_marked_ids: HashSet<PluginId>,
        /// PluginList から遷移チェーンで引き継いだ更新ステータス
        saved_update_statuses: HashMap<PluginId, UpdateStatusDisplay>,
    },
    /// コンポーネント一覧画面
    ComponentList {
        plugin_id: PluginId,
        kind: ComponentKind,
        selected_idx: usize,
        state: ListState,
        /// PluginList から遷移チェーンで引き継いだマーク状態
        saved_marked_ids: HashSet<PluginId>,
        /// PluginList から遷移チェーンで引き継いだ更新ステータス
        saved_update_statuses: HashMap<PluginId, UpdateStatusDisplay>,
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
        Model::PluginList {
            selected_id,
            state,
            marked_ids: HashSet::new(),
            update_statuses: HashMap::new(),
        }
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

        // マーク状態を復元（DataStore に存在しないプラグインIDは除外）
        let marked_ids = cache
            .marked_ids
            .iter()
            .filter(|id| data.find_plugin(id).is_some())
            .cloned()
            .collect();

        Model::PluginList {
            selected_id,
            state,
            marked_ids,
            update_statuses: HashMap::new(),
        }
    }

    /// キャッシュ状態を取得
    pub fn to_cache(&self) -> CacheState {
        match self {
            Model::PluginList {
                selected_id,
                marked_ids,
                ..
            } => CacheState {
                selected_plugin_id: selected_id.clone(),
                marked_ids: marked_ids.clone(),
            },
            Model::PluginDetail {
                plugin_id,
                saved_marked_ids,
                ..
            } => CacheState {
                selected_plugin_id: Some(plugin_id.clone()),
                marked_ids: saved_marked_ids.clone(),
            },
            Model::ComponentTypes {
                plugin_id,
                saved_marked_ids,
                ..
            } => CacheState {
                selected_plugin_id: Some(plugin_id.clone()),
                marked_ids: saved_marked_ids.clone(),
            },
            Model::ComponentList {
                plugin_id,
                saved_marked_ids,
                ..
            } => CacheState {
                selected_plugin_id: Some(plugin_id.clone()),
                marked_ids: saved_marked_ids.clone(),
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
    ToggleMark,
    ToggleAllMarks,
    BatchUpdate,
    UpdateAll,
    ExecuteBatch,
}

/// キーコードをメッセージに変換
///
/// この関数はフィルタ非フォーカス時にのみ app.rs から呼ばれるため、
/// j/k キーバインドはフィルタ入力と競合しない。
/// Esc はフィルタフォーカス中は app.rs 側で FilterClear として処理されるが、
/// フィルタがフォーカスされていない通常状態では、トップレベルか否かに関わらず Back を返す。
pub fn key_to_msg(key: KeyCode) -> Option<Msg> {
    match key {
        KeyCode::Up | KeyCode::Char('k') => Some(Msg::Up),
        KeyCode::Down | KeyCode::Char('j') => Some(Msg::Down),
        KeyCode::Enter => Some(Msg::Enter),
        KeyCode::Esc => Some(Msg::Back),
        KeyCode::Char(' ') => Some(Msg::ToggleMark),
        KeyCode::Char('a') => Some(Msg::ToggleAllMarks),
        KeyCode::Char('U') => Some(Msg::BatchUpdate),
        KeyCode::Char('A') => Some(Msg::UpdateAll),
        _ => None,
    }
}

#[cfg(test)]
#[path = "model_test.rs"]
mod model_test;

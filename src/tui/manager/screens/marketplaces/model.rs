//! Marketplaces タブの Model/Msg 定義
//!
//! 画面状態とメッセージ型を定義。

use crate::tui::manager::core::DataStore;
use crossterm::event::KeyCode;
use ratatui::widgets::ListState;

// ============================================================================
// OperationStatus（非同期操作の状態）
// ============================================================================

/// 非同期操作の状態
pub enum OperationStatus {
    Adding(String),
    Updating(String),
    UpdatingAll,
    Removing(String),
}

// ============================================================================
// CacheState（タブ切替時の保持状態）
// ============================================================================

/// キャッシュ状態（タブ切替時に保持）
#[derive(Debug, Default)]
pub struct CacheState {
    pub selected_id: Option<String>,
}

// ============================================================================
// DetailAction（詳細画面のアクション）
// ============================================================================

/// マーケットプレイス詳細画面のアクション
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DetailAction {
    Update,
    Remove,
    ShowPlugins,
    Back,
}

impl DetailAction {
    pub fn all() -> &'static [DetailAction] {
        &[
            DetailAction::Update,
            DetailAction::Remove,
            DetailAction::ShowPlugins,
            DetailAction::Back,
        ]
    }

    pub fn label(&self) -> &'static str {
        match self {
            DetailAction::Update => "Update",
            DetailAction::Remove => "Remove",
            DetailAction::ShowPlugins => "Show plugins",
            DetailAction::Back => "Back to list",
        }
    }

    pub fn style(&self) -> ratatui::prelude::Style {
        use ratatui::prelude::*;
        match self {
            DetailAction::Update => Style::default().fg(Color::Green),
            DetailAction::Remove => Style::default().fg(Color::Red),
            _ => Style::default(),
        }
    }
}

// ============================================================================
// AddFormModel（追加フォームの内部状態）
// ============================================================================

/// AddForm の内部状態
pub enum AddFormModel {
    /// source（owner/repo）入力画面
    Source {
        source_input: String,
        error_message: Option<String>,
    },
    /// name 入力画面
    Name {
        source: String,
        name_input: String,
        default_name: String,
        error_message: Option<String>,
    },
    /// 確認画面
    Confirm {
        source: String,
        name: String,
        error_message: Option<String>,
    },
}

// ============================================================================
// Model（画面状態）
// ============================================================================

/// Marketplaces タブの画面状態
pub enum Model {
    /// マーケットプレイス一覧
    MarketList {
        selected_id: Option<String>,
        state: ListState,
        operation_status: Option<OperationStatus>,
        error_message: Option<String>,
    },
    /// マーケットプレイス詳細（アクションメニュー）
    MarketDetail {
        marketplace_name: String,
        state: ListState,
        error_message: Option<String>,
    },
    /// プラグイン一覧表示
    PluginList {
        marketplace_name: String,
        selected_idx: usize,
        state: ListState,
        /// キャッシュ済みプラグインリスト（ディスクI/O回避）
        plugins: Vec<(String, Option<String>)>,
    },
    /// 新規マーケットプレイス追加フォーム
    AddForm(AddFormModel),
}

impl Model {
    /// 新しいモデルを作成
    pub fn new(data: &DataStore) -> Self {
        let selected_id = data.marketplaces.first().map(|m| m.name.clone());
        let mut state = ListState::default();
        // マーケットプレイスがあれば最初を選択、なければ "+ Add new" (index 0) を選択
        state.select(Some(0));
        Model::MarketList {
            selected_id,
            state,
            operation_status: None,
            error_message: None,
        }
    }

    /// キャッシュから復元
    pub fn from_cache(data: &DataStore, cache: &CacheState) -> Self {
        let selected_id = cache
            .selected_id
            .clone()
            .filter(|id| data.find_marketplace(id).is_some())
            .or_else(|| data.marketplaces.first().map(|m| m.name.clone()));

        let index = selected_id
            .as_ref()
            .and_then(|id| data.marketplace_index(id))
            .unwrap_or(0);

        let mut state = ListState::default();
        state.select(Some(index));

        Model::MarketList {
            selected_id,
            state,
            operation_status: None,
            error_message: None,
        }
    }

    /// キャッシュ状態を取得
    pub fn to_cache(&self) -> CacheState {
        match self {
            Model::MarketList { selected_id, .. } => CacheState {
                selected_id: selected_id.clone(),
            },
            Model::MarketDetail {
                marketplace_name, ..
            } => CacheState {
                selected_id: Some(marketplace_name.clone()),
            },
            Model::PluginList {
                marketplace_name, ..
            } => CacheState {
                selected_id: Some(marketplace_name.clone()),
            },
            Model::AddForm(_) => CacheState { selected_id: None },
        }
    }

    /// トップレベル（タブ切替可能）かどうか
    pub fn is_top_level(&self) -> bool {
        matches!(
            self,
            Model::MarketList {
                operation_status: None,
                ..
            }
        )
    }

    /// フォームがアクティブかどうか
    pub fn is_form_active(&self) -> bool {
        matches!(self, Model::AddForm(_))
    }
}

// ============================================================================
// Msg（メッセージ）
// ============================================================================

/// Marketplaces タブへのメッセージ
pub enum Msg {
    Up,
    Down,
    Enter,
    Back,
    FormInput(char),
    FormBackspace,
    UpdateMarket,
    UpdateAll,
    ExecuteAdd,
    ExecuteUpdate,
    ExecuteRemove,
}

/// キーコードをメッセージに変換
pub fn key_to_msg(key: KeyCode, model: &Model) -> Option<Msg> {
    if model.is_form_active() {
        // AddForm 状態ではフォーム入力として処理
        match key {
            KeyCode::Up => Some(Msg::Up),
            KeyCode::Down => Some(Msg::Down),
            KeyCode::Enter => Some(Msg::Enter),
            KeyCode::Esc => Some(Msg::Back),
            KeyCode::Backspace => Some(Msg::FormBackspace),
            KeyCode::Char(c) => Some(Msg::FormInput(c)),
            _ => None,
        }
    } else {
        match key {
            KeyCode::Up | KeyCode::Char('k') => Some(Msg::Up),
            KeyCode::Down | KeyCode::Char('j') => Some(Msg::Down),
            KeyCode::Enter => Some(Msg::Enter),
            KeyCode::Esc => Some(Msg::Back),
            KeyCode::Char('u') => Some(Msg::UpdateMarket),
            KeyCode::Char('U') => Some(Msg::UpdateAll),
            _ => None,
        }
    }
}

#[cfg(test)]
#[path = "model_test.rs"]
mod model_test;

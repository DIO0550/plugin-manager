//! Marketplaces タブの Model/Msg 定義
//!
//! 画面状態とメッセージ型を定義。

use crate::component::Scope;
use crate::marketplace::PluginSource;
use crate::tui::manager::core::{DataStore, SelectionState};
use crossterm::event::KeyCode;
use ratatui::widgets::ListState;
use std::collections::HashSet;

/// 非同期操作の状態
pub enum OperationStatus {
    Updating(String),
    UpdatingAll,
    Removing(String),
}

/// キャッシュ状態（タブ切替時に保持）
#[derive(Debug, Default)]
pub struct CacheState {
    pub selected_id: Option<String>,
}

/// マーケットプレイス詳細画面のアクション
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DetailAction {
    Update,
    Remove,
    ShowPlugins,
    BrowsePlugins,
    Back,
}

impl DetailAction {
    pub fn all() -> &'static [DetailAction] {
        &[
            DetailAction::Update,
            DetailAction::Remove,
            DetailAction::ShowPlugins,
            DetailAction::BrowsePlugins,
            DetailAction::Back,
        ]
    }

    pub fn label(&self) -> &'static str {
        match self {
            DetailAction::Update => "Update",
            DetailAction::Remove => "Remove",
            DetailAction::ShowPlugins => "Show plugins",
            DetailAction::BrowsePlugins => "Browse plugins",
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

/// マーケットプレイスのプラグイン情報（ブラウズ画面用）
pub struct BrowsePlugin {
    pub name: String,
    pub description: Option<String>,
    pub version: Option<String>,
    pub source: PluginSource,
    pub installed: bool,
}

/// プラグインインストール結果（1件分）
pub struct PluginInstallResult {
    pub plugin_name: String,
    pub success: bool,
    pub error: Option<String>,
}

/// インストールサマリー
pub struct InstallSummary {
    pub results: Vec<PluginInstallResult>,
    pub total: usize,
    pub succeeded: usize,
    pub failed: usize,
}

/// Marketplaces タブの画面状態
pub enum Model {
    /// マーケットプレイス一覧
    MarketList {
        selection: SelectionState<String>,
        operation_status: Option<OperationStatus>,
        error_message: Option<String>,
    },
    /// マーケットプレイス詳細（アクションメニュー）
    MarketDetail {
        marketplace_name: String,
        state: ListState,
        error_message: Option<String>,
        /// ブラウズ状態保持（再入時に復元）
        browse_plugins: Option<Vec<BrowsePlugin>>,
        /// ブラウズ選択状態保持（再入時に復元）
        browse_selected: Option<HashSet<String>>,
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
    /// プラグインブラウズ画面
    PluginBrowse {
        marketplace_name: String,
        plugins: Vec<BrowsePlugin>,
        selected_plugins: HashSet<String>,
        highlighted_idx: usize,
        state: ListState,
    },
    /// ターゲット選択画面
    TargetSelect {
        marketplace_name: String,
        plugins: Vec<BrowsePlugin>,
        selected_plugins: HashSet<String>,
        targets: Vec<(String, String, bool)>,
        highlighted_idx: usize,
        state: ListState,
    },
    /// スコープ選択画面
    ScopeSelect {
        marketplace_name: String,
        plugins: Vec<BrowsePlugin>,
        selected_plugins: HashSet<String>,
        target_names: Vec<String>,
        highlighted_idx: usize,
        state: ListState,
    },
    /// インストール実行中
    Installing {
        marketplace_name: String,
        plugins: Vec<BrowsePlugin>,
        plugin_names: Vec<String>,
        target_names: Vec<String>,
        scope: Scope,
        current_idx: usize,
        total: usize,
    },
    /// インストール結果表示
    InstallResult {
        marketplace_name: String,
        plugins: Vec<BrowsePlugin>,
        summary: InstallSummary,
    },
}

impl Model {
    /// 新しいモデルを作成
    ///
    /// # Arguments
    ///
    /// * `data` - Data store providing the marketplace list.
    pub fn new(data: &DataStore) -> Self {
        let selected_id = data.marketplaces.first().map(|m| m.name.clone());
        // マーケットプレイスがあれば最初を選択、なければ "+ Add new" (index 0) を選択
        Model::MarketList {
            selection: SelectionState::new(selected_id, Some(0)),
            operation_status: None,
            error_message: None,
        }
    }

    /// キャッシュから復元
    ///
    /// # Arguments
    ///
    /// * `data` - Data store providing the marketplace list.
    /// * `cache` - Previously saved cache state to restore from.
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

        Model::MarketList {
            selection: SelectionState::new(selected_id, Some(index)),
            operation_status: None,
            error_message: None,
        }
    }

    /// キャッシュ状態を取得
    pub fn to_cache(&self) -> CacheState {
        match self {
            Model::MarketList { selection, .. } => CacheState {
                selected_id: selection.selected_id.clone(),
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
            Model::PluginBrowse {
                marketplace_name, ..
            }
            | Model::TargetSelect {
                marketplace_name, ..
            }
            | Model::ScopeSelect {
                marketplace_name, ..
            }
            | Model::Installing {
                marketplace_name, ..
            }
            | Model::InstallResult {
                marketplace_name, ..
            } => CacheState {
                selected_id: Some(marketplace_name.clone()),
            },
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
    ExecuteUpdate,
    ExecuteRemove,
    ToggleSelect,
    StartInstall,
    ExecuteInstall,
    ConfirmTargets,
    ConfirmScope,
    BackToPluginBrowse,
}

/// キーコードをメッセージに変換
///
/// # Arguments
///
/// * `key` - Raw key code received from crossterm.
/// * `model` - Current marketplaces model used to disambiguate bindings.
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
        match model {
            Model::PluginBrowse { .. } => match key {
                KeyCode::Char(' ') => Some(Msg::ToggleSelect),
                KeyCode::Char('i') | KeyCode::Enter => Some(Msg::StartInstall),
                KeyCode::Up | KeyCode::Char('k') => Some(Msg::Up),
                KeyCode::Down | KeyCode::Char('j') => Some(Msg::Down),
                KeyCode::Esc => Some(Msg::Back),
                _ => None,
            },
            Model::TargetSelect { .. } => match key {
                KeyCode::Char(' ') => Some(Msg::ToggleSelect),
                KeyCode::Enter => Some(Msg::ConfirmTargets),
                KeyCode::Up | KeyCode::Char('k') => Some(Msg::Up),
                KeyCode::Down | KeyCode::Char('j') => Some(Msg::Down),
                KeyCode::Esc => Some(Msg::Back),
                _ => None,
            },
            Model::ScopeSelect { .. } => match key {
                KeyCode::Enter => Some(Msg::ConfirmScope),
                KeyCode::Up | KeyCode::Char('k') => Some(Msg::Up),
                KeyCode::Down | KeyCode::Char('j') => Some(Msg::Down),
                KeyCode::Esc => Some(Msg::Back),
                _ => None,
            },
            Model::Installing { .. } => None,
            Model::InstallResult { .. } => match key {
                KeyCode::Enter | KeyCode::Esc => Some(Msg::BackToPluginBrowse),
                _ => None,
            },
            _ => match key {
                KeyCode::Up | KeyCode::Char('k') => Some(Msg::Up),
                KeyCode::Down | KeyCode::Char('j') => Some(Msg::Down),
                KeyCode::Enter => Some(Msg::Enter),
                KeyCode::Esc => Some(Msg::Back),
                KeyCode::Char('u') => Some(Msg::UpdateMarket),
                KeyCode::Char('U') => Some(Msg::UpdateAll),
                _ => None,
            },
        }
    }
}

#[cfg(test)]
#[path = "model_test.rs"]
mod model_test;

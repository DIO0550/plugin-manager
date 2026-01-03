//! プラグイン管理 TUI の Elm Architecture ベースのアプリケーション構造
//!
//! - `Model`: アプリケーション全体の状態（データ + 画面 + キャッシュ）
//! - `Screen`: アクティブ画面の状態
//! - `Msg`: アプリケーションへのメッセージ
//! - `ScreenCache`: タブ切替時に保持する軽量な状態

use super::data::DataStore;
use crate::tui::manager::screens::{discover, errors, installed, marketplaces};
use crossterm::event::KeyCode;
use ratatui::prelude::*;

// ============================================================================
// Screen Cache（タブ切替時の状態保持）
// ============================================================================

/// タブ切替時に保持する軽量な状態
#[derive(Debug, Default)]
pub struct ScreenCache {
    pub installed: installed::CacheState,
    pub discover: discover::CacheState,
    pub marketplaces: marketplaces::CacheState,
}

// ============================================================================
// Tab（タブ種別）
// ============================================================================

/// タブ種別
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Tab {
    Discover,
    #[default]
    Installed,
    Marketplaces,
    Errors,
}

impl Tab {
    pub fn all() -> &'static [Tab] {
        &[Tab::Discover, Tab::Installed, Tab::Marketplaces, Tab::Errors]
    }

    pub fn title(&self) -> &'static str {
        match self {
            Tab::Discover => "Discover",
            Tab::Installed => "Installed",
            Tab::Marketplaces => "Marketplaces",
            Tab::Errors => "Errors",
        }
    }

    pub fn index(&self) -> usize {
        match self {
            Tab::Discover => 0,
            Tab::Installed => 1,
            Tab::Marketplaces => 2,
            Tab::Errors => 3,
        }
    }

    pub fn from_index(index: usize) -> Self {
        match index % 4 {
            0 => Tab::Discover,
            1 => Tab::Installed,
            2 => Tab::Marketplaces,
            _ => Tab::Errors,
        }
    }

    pub fn next(&self) -> Self {
        Self::from_index(self.index() + 1)
    }

    pub fn prev(&self) -> Self {
        Self::from_index(self.index() + 3)
    }
}

// ============================================================================
// Screen（アクティブ画面の状態）
// ============================================================================

/// アクティブ画面の状態
pub enum Screen {
    Installed(installed::Model),
    Discover(discover::Model),
    Marketplaces(marketplaces::Model),
    Errors(errors::Model),
}

impl Screen {
    /// 現在のタブを取得
    pub fn tab(&self) -> Tab {
        match self {
            Screen::Installed(_) => Tab::Installed,
            Screen::Discover(_) => Tab::Discover,
            Screen::Marketplaces(_) => Tab::Marketplaces,
            Screen::Errors(_) => Tab::Errors,
        }
    }

    /// トップレベル（タブ切替可能な状態）かどうか
    pub fn is_top_level(&self) -> bool {
        match self {
            Screen::Installed(m) => m.is_top_level(),
            Screen::Discover(_) => true,
            Screen::Marketplaces(_) => true,
            Screen::Errors(_) => true,
        }
    }
}

// ============================================================================
// Msg（アプリケーションへのメッセージ）
// ============================================================================

/// アプリケーションへのメッセージ
pub enum Msg {
    /// 終了
    Quit,
    /// 次のタブへ
    NextTab,
    /// 前のタブへ
    PrevTab,
    /// Installed タブのメッセージ
    Installed(installed::Msg),
    /// Discover タブのメッセージ
    Discover(discover::Msg),
    /// Marketplaces タブのメッセージ
    Marketplaces(marketplaces::Msg),
    /// Errors タブのメッセージ
    Errors(errors::Msg),
}

// ============================================================================
// Model（アプリケーション全体の状態）
// ============================================================================

/// アプリケーション全体の状態
pub struct Model {
    /// 共有データストア
    pub data: DataStore,
    /// アクティブ画面
    pub screen: Screen,
    /// タブキャッシュ
    pub cache: ScreenCache,
    /// 終了フラグ
    pub should_quit: bool,
}

impl Model {
    /// 新しいモデルを作成
    pub fn new() -> std::io::Result<Self> {
        let data = DataStore::new()?;
        let screen = Screen::Installed(installed::Model::new(&data));

        Ok(Self {
            data,
            screen,
            cache: ScreenCache::default(),
            should_quit: false,
        })
    }

    /// キー入力をメッセージに変換
    pub fn key_to_msg(&self, key: KeyCode) -> Option<Msg> {
        match key {
            KeyCode::Char('q') => Some(Msg::Quit),
            KeyCode::Tab if self.screen.is_top_level() => Some(Msg::NextTab),
            KeyCode::BackTab if self.screen.is_top_level() => Some(Msg::PrevTab),
            _ => {
                // 画面固有のキー処理
                match &self.screen {
                    Screen::Installed(_) => installed::key_to_msg(key).map(Msg::Installed),
                    Screen::Discover(_) => discover::key_to_msg(key).map(Msg::Discover),
                    Screen::Marketplaces(_) => marketplaces::key_to_msg(key).map(Msg::Marketplaces),
                    Screen::Errors(_) => errors::key_to_msg(key).map(Msg::Errors),
                }
            }
        }
    }
}

// ============================================================================
// update（状態更新）
// ============================================================================

/// メッセージに応じて状態を更新
pub fn update(model: &mut Model, msg: Msg) {
    match msg {
        Msg::Quit => {
            model.should_quit = true;
        }
        Msg::NextTab => {
            switch_tab(model, model.screen.tab().next());
        }
        Msg::PrevTab => {
            switch_tab(model, model.screen.tab().prev());
        }
        Msg::Installed(msg) => {
            if let Screen::Installed(m) = &mut model.screen {
                installed::update(m, msg, &mut model.data);
            }
        }
        Msg::Discover(msg) => {
            if let Screen::Discover(m) = &mut model.screen {
                discover::update(m, msg, &model.data);
            }
        }
        Msg::Marketplaces(msg) => {
            if let Screen::Marketplaces(m) = &mut model.screen {
                marketplaces::update(m, msg, &model.data);
            }
        }
        Msg::Errors(msg) => {
            if let Screen::Errors(m) = &mut model.screen {
                errors::update(m, msg, &model.data);
            }
        }
    }
}

/// タブを切り替え
fn switch_tab(model: &mut Model, new_tab: Tab) {
    // 現在の画面状態をキャッシュに保存
    match &model.screen {
        Screen::Installed(m) => {
            model.cache.installed = m.to_cache();
        }
        Screen::Discover(m) => {
            model.cache.discover = m.to_cache();
        }
        Screen::Marketplaces(m) => {
            model.cache.marketplaces = m.to_cache();
        }
        Screen::Errors(_) => {
            // Errors タブはキャッシュ不要
        }
    }

    // 新しい画面を作成（キャッシュから復元）
    model.screen = match new_tab {
        Tab::Installed => {
            Screen::Installed(installed::Model::from_cache(&model.data, &model.cache.installed))
        }
        Tab::Discover => {
            Screen::Discover(discover::Model::from_cache(&model.data, &model.cache.discover))
        }
        Tab::Marketplaces => Screen::Marketplaces(marketplaces::Model::from_cache(
            &model.data,
            &model.cache.marketplaces,
        )),
        Tab::Errors => Screen::Errors(errors::Model::new(&model.data)),
    };
}

// ============================================================================
// view（描画）
// ============================================================================

/// 画面を描画
pub fn view(f: &mut Frame, model: &Model) {
    match &model.screen {
        Screen::Installed(m) => installed::view(f, m, &model.data),
        Screen::Discover(m) => discover::view(f, m, &model.data),
        Screen::Marketplaces(m) => marketplaces::view(f, m, &model.data),
        Screen::Errors(m) => errors::view(f, m, &model.data),
    }
}

//! プラグイン管理 TUI の Elm Architecture ベースのアプリケーション構造
//!
//! - `Model`: アプリケーション全体の状態（データ + 画面 + キャッシュ）
//! - `Screen`: アクティブ画面の状態
//! - `Msg`: アプリケーションへのメッセージ
//! - `ScreenCache`: タブ切替時に保持する軽量な状態

use super::data::DataStore;
use super::filter::filter_plugins;
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
        &[
            Tab::Discover,
            Tab::Installed,
            Tab::Marketplaces,
            Tab::Errors,
        ]
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
            Screen::Marketplaces(m) => m.is_top_level(),
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
    /// フィルタにフォーカス移動
    FilterFocus,
    /// フィルタからフォーカス解除（リストへ戻る）
    FilterUnfocus,
    /// フィルタ文字入力
    FilterInput(char),
    /// フィルタ文字削除
    FilterBackspace,
    /// フィルタクリア
    FilterClear,
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
    /// フィルタテキスト（全タブ共通）
    pub filter_text: String,
    /// フィルタ入力欄にフォーカスしているか
    pub filter_focused: bool,
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
            filter_text: String::new(),
            filter_focused: false,
        })
    }

    /// キー入力をメッセージに変換
    pub fn key_to_msg(&self, key: KeyCode) -> Option<Msg> {
        let is_top_level = self.screen.is_top_level();

        if self.filter_focused {
            // フィルタにフォーカス中のキー処理
            match key {
                KeyCode::Esc if !self.filter_text.is_empty() => Some(Msg::FilterClear),
                KeyCode::Esc => Some(Msg::FilterUnfocus),
                KeyCode::Down | KeyCode::Enter => Some(Msg::FilterUnfocus),
                KeyCode::Tab | KeyCode::Right if is_top_level => Some(Msg::NextTab),
                KeyCode::BackTab | KeyCode::Left if is_top_level => Some(Msg::PrevTab),
                KeyCode::Backspace => Some(Msg::FilterBackspace),
                KeyCode::Char(c) => Some(Msg::FilterInput(c)),
                _ => None,
            }
        } else {
            // フォームがアクティブな場合は 'q' を画面に委譲
            let form_active = matches!(&self.screen, Screen::Marketplaces(m) if m.is_form_active());

            // リスト（通常）フォーカス時のキー処理
            match key {
                KeyCode::Char('q') if !form_active => Some(Msg::Quit),
                KeyCode::Tab | KeyCode::Right if is_top_level => Some(Msg::NextTab),
                KeyCode::BackTab | KeyCode::Left if is_top_level => Some(Msg::PrevTab),
                // 画面固有のキー処理に委譲
                // フィルタへのフォーカス移動は installed::update の返り値で処理される
                _ => match &self.screen {
                    Screen::Installed(_) => installed::key_to_msg(key).map(Msg::Installed),
                    Screen::Discover(_) => discover::key_to_msg(key).map(Msg::Discover),
                    Screen::Marketplaces(m) => {
                        marketplaces::key_to_msg(key, m).map(Msg::Marketplaces)
                    }
                    Screen::Errors(_) => errors::key_to_msg(key).map(Msg::Errors),
                },
            }
        }
    }
}

// ============================================================================
// update（状態更新）
// ============================================================================

/// app::update() の戻り値
pub struct AppUpdateEffect {
    /// 描画後にバッチ更新を実行すべき
    pub needs_execute_batch: bool,
}

impl AppUpdateEffect {
    fn none() -> Self {
        Self {
            needs_execute_batch: false,
        }
    }
}

/// メッセージに応じて状態を更新
pub fn update(model: &mut Model, msg: Msg) -> AppUpdateEffect {
    match msg {
        Msg::Quit => {
            model.should_quit = true;
            AppUpdateEffect::none()
        }
        Msg::NextTab => {
            model.filter_focused = false;
            switch_tab(model, model.screen.tab().next());
            AppUpdateEffect::none()
        }
        Msg::PrevTab => {
            model.filter_focused = false;
            switch_tab(model, model.screen.tab().prev());
            AppUpdateEffect::none()
        }
        Msg::FilterFocus => {
            model.filter_focused = true;
            AppUpdateEffect::none()
        }
        Msg::FilterUnfocus => {
            model.filter_focused = false;
            AppUpdateEffect::none()
        }
        Msg::FilterInput(c) => {
            model.filter_text.push(c);
            clamp_selection(model);
            AppUpdateEffect::none()
        }
        Msg::FilterBackspace => {
            model.filter_text.pop();
            clamp_selection(model);
            AppUpdateEffect::none()
        }
        Msg::FilterClear => {
            model.filter_text.clear();
            clamp_selection(model);
            AppUpdateEffect::none()
        }
        Msg::Installed(msg) => {
            if let Screen::Installed(m) = &mut model.screen {
                let effect = installed::update(m, msg, &mut model.data, &model.filter_text);
                if effect.should_focus_filter {
                    model.filter_focused = true;
                }
                AppUpdateEffect {
                    needs_execute_batch: effect.needs_execute_batch,
                }
            } else {
                AppUpdateEffect::none()
            }
        }
        Msg::Discover(msg) => {
            if let Screen::Discover(m) = &mut model.screen {
                discover::update(m, msg, &model.data);
            }
            AppUpdateEffect::none()
        }
        Msg::Marketplaces(msg) => {
            if let Screen::Marketplaces(m) = &mut model.screen {
                let effect = marketplaces::update(m, msg, &mut model.data);
                if effect.should_focus_filter {
                    model.filter_focused = true;
                }
                AppUpdateEffect {
                    needs_execute_batch: effect.needs_execute_batch,
                }
            } else {
                AppUpdateEffect::none()
            }
        }
        Msg::Errors(msg) => {
            if let Screen::Errors(m) = &mut model.screen {
                errors::update(m, msg, &model.data);
            }
            AppUpdateEffect::none()
        }
    }
}

/// フィルタ変更後に選択状態を整合させる
fn clamp_selection(model: &mut Model) {
    if let Screen::Installed(m) = &mut model.screen {
        let filtered = filter_plugins(&model.data.plugins, &model.filter_text);
        if let installed::Model::PluginList {
            selected_id, state, ..
        } = m
        {
            if let Some(id) = selected_id.as_ref() {
                // 現在の選択が絞り込み結果に含まれるか
                if let Some(idx) = filtered.iter().position(|p| &p.name == id) {
                    state.select(Some(idx));
                } else if !filtered.is_empty() {
                    state.select(Some(0));
                    *selected_id = Some(filtered[0].name.clone());
                } else {
                    state.select(None);
                    *selected_id = None;
                }
            } else if !filtered.is_empty() {
                state.select(Some(0));
                *selected_id = Some(filtered[0].name.clone());
            } else {
                state.select(None);
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
        Tab::Installed => Screen::Installed(installed::Model::from_cache(
            &model.data,
            &model.cache.installed,
        )),
        Tab::Discover => Screen::Discover(discover::Model::from_cache(
            &model.data,
            &model.cache.discover,
        )),
        Tab::Marketplaces => Screen::Marketplaces(marketplaces::Model::from_cache(
            &model.data,
            &model.cache.marketplaces,
        )),
        Tab::Errors => Screen::Errors(errors::Model::new(&model.data)),
    };

    // Installed タブ復元後にフィルタ済みリストと選択状態を整合
    if new_tab == Tab::Installed {
        clamp_selection(model);
    }
}

// ============================================================================
// view（描画）
// ============================================================================

/// 画面を描画
pub fn view(f: &mut Frame, model: &Model) {
    match &model.screen {
        Screen::Installed(m) => {
            installed::view(f, m, &model.data, &model.filter_text, model.filter_focused)
        }
        Screen::Discover(m) => {
            discover::view(f, m, &model.data, &model.filter_text, model.filter_focused)
        }
        Screen::Marketplaces(m) => {
            marketplaces::view(f, m, &model.data, &model.filter_text, model.filter_focused)
        }
        Screen::Errors(m) => {
            errors::view(f, m, &model.data, &model.filter_text, model.filter_focused)
        }
    }
}

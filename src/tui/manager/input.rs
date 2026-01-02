//! プラグイン管理 TUI の入力処理
//!
//! キー入力とナビゲーション処理。

use super::state::{ManagerApp, ManagerScreen, ManagerTab};
use crossterm::event::KeyCode;

impl ManagerApp {
    /// 次のタブに移動
    fn next_tab(&mut self) {
        let next_index = (self.current_tab.index() + 1) % 4;
        self.current_tab = ManagerTab::from_index(next_index);
    }

    /// 前のタブに移動
    fn prev_tab(&mut self) {
        let prev_index = (self.current_tab.index() + 3) % 4;
        self.current_tab = ManagerTab::from_index(prev_index);
    }

    /// 現在の画面に応じたリスト長を取得
    fn current_list_len(&self) -> usize {
        match &self.screen {
            ManagerScreen::PluginList => self.plugins.len(),
            ManagerScreen::ComponentTypes(plugin_idx) => {
                // 空でないコンポーネント種別の数（0の場合は選択不可）
                if let Some(plugin) = self.plugins.get(*plugin_idx) {
                    let len = self.available_types(plugin).len();
                    if len == 0 {
                        1
                    } else {
                        len
                    } // 0だとselect_prev/nextが動かないので1を返す
                } else {
                    0
                }
            }
            ManagerScreen::ComponentList(plugin_idx, type_idx) => {
                if let Some(plugin) = self.plugins.get(*plugin_idx) {
                    let types = self.available_types(plugin);
                    if let Some(comp_type) = types.get(*type_idx) {
                        self.get_components(plugin, *comp_type).len()
                    } else {
                        0
                    }
                } else {
                    0
                }
            }
        }
    }

    /// 現在の ListState を取得
    fn current_state_mut(&mut self) -> &mut ratatui::widgets::ListState {
        match &self.screen {
            ManagerScreen::PluginList => &mut self.list_state,
            ManagerScreen::ComponentTypes(_) => &mut self.component_type_state,
            ManagerScreen::ComponentList(_, _) => &mut self.component_state,
        }
    }

    /// 選択を上に移動
    fn select_prev(&mut self) {
        let len = self.current_list_len();
        if len == 0 {
            return;
        }
        let state = self.current_state_mut();
        let current = state.selected().unwrap_or(0);
        let prev = current.saturating_sub(1);
        state.select(Some(prev));
    }

    /// 選択を下に移動
    fn select_next(&mut self) {
        let len = self.current_list_len();
        if len == 0 {
            return;
        }
        let state = self.current_state_mut();
        let current = state.selected().unwrap_or(0);
        let next = (current + 1).min(len.saturating_sub(1));
        state.select(Some(next));
    }

    /// 次の階層へ遷移
    fn enter(&mut self) {
        match &self.screen {
            ManagerScreen::PluginList => {
                if let Some(plugin_idx) = self.list_state.selected() {
                    if self.plugins.get(plugin_idx).is_some() {
                        self.component_type_state.select(Some(0));
                        self.screen = ManagerScreen::ComponentTypes(plugin_idx);
                    }
                }
            }
            ManagerScreen::ComponentTypes(plugin_idx) => {
                if let Some(type_idx) = self.component_type_state.selected() {
                    if let Some(plugin) = self.plugins.get(*plugin_idx) {
                        let types = self.available_types(plugin);
                        if let Some(comp_type) = types.get(type_idx) {
                            if !self.get_components(plugin, *comp_type).is_empty() {
                                self.component_state.select(Some(0));
                                self.screen = ManagerScreen::ComponentList(*plugin_idx, type_idx);
                            }
                        }
                    }
                }
            }
            ManagerScreen::ComponentList(_, _) => {
                // 最下層なので何もしない
            }
        }
    }

    /// 前の階層へ戻る
    fn back(&mut self) {
        match &self.screen {
            ManagerScreen::PluginList => {
                self.should_quit = true;
            }
            ManagerScreen::ComponentTypes(_) => {
                self.screen = ManagerScreen::PluginList;
            }
            ManagerScreen::ComponentList(plugin_idx, _) => {
                self.screen = ManagerScreen::ComponentTypes(*plugin_idx);
            }
        }
    }

    /// キー入力を処理
    pub(super) fn handle_key(&mut self, key: KeyCode) {
        match key {
            KeyCode::Char('q') => self.should_quit = true,
            KeyCode::Esc => self.back(),
            KeyCode::Tab => {
                if matches!(self.screen, ManagerScreen::PluginList) {
                    self.next_tab();
                }
            }
            KeyCode::BackTab => {
                if matches!(self.screen, ManagerScreen::PluginList) {
                    self.prev_tab();
                }
            }
            KeyCode::Up | KeyCode::Char('k') => self.select_prev(),
            KeyCode::Down | KeyCode::Char('j') => self.select_next(),
            KeyCode::Enter => self.enter(),
            _ => {}
        }
    }
}

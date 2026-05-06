//! TUI 画面で共有するリスト選択状態。

use ratatui::widgets::ListState;

/// semantic な選択 ID と ratatui のリストカーソルを束ねた選択状態。
#[derive(Debug, Clone, Default)]
pub struct SelectionState<K> {
    pub selected_id: Option<K>,
    pub state: ListState,
}

impl<K> SelectionState<K> {
    /// 選択 ID とリスト index から選択状態を作成する。
    pub fn new(selected_id: Option<K>, selected_index: Option<usize>) -> Self {
        let mut state = ListState::default();
        state.select(selected_index);
        Self { selected_id, state }
    }

    pub fn selected_id(&self) -> Option<&K> {
        self.selected_id.as_ref()
    }

    pub fn selected_id_mut(&mut self) -> &mut Option<K> {
        &mut self.selected_id
    }

    pub fn list_state(&self) -> &ListState {
        &self.state
    }

    pub fn list_state_mut(&mut self) -> &mut ListState {
        &mut self.state
    }

    pub fn set(&mut self, selected_id: Option<K>, selected_index: Option<usize>) {
        self.selected_id = selected_id;
        self.state.select(selected_index);
    }
}

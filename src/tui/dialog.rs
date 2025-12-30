//! TUI選択ダイアログの基盤
//!
//! multi_select と single_select の共通機能を提供する。

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::terminal::{self, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::ExecutableCommand;
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph};
use std::io::{self, stdout};

/// 選択項目
#[derive(Debug, Clone)]
pub struct SelectItem<T: Clone> {
    /// 表示ラベル
    pub label: String,
    /// 説明文
    pub description: Option<String>,
    /// 値
    pub value: T,
    /// 選択状態
    pub selected: bool,
    /// 有効かどうか（無効な場合はグレーアウト）
    pub enabled: bool,
}

impl<T: Clone> SelectItem<T> {
    /// 新しい選択項目を作成
    pub fn new(label: impl Into<String>, value: T) -> Self {
        Self {
            label: label.into(),
            description: None,
            value,
            selected: false,
            enabled: true,
        }
    }

    /// 説明文を設定
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// 選択状態を設定
    pub fn with_selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    /// 有効状態を設定
    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
}

/// 複数選択の結果
#[derive(Debug)]
pub struct MultiSelectResult<T> {
    /// 選択された値のリスト
    pub selected: Vec<T>,
    /// キャンセルされたかどうか
    pub cancelled: bool,
}

/// 単一選択の結果
#[derive(Debug)]
pub struct SingleSelectResult<T> {
    /// 選択された値
    pub selected: Option<T>,
    /// キャンセルされたかどうか
    pub cancelled: bool,
}

/// 複数選択ダイアログを表示
pub fn multi_select<T: Clone>(
    title: &str,
    items: &mut [SelectItem<T>],
) -> io::Result<MultiSelectResult<T>> {
    // ターミナル設定
    terminal::enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(stdout());
    let mut terminal = Terminal::new(backend)?;

    let mut state = ListState::default();
    state.select(Some(0));

    let result = loop {
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints([Constraint::Min(3), Constraint::Length(2)])
                .split(f.area());

            // リスト表示
            let list_items: Vec<ListItem> = items
                .iter()
                .map(|item| {
                    let checkbox = if item.selected { "[x]" } else { "[ ]" };
                    let style = if item.enabled {
                        Style::default()
                    } else {
                        Style::default().fg(Color::DarkGray)
                    };

                    let text = match &item.description {
                        Some(desc) => format!("{} {}  {}", checkbox, item.label, desc),
                        None => format!("{} {}", checkbox, item.label),
                    };
                    ListItem::new(text).style(style)
                })
                .collect();

            let list = List::new(list_items)
                .block(Block::default().title(title).borders(Borders::ALL))
                .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
                .highlight_symbol("> ");

            f.render_stateful_widget(list, chunks[0], &mut state);

            // ヘルプ表示
            let help = Paragraph::new("↑/↓: move  space: toggle  enter: confirm  q/esc: cancel")
                .style(Style::default().fg(Color::DarkGray));
            f.render_widget(help, chunks[1]);
        })?;

        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => {
                        break MultiSelectResult {
                            selected: vec![],
                            cancelled: true,
                        };
                    }
                    KeyCode::Enter => {
                        break MultiSelectResult {
                            selected: items
                                .iter()
                                .filter(|i| i.selected && i.enabled)
                                .map(|i| i.value.clone())
                                .collect(),
                            cancelled: false,
                        };
                    }
                    KeyCode::Char(' ') => {
                        if let Some(i) = state.selected() {
                            if items[i].enabled {
                                items[i].selected = !items[i].selected;
                            }
                        }
                    }
                    KeyCode::Up | KeyCode::Char('k') => {
                        let i = state.selected().unwrap_or(0);
                        state.select(Some(i.saturating_sub(1)));
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        let i = state.selected().unwrap_or(0);
                        state.select(Some((i + 1).min(items.len().saturating_sub(1))));
                    }
                    _ => {}
                }
            }
        }
    };

    // ターミナルを復元
    terminal::disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;

    Ok(result)
}

/// 単一選択ダイアログを表示
pub fn single_select<T: Clone>(
    title: &str,
    items: &[SelectItem<T>],
) -> io::Result<SingleSelectResult<T>> {
    // ターミナル設定
    terminal::enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(stdout());
    let mut terminal = Terminal::new(backend)?;

    // 初期選択位置を探す
    let initial_index = items.iter().position(|i| i.selected).unwrap_or(0);
    let mut state = ListState::default();
    state.select(Some(initial_index));

    let result = loop {
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints([Constraint::Min(3), Constraint::Length(2)])
                .split(f.area());

            // リスト表示
            let list_items: Vec<ListItem> = items
                .iter()
                .enumerate()
                .map(|(idx, item)| {
                    let is_current = state.selected() == Some(idx);
                    let radio = if is_current { "(x)" } else { "( )" };
                    let style = if item.enabled {
                        Style::default()
                    } else {
                        Style::default().fg(Color::DarkGray)
                    };

                    let text = match &item.description {
                        Some(desc) => format!("{} {}  {}", radio, item.label, desc),
                        None => format!("{} {}", radio, item.label),
                    };
                    ListItem::new(text).style(style)
                })
                .collect();

            let list = List::new(list_items)
                .block(Block::default().title(title).borders(Borders::ALL))
                .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
                .highlight_symbol("> ");

            f.render_stateful_widget(list, chunks[0], &mut state);

            // ヘルプ表示
            let help = Paragraph::new("↑/↓: move  enter: select  q/esc: cancel")
                .style(Style::default().fg(Color::DarkGray));
            f.render_widget(help, chunks[1]);
        })?;

        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => {
                        break SingleSelectResult {
                            selected: None,
                            cancelled: true,
                        };
                    }
                    KeyCode::Enter => {
                        if let Some(i) = state.selected() {
                            if items[i].enabled {
                                break SingleSelectResult {
                                    selected: Some(items[i].value.clone()),
                                    cancelled: false,
                                };
                            }
                        }
                    }
                    KeyCode::Up | KeyCode::Char('k') => {
                        let i = state.selected().unwrap_or(0);
                        state.select(Some(i.saturating_sub(1)));
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        let i = state.selected().unwrap_or(0);
                        state.select(Some((i + 1).min(items.len().saturating_sub(1))));
                    }
                    _ => {}
                }
            }
        }
    };

    // ターミナルを復元
    terminal::disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;

    Ok(result)
}

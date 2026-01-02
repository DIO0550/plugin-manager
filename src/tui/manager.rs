//! プラグイン管理 TUI
//!
//! インストール済みプラグインの一覧表示と管理を行う TUI。
//!
//! ## モジュール構成
//!
//! - `state`: アプリケーション状態（Tab, Screen, App）
//! - `input`: キー入力処理
//! - `render`: 画面描画

mod input;
mod render;
mod state;

use crossterm::event::{self, Event, KeyEventKind};
use crossterm::terminal::{self, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::ExecutableCommand;
use ratatui::prelude::*;
use state::ManagerApp;
use std::io::{self, stdout};

/// TUI を実行
pub fn run() -> io::Result<()> {
    // ターミナル設定
    terminal::enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(stdout());
    let mut terminal = Terminal::new(backend)?;

    let mut app = ManagerApp::new()?;

    // メインループ
    while !app.should_quit {
        terminal.draw(|f| render::draw(f, &mut app))?;

        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                app.handle_key(key.code);
            }
        }
    }

    // ターミナルを復元
    terminal::disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;

    Ok(())
}

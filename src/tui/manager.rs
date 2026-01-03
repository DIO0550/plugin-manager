//! プラグイン管理 TUI
//!
//! インストール済みプラグインの一覧表示と管理を行う TUI。
//!
//! ## Elm Architecture
//!
//! このモジュールは Elm Architecture パターンを採用：
//! - `Model`: アプリケーション状態（data + screen + cache）
//! - `Msg`: 状態変更のトリガー
//! - `update`: Msg に応じて Model を更新
//! - `view`: Model から画面を描画
//!
//! ## モジュール構成
//!
//! - `core/`: コアモジュール
//!   - `app`: Model/Screen/Msg/update/view のトップレベル定義
//!   - `data`: 共有データストア（DataStore）
//!   - `common`: 共通 UI ユーティリティ
//! - `screens/`: 画面モジュール
//!   - `installed`: Installed タブ
//!   - `discover`: Discover タブ
//!   - `marketplaces`: Marketplaces タブ
//!   - `errors`: Errors タブ

mod core;
pub mod screens;

use core::{update, view, Model};
use crossterm::event::{self, Event, KeyEventKind};
use crossterm::terminal::{self, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::ExecutableCommand;
use ratatui::prelude::*;
use std::io::{self, stdout};

/// TUI を実行
pub fn run() -> io::Result<()> {
    // ターミナル設定
    terminal::enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(stdout());
    let mut terminal = Terminal::new(backend)?;

    let mut model = Model::new()?;

    // メインループ
    while !model.should_quit {
        terminal.draw(|f| view(f, &model))?;

        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                if let Some(msg) = model.key_to_msg(key.code) {
                    update(&mut model, msg);
                }
            }
        }
    }

    // ターミナルを復元
    terminal::disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;

    Ok(())
}

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
                    let effect = update(&mut model, msg);

                    // 2段階方式: Phase 1 後に描画してから Phase 2 メッセージを実行
                    // バッチ更新中のキー入力はキューに溜まるが、完了後に破棄する
                    //
                    // Note: バッチ更新中は入力がブロックされるため、'q' を押しても終了しない。
                    // 将来的な改善案:
                    // - 更新中であることを示すメッセージを表示
                    // - 'q' や Escape でキャンセル可能にする
                    // - 進捗インジケータ（例: "Updating plugin 3 of 10..."）を表示
                    if let Some(phase2_msg) = effect.phase2_msg {
                        terminal.draw(|f| view(f, &model))?;
                        update(&mut model, phase2_msg);
                        // バッチ更新中にキューされたキー入力を破棄
                        while event::poll(std::time::Duration::ZERO)? {
                            let _ = event::read()?;
                        }
                        terminal.draw(|f| view(f, &model))?;
                    }
                }
            }
        }
    }

    // ターミナルを復元
    terminal::disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;

    Ok(())
}

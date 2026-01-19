# TUIアーキテクチャ

PLMのTUI（ターミナルユーザーインターフェース）設計について説明します。

## 技術選定

| ライブラリ | 選定理由 |
|------------|----------|
| **ratatui** | Rust製TUIのデファクト、活発なメンテナンス |
| **crossterm** | クロスプラットフォームターミナル操作 |

## 画面構成

```
┌─────────────────────────────────────────────────────────────────┐
│  Discover    [Installed]    Marketplaces    Errors  (tab)       │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  cc-plugin @ DIO0550-marketplace                                │
│                                                                 │
│  Scope: user                                                    │
│  Version: 1.0.1                                                 │
│  プラグイン                                                      │
│                                                                 │
│  Author: DIO0550                                                │
│  Status: Enabled                                                │
│                                                                 │
│  Installed components:                                          │
│  • Commands: commit, review-test-code, fix-all-issues, ...      │
│  • Agents: git-commit-agent, tidy-first-reviewer, ...           │
│  • Hooks: PreToolUse                                            │
│                                                                 │
├─────────────────────────────────────────────────────────────────┤
│  > Disable plugin                                               │
│    Mark for update                                              │
│    Update now                                                   │
│    Uninstall                                                    │
│    View on GitHub                                               │
│    Back to plugin list                                          │
└─────────────────────────────────────────────────────────────────┘
```

## ディレクトリ構成

```
src/tui/
├── app.rs                # アプリケーション状態
├── ui.rs                 # UI描画
├── tabs/                 # 各タブ
│   ├── discover.rs       # マーケットプレイス検索
│   ├── installed.rs      # インストール済み管理
│   ├── marketplaces.rs   # マーケットプレイス管理
│   └── errors.rs         # エラー一覧
└── widgets/              # 再利用可能ウィジェット
    └── plugin_select.rs  # プラグイン選択ダイアログ
```

## アプリケーション状態

```rust
pub struct App {
    pub current_tab: Tab,
    pub plugins: Vec<CachedPlugin>,
    pub marketplaces: Vec<Marketplace>,
    pub selected_plugin: Option<usize>,
    pub selected_action: usize,
    pub errors: Vec<AppError>,
    pub dialog: Option<Dialog>,
}

pub enum Tab {
    Discover,
    Installed,
    Marketplaces,
    Errors,
}

pub enum Dialog {
    PluginSelect(PluginSelectState),
    Confirm(ConfirmState),
}
```

## タブ構成

| タブ | 内容 | 主な操作 |
|------|------|----------|
| **Discover** | マーケットプレイスからプラグイン検索 | インストール |
| **Installed** | インストール済みプラグイン管理 | 有効/無効、更新、削除 |
| **Marketplaces** | 登録済みマーケットプレイス一覧 | 追加、削除、更新 |
| **Errors** | エラー・警告一覧 | 詳細表示、クリア |

## キーバインド設計

### グローバル

| キー | 操作 |
|------|------|
| `Tab` / `Shift+Tab` | タブ切り替え |
| `q` | 終了 |
| `?` | ヘルプ表示 |
| `Esc` | ダイアログを閉じる / 戻る |

### リスト操作

| キー | 操作 |
|------|------|
| `↑` / `k` | 上に移動 |
| `↓` / `j` | 下に移動 |
| `Enter` | 選択 / アクション実行 |
| `Space` | チェックボックス切り替え |

### アクション

| キー | 操作 |
|------|------|
| `e` | 有効化 |
| `d` | 無効化 |
| `u` | 更新 |
| `x` | 削除 |
| `g` | GitHubを開く |

## アクション一覧

| アクション | 説明 | 実装 |
|------------|------|------|
| Disable/Enable plugin | プラグインの有効/無効切替 | キャッシュ更新 |
| Mark for update | 更新対象としてマーク | バッチ更新用 |
| Update now | 即座に更新 | GitHub API → キャッシュ更新 |
| Uninstall | プラグイン削除 | ファイル削除 + キャッシュ更新 |
| View on GitHub | リポジトリページを開く | `GitRepo.github_web_url()` |

## プラグイン選択ダイアログ

複数マーケットプレイスに同名プラグインがある場合に表示:

```
┌─────────────────────────────────────────────────────────────┐
│  Multiple plugins found: formatter                          │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  > [ ] formatter@company-tools                              │
│        v1.0.0 - Code formatting tool                        │
│                                                             │
│    [ ] formatter@anthropic                                  │
│        v2.0.0 - Advanced formatter with AI                  │
│                                                             │
├─────────────────────────────────────────────────────────────┤
│  [Enter] Select   [Esc] Cancel                              │
└─────────────────────────────────────────────────────────────┘
```

### 実装

```rust
pub struct PluginSelectState {
    pub query: String,
    pub matches: Vec<PluginMatch>,
    pub selected: usize,
}

impl PluginSelectState {
    pub fn selected_plugin(&self) -> Option<&PluginMatch> {
        self.matches.get(self.selected)
    }
}
```

## UI描画

```rust
pub fn draw(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // タブバー
            Constraint::Min(0),     // メインコンテンツ
            Constraint::Length(1),  // ステータスバー
        ])
        .split(f.area());

    draw_tabs(f, chunks[0], app);
    draw_content(f, chunks[1], app);
    draw_status(f, chunks[2], app);

    if let Some(dialog) = &app.dialog {
        draw_dialog(f, dialog);
    }
}
```

## イベントループ

```rust
pub async fn run(terminal: &mut Terminal<impl Backend>) -> Result<()> {
    let mut app = App::new()?;

    loop {
        terminal.draw(|f| draw(f, &app))?;

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match handle_key(key, &mut app).await {
                    Action::Quit => break,
                    Action::Continue => {}
                }
            }
        }
    }

    Ok(())
}
```

## ブラウザ起動

```rust
fn open_browser(url: &str) -> Result<()> {
    #[cfg(target_os = "macos")]
    std::process::Command::new("open").arg(url).spawn()?;

    #[cfg(target_os = "linux")]
    std::process::Command::new("xdg-open").arg(url).spawn()?;

    #[cfg(target_os = "windows")]
    std::process::Command::new("cmd").args(["/c", "start", url]).spawn()?;

    Ok(())
}
```

## 関連

- [commands/managed](../commands/managed.md) - TUI管理画面の使い方
- [overview](./overview.md) - アーキテクチャ概要

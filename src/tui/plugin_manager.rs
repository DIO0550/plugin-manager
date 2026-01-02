//! プラグイン管理 TUI
//!
//! インストール済みプラグインの一覧表示と管理を行う TUI。

use crate::plugin::PluginCache;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::terminal::{self, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::ExecutableCommand;
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Tabs};
use std::io::{self, stdout};

/// タブ種別
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Discover,
    Installed,
    Marketplaces,
    Errors,
}

impl Tab {
    fn all() -> &'static [Tab] {
        &[Tab::Discover, Tab::Installed, Tab::Marketplaces, Tab::Errors]
    }

    fn title(&self) -> &'static str {
        match self {
            Tab::Discover => "Discover",
            Tab::Installed => "Installed",
            Tab::Marketplaces => "Marketplaces",
            Tab::Errors => "Errors",
        }
    }

    fn index(&self) -> usize {
        match self {
            Tab::Discover => 0,
            Tab::Installed => 1,
            Tab::Marketplaces => 2,
            Tab::Errors => 3,
        }
    }

    fn from_index(index: usize) -> Self {
        match index % 4 {
            0 => Tab::Discover,
            1 => Tab::Installed,
            2 => Tab::Marketplaces,
            _ => Tab::Errors,
        }
    }
}

/// コンポーネント種別
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComponentType {
    Skills,
    Agents,
    Commands,
    Hooks,
}

impl ComponentType {
    fn all() -> &'static [ComponentType] {
        &[
            ComponentType::Skills,
            ComponentType::Agents,
            ComponentType::Commands,
            ComponentType::Hooks,
        ]
    }

    fn title(&self) -> &'static str {
        match self {
            ComponentType::Skills => "Skills",
            ComponentType::Agents => "Agents",
            ComponentType::Commands => "Commands",
            ComponentType::Hooks => "Hooks",
        }
    }
}

/// 画面状態
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Screen {
    /// プラグイン一覧
    PluginList,
    /// コンポーネント種別選択（プラグインインデックス）
    ComponentTypes(usize),
    /// コンポーネント一覧（プラグインインデックス、種別インデックス）
    ComponentList(usize, usize),
}

/// プラグイン一覧の表示アイテム
#[derive(Debug, Clone)]
pub struct PluginListItem {
    pub name: String,
    pub marketplace: Option<String>,
    pub version: String,
    pub skills: Vec<String>,
    pub agents: Vec<String>,
    pub commands: Vec<String>,
    pub hooks: Vec<String>,
}

/// アプリケーション状態
pub struct App {
    current_tab: Tab,
    screen: Screen,
    plugins: Vec<PluginListItem>,
    list_state: ListState,
    type_state: ListState,
    component_state: ListState,
    should_quit: bool,
}

impl App {
    /// 新しいアプリケーション状態を作成
    pub fn new() -> io::Result<Self> {
        let plugins = load_plugins()?;
        let mut list_state = ListState::default();
        if !plugins.is_empty() {
            list_state.select(Some(0));
        }

        let mut type_state = ListState::default();
        type_state.select(Some(0));

        Ok(Self {
            current_tab: Tab::Installed,
            screen: Screen::PluginList,
            plugins,
            list_state,
            type_state,
            component_state: ListState::default(),
            should_quit: false,
        })
    }

    /// 次のタブに移動
    fn next_tab(&mut self) {
        let next_index = (self.current_tab.index() + 1) % 4;
        self.current_tab = Tab::from_index(next_index);
    }

    /// 前のタブに移動
    fn prev_tab(&mut self) {
        let prev_index = (self.current_tab.index() + 3) % 4;
        self.current_tab = Tab::from_index(prev_index);
    }

    /// 現在の画面に応じたリスト長を取得
    fn current_list_len(&self) -> usize {
        match &self.screen {
            Screen::PluginList => self.plugins.len(),
            Screen::ComponentTypes(plugin_idx) => {
                // 空でないコンポーネント種別の数（0の場合は選択不可）
                if let Some(plugin) = self.plugins.get(*plugin_idx) {
                    let len = self.available_types(plugin).len();
                    if len == 0 { 1 } else { len } // 0だとselect_prev/nextが動かないので1を返す
                } else {
                    0
                }
            }
            Screen::ComponentList(plugin_idx, type_idx) => {
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

    /// プラグインの空でないコンポーネント種別を取得
    fn available_types(&self, plugin: &PluginListItem) -> Vec<ComponentType> {
        let mut types = Vec::new();
        if !plugin.skills.is_empty() {
            types.push(ComponentType::Skills);
        }
        if !plugin.agents.is_empty() {
            types.push(ComponentType::Agents);
        }
        if !plugin.commands.is_empty() {
            types.push(ComponentType::Commands);
        }
        if !plugin.hooks.is_empty() {
            types.push(ComponentType::Hooks);
        }
        types
    }

    /// コンポーネント種別に応じたリストを取得
    fn get_components<'a>(&self, plugin: &'a PluginListItem, comp_type: ComponentType) -> &'a Vec<String> {
        match comp_type {
            ComponentType::Skills => &plugin.skills,
            ComponentType::Agents => &plugin.agents,
            ComponentType::Commands => &plugin.commands,
            ComponentType::Hooks => &plugin.hooks,
        }
    }

    /// 現在の ListState を取得
    fn current_state_mut(&mut self) -> &mut ListState {
        match &self.screen {
            Screen::PluginList => &mut self.list_state,
            Screen::ComponentTypes(_) => &mut self.type_state,
            Screen::ComponentList(_, _) => &mut self.component_state,
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
            Screen::PluginList => {
                if let Some(plugin_idx) = self.list_state.selected() {
                    if self.plugins.get(plugin_idx).is_some() {
                        self.type_state.select(Some(0));
                        self.screen = Screen::ComponentTypes(plugin_idx);
                    }
                }
            }
            Screen::ComponentTypes(plugin_idx) => {
                if let Some(type_idx) = self.type_state.selected() {
                    if let Some(plugin) = self.plugins.get(*plugin_idx) {
                        let types = self.available_types(plugin);
                        if let Some(comp_type) = types.get(type_idx) {
                            if !self.get_components(plugin, *comp_type).is_empty() {
                                self.component_state.select(Some(0));
                                self.screen = Screen::ComponentList(*plugin_idx, type_idx);
                            }
                        }
                    }
                }
            }
            Screen::ComponentList(_, _) => {
                // 最下層なので何もしない
            }
        }
    }

    /// 前の階層へ戻る
    fn back(&mut self) {
        match &self.screen {
            Screen::PluginList => {
                self.should_quit = true;
            }
            Screen::ComponentTypes(_) => {
                self.screen = Screen::PluginList;
            }
            Screen::ComponentList(plugin_idx, _) => {
                self.screen = Screen::ComponentTypes(*plugin_idx);
            }
        }
    }

    /// キー入力を処理
    fn handle_key(&mut self, key: KeyCode) {
        match key {
            KeyCode::Char('q') => self.should_quit = true,
            KeyCode::Esc => self.back(),
            KeyCode::Tab => {
                if matches!(self.screen, Screen::PluginList) {
                    self.next_tab();
                }
            }
            KeyCode::BackTab => {
                if matches!(self.screen, Screen::PluginList) {
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

/// キャッシュからプラグイン一覧を読み込み
fn load_plugins() -> io::Result<Vec<PluginListItem>> {
    let cache = match PluginCache::new() {
        Ok(c) => c,
        Err(e) => {
            return Err(io::Error::new(io::ErrorKind::Other, e.to_string()));
        }
    };

    let plugin_list = match cache.list() {
        Ok(list) => list,
        Err(e) => {
            return Err(io::Error::new(io::ErrorKind::Other, e.to_string()));
        }
    };

    let mut plugins = Vec::new();
    let mut seen_marketplaces = std::collections::HashSet::new();

    for (marketplace, name) in plugin_list {
        // 隠しディレクトリやメタデータディレクトリは除外
        if name.starts_with('.') {
            // ただし、マーケットプレイスディレクトリ自体がプラグインの場合をチェック
            if let Some(mp) = &marketplace {
                if !seen_marketplaces.contains(mp) {
                    seen_marketplaces.insert(mp.clone());
                    // マーケットプレイスルートが直接プラグインかチェック
                    let mp_path = cache.plugin_path(Some(mp), "").parent().unwrap().to_path_buf();
                    if mp_path.join(".claude-plugin/plugin.json").exists() {
                        let manifest = cache.load_manifest(Some(mp), "").ok();
                        let version = manifest
                            .as_ref()
                            .map(|m| m.version.clone())
                            .unwrap_or_else(|| "unknown".to_string());
                        let scan = scan_components(&mp_path);
                        plugins.push(PluginListItem {
                            name: mp.clone(),
                            marketplace: Some(mp.clone()),
                            version,
                            skills: scan.skills,
                            agents: scan.agents,
                            commands: scan.commands,
                            hooks: scan.hooks,
                        });
                    }
                }
            }
            continue;
        }

        let plugin_path = cache.plugin_path(marketplace.as_deref(), &name);

        // plugin.json が存在するもののみをプラグインとして扱う
        // .claude-plugin/plugin.json または plugin.json をチェック
        let has_manifest = plugin_path.join(".claude-plugin/plugin.json").exists()
            || plugin_path.join("plugin.json").exists();
        if !has_manifest {
            continue;
        }

        let manifest = cache.load_manifest(marketplace.as_deref(), &name).ok();
        let version = manifest
            .as_ref()
            .map(|m| m.version.clone())
            .unwrap_or_else(|| "unknown".to_string());

        // コンポーネント情報を取得
        let scan = scan_components(&plugin_path);

        plugins.push(PluginListItem {
            name,
            marketplace,
            version,
            skills: scan.skills,
            agents: scan.agents,
            commands: scan.commands,
            hooks: scan.hooks,
        });
    }

    Ok(plugins)
}

/// スキャン結果
struct ScanResult {
    skills: Vec<String>,
    agents: Vec<String>,
    commands: Vec<String>,
    hooks: Vec<String>,
}

/// プラグインディレクトリからコンポーネントをスキャン
fn scan_components(plugin_path: &std::path::Path) -> ScanResult {
    let mut skills = Vec::new();
    let mut agents = Vec::new();
    let mut commands = Vec::new();
    let mut hooks = Vec::new();

    // Skills: skills/ ディレクトリ内の SKILL.md を持つディレクトリ
    let skills_dir = plugin_path.join("skills");
    if skills_dir.is_dir() {
        if let Ok(entries) = std::fs::read_dir(&skills_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() && path.join("SKILL.md").exists() {
                    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                        skills.push(name.to_string());
                    }
                }
            }
        }
    }

    // Agents: agents/ ディレクトリ内の .agent.md または .md ファイル
    let agents_dir = plugin_path.join("agents");
    if agents_dir.is_dir() {
        if let Ok(entries) = std::fs::read_dir(&agents_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                        if name.ends_with(".agent.md") {
                            agents.push(name.trim_end_matches(".agent.md").to_string());
                        } else if name.ends_with(".md") {
                            agents.push(name.trim_end_matches(".md").to_string());
                        }
                    }
                }
            }
        }
    }

    // Commands: commands/ ディレクトリ内の .prompt.md または .md ファイル
    let commands_dir = plugin_path.join("commands");
    if commands_dir.is_dir() {
        if let Ok(entries) = std::fs::read_dir(&commands_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                        if name.ends_with(".prompt.md") {
                            commands.push(name.trim_end_matches(".prompt.md").to_string());
                        } else if name.ends_with(".md") {
                            commands.push(name.trim_end_matches(".md").to_string());
                        }
                    }
                }
            }
        }
    }

    // Hooks: hooks/ ディレクトリ内のファイル
    let hooks_dir = plugin_path.join("hooks");
    if hooks_dir.is_dir() {
        if let Ok(entries) = std::fs::read_dir(&hooks_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                        // 拡張子を除いた名前を取得
                        let hook_name = name
                            .rsplit_once('.')
                            .map(|(n, _)| n.to_string())
                            .unwrap_or_else(|| name.to_string());
                        hooks.push(hook_name);
                    }
                }
            }
        }
    }

    ScanResult {
        skills,
        agents,
        commands,
        hooks,
    }
}

/// コンテンツに合わせたダイアログ領域を計算（左寄せ）
fn dialog_rect(width: u16, height: u16, area: Rect) -> Rect {
    Rect::new(area.x, area.y, width.min(area.width), height.min(area.height))
}

/// UI をレンダリング
fn render(f: &mut Frame, app: &mut App) {
    // 背景をクリア
    f.render_widget(Clear, f.area());

    match &app.screen {
        Screen::PluginList => render_list_screen(f, app),
        Screen::ComponentTypes(plugin_idx) => render_component_types_screen(f, app, *plugin_idx),
        Screen::ComponentList(plugin_idx, type_idx) => {
            render_component_list_screen(f, app, *plugin_idx, *type_idx)
        }
    }
}

/// プラグイン一覧画面
fn render_list_screen(f: &mut Frame, app: &mut App) {
    // タブに応じたコンテンツ高さを計算
    let content_height = match app.current_tab {
        Tab::Installed => (app.plugins.len() as u16).max(1) + 6,
        _ => 8, // placeholder tabs
    };
    let dialog_width = 55u16;
    let dialog_height = content_height.min(22);

    let dialog_area = dialog_rect(dialog_width, dialog_height, f.area());

    // 背景クリア
    f.render_widget(Clear, dialog_area);

    // レイアウト（タブ + コンテンツ + ヘルプ）
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // タブバー
            Constraint::Min(1),    // コンテンツ
            Constraint::Length(1), // ヘルプ
        ])
        .split(dialog_area);

    // タブバー
    let tab_titles: Vec<&str> = Tab::all().iter().map(|t| t.title()).collect();
    let tabs = Tabs::new(tab_titles)
        .select(app.current_tab.index())
        .style(Style::default().fg(Color::DarkGray))
        .highlight_style(
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )
        .divider(" | ");
    f.render_widget(tabs, chunks[0]);

    // タブに応じたコンテンツを表示
    match app.current_tab {
        Tab::Installed => render_installed_tab(f, app, chunks[1]),
        Tab::Discover => render_placeholder_tab(f, "Discover", "Browse available plugins", chunks[1]),
        Tab::Marketplaces => render_placeholder_tab(f, "Marketplaces", "Manage marketplace sources", chunks[1]),
        Tab::Errors => render_placeholder_tab(f, "Errors", "No errors", chunks[1]),
    }

    // ヘルプ
    let help_text = match app.current_tab {
        Tab::Installed => " Tab: switch · ↑/↓: move · Enter: details · q: quit",
        _ => " Tab: switch · q: quit",
    };
    let help = Paragraph::new(help_text).style(Style::default().fg(Color::DarkGray));
    f.render_widget(help, chunks[2]);
}

/// Installed タブのコンテンツ
fn render_installed_tab(f: &mut Frame, app: &mut App, area: Rect) {
    let items: Vec<ListItem> = app
        .plugins
        .iter()
        .map(|plugin| {
            let marketplace_str = plugin
                .marketplace
                .as_ref()
                .map(|m| format!(" @{}", m))
                .unwrap_or_default();
            let text = format!("  {}{}  v{}", plugin.name, marketplace_str, plugin.version);
            ListItem::new(text)
        })
        .collect();

    let title = format!(" Installed Plugins ({}) ", app.plugins.len());
    let list = List::new(items)
        .block(Block::default().title(title).borders(Borders::ALL))
        .highlight_style(
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(Color::Green),
        )
        .highlight_symbol("> ");

    f.render_stateful_widget(list, area, &mut app.list_state);
}

/// プレースホルダータブのコンテンツ
fn render_placeholder_tab(f: &mut Frame, title: &str, message: &str, area: Rect) {
    let content = Paragraph::new(format!("\n  {}", message))
        .block(
            Block::default()
                .title(format!(" {} ", title))
                .borders(Borders::ALL),
        )
        .style(Style::default().fg(Color::DarkGray));
    f.render_widget(content, area);
}

/// コンポーネント種別選択画面
fn render_component_types_screen(f: &mut Frame, app: &mut App, plugin_idx: usize) {
    let plugin = match app.plugins.get(plugin_idx) {
        Some(p) => p,
        None => return,
    };

    // プラグイン情報 + コンポーネント種別
    let mut lines = Vec::new();
    lines.push(format!("  Version: {}", plugin.version));
    if let Some(marketplace) = &plugin.marketplace {
        lines.push(format!("  Marketplace: {}", marketplace));
    }
    lines.push(String::new());

    let types = app.available_types(plugin);
    if types.is_empty() {
        lines.push("  Components: (none)".to_string());
    } else {
        lines.push("  Components:".to_string());
        for t in &types {
            let count = match t {
                ComponentType::Skills => plugin.skills.len(),
                ComponentType::Agents => plugin.agents.len(),
                ComponentType::Commands => plugin.commands.len(),
                ComponentType::Hooks => plugin.hooks.len(),
            };
            lines.push(format!("    {} ({})", t.title(), count));
        }
    }

    // ダイアログサイズ
    let content_height = (lines.len() as u16) + 4;
    let dialog_width = 55u16;
    let dialog_height = content_height.min(15);

    let dialog_area = dialog_rect(dialog_width, dialog_height, f.area());
    f.render_widget(Clear, dialog_area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(1)])
        .split(dialog_area);

    let title = format!(" {} ", plugin.name);
    let content = lines.join("\n");

    if types.is_empty() {
        // コンポーネントがない場合は静的表示
        let paragraph = Paragraph::new(content)
            .block(Block::default().title(title).borders(Borders::ALL));
        f.render_widget(paragraph, chunks[0]);
    } else {
        // コンポーネントがある場合はリスト選択可能
        let items: Vec<ListItem> = types
            .iter()
            .map(|t| {
                let count = match t {
                    ComponentType::Skills => plugin.skills.len(),
                    ComponentType::Agents => plugin.agents.len(),
                    ComponentType::Commands => plugin.commands.len(),
                    ComponentType::Hooks => plugin.hooks.len(),
                };
                let text = format!("  {} ({})", t.title(), count);
                ListItem::new(text)
            })
            .collect();

        let list = List::new(items)
            .block(Block::default().title(title).borders(Borders::ALL))
            .highlight_style(
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .fg(Color::Green),
            )
            .highlight_symbol("> ");

        f.render_stateful_widget(list, chunks[0], &mut app.type_state);
    }

    let help_text = if types.is_empty() {
        " Esc: back · q: quit"
    } else {
        " ↑/↓: move · Enter: open · Esc: back · q: quit"
    };
    let help = Paragraph::new(help_text).style(Style::default().fg(Color::DarkGray));
    f.render_widget(help, chunks[1]);
}

/// コンポーネント一覧画面
fn render_component_list_screen(f: &mut Frame, app: &mut App, plugin_idx: usize, type_idx: usize) {
    let plugin = match app.plugins.get(plugin_idx) {
        Some(p) => p,
        None => return,
    };

    let types = app.available_types(plugin);
    let comp_type = match types.get(type_idx) {
        Some(t) => *t,
        None => return,
    };

    let components = app.get_components(plugin, comp_type);
    let items: Vec<ListItem> = components
        .iter()
        .map(|name| ListItem::new(format!("  {}", name)))
        .collect();

    // ダイアログサイズ
    let content_height = (components.len() as u16).max(1) + 4;
    let dialog_width = 55u16;
    let dialog_height = content_height.min(20);

    let dialog_area = dialog_rect(dialog_width, dialog_height, f.area());
    f.render_widget(Clear, dialog_area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(1)])
        .split(dialog_area);

    let title = format!(" {} > {} ({}) ", plugin.name, comp_type.title(), components.len());
    let list = List::new(items)
        .block(Block::default().title(title).borders(Borders::ALL))
        .highlight_style(
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(Color::Green),
        )
        .highlight_symbol("> ");

    f.render_stateful_widget(list, chunks[0], &mut app.component_state);

    let help = Paragraph::new(" ↑/↓: move · Esc: back · q: quit")
        .style(Style::default().fg(Color::DarkGray));
    f.render_widget(help, chunks[1]);
}

/// TUI を実行
pub fn run() -> io::Result<()> {
    // ターミナル設定
    terminal::enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(stdout());
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new()?;

    // メインループ
    while !app.should_quit {
        terminal.draw(|f| render(f, &mut app))?;

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

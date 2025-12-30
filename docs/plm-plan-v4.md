# plm - Plugin Manager CLI 実装計画 v4

GitHubからAI開発環境向けのプラグインをダウンロードし、複数のAI環境を統一的に管理するRust製CLIツール

> **バージョン**: v4（統合版）
> **前バージョン**: [plm-plan-v3.md](./old/plm-plan-v3.md)

---

## 概要

### 背景

- Claude CodeはPluginとマーケットプレイスでskills, agents, commands, hooksを統合管理
- OpenAI CodexやVSCode CopilotもAgent Skills仕様に対応
- Claude Code以外にはマーケットプレイス機能がない
- GitHubからプラグインコンポーネントをダウンロードして管理する統一CLIが必要

### 目標

- GitHubベースのマーケットプレイスからプラグインをインストール
- プラグイン内のコンポーネントを自動的にCodex/Copilotへ展開
- TUI管理画面で直感的な操作を提供
- 詳細なプラグインメタデータの保持

---

## 対応環境とコンポーネント

### 共通規格

| 規格 | 説明 | 参照 |
|------|------|------|
| **AGENTS.md** | カスタム指示ファイル（Linux Foundation管轄のオープン標準） | https://agents.md |
| **SKILL.md** | スキル定義（Anthropicがオープン標準として公開、OpenAI/Microsoftが採用） | - |

### コンポーネント種別

| 種別 | 説明 | ファイル形式 |
|------|------|-------------|
| **Skills** | 専門的な知識・ワークフロー | `SKILL.md` (YAML frontmatter) |
| **Agents** | カスタムエージェント定義 | `*.agent.md` |
| **Prompts** | 再利用可能なプロンプト | `*.prompt.md` |
| **Instructions** | コーディング規約・カスタム指示 | `AGENTS.md` / `copilot-instructions.md` |

### 環境別の配置場所

#### OpenAI Codex

| 種別 | ファイル形式 | Personal | Project |
|------|-------------|----------|---------|
| Skills | `SKILL.md` | `~/.codex/skills/<name>/` | `.codex/skills/<name>/` |
| Agents | `*.agent.md` | `~/.codex/agents/` | `.codex/agents/` |
| Instructions | `AGENTS.md` | `~/.codex/AGENTS.md` | `AGENTS.md` |

> **注**: Codexは現時点で`.agent.md`を公式サポートしていないが、将来対応を見越して配置する

#### GitHub Copilot / VSCode

| 種別 | ファイル形式 | Personal | Project |
|------|-------------|----------|---------|
| Skills | `SKILL.md` | - | `.github/skills/<name>/` |
| Agents | `*.agent.md` | `~/.copilot/agents/` | `.github/agents/` |
| Prompts | `*.prompt.md` | - | `.github/prompts/` |
| Instructions | `AGENTS.md` | - | `AGENTS.md` |
| Instructions | `copilot-instructions.md` | - | `.github/copilot-instructions.md` |

---

## コマンド設計

### コマンド体系

```bash
# インストール（直接CLI）
plm install <source>                    # GitHubからインストール
plm install formatter@my-market         # マーケットプレイス経由
plm install owner/repo --target codex   # ターゲット指定
plm install owner/repo --scope personal # スコープ指定

# 管理画面（TUI）
plm managed                             # インタラクティブ管理画面

# マーケットプレイス管理
plm marketplace list
plm marketplace add owner/repo
plm marketplace add owner/repo --name my-market
plm marketplace remove <name>
plm marketplace update

# ターゲット管理
plm target list
plm target add codex
plm target add copilot
plm target remove copilot

# 簡易一覧・情報（非インタラクティブ）
plm list                                # インストール済み一覧
plm list --target codex                 # ターゲット別
plm list --type skill                   # 種別フィルタ
plm info <plugin-name>                  # 詳細情報

# コンポーネント作成・配布
plm init my-skill --type skill          # テンプレート作成
plm init my-agent --type agent
plm pack ./my-component                 # 配布用パッケージ作成

# 環境間同期
plm sync --from codex --to copilot      # コンポーネントをコピー
plm sync --from codex --to copilot --type skill

# Claude Code Plugin からのインポート
plm import owner/claude-plugin --component skills/pdf
plm import owner/claude-plugin --type skill
```

### 使い分け

| 操作 | CLI直接 | TUI管理画面 |
|------|---------|-------------|
| インストール | `plm install` | Discoverタブ |
| 更新 | - | ○ |
| 有効/無効 | - | ○ |
| 削除 | - | ○ |
| 状態確認 | `plm list` | ○ |
| GitHub参照 | - | ○ "View on GitHub" |
| 詳細表示 | `plm info` | ○ |

### インタラクティブ選択の動作

`--target`未指定時、有効なターゲットから選択UIを表示：

```
$ plm install formatter@my-market

? Select target(s) to deploy: (use space to select, enter to confirm)
> [x] codex   - Skills, Agents, Instructions
  [x] copilot - Skills, Agents, Prompts, Instructions

? Select scope:
> ( ) personal - ~/.codex/, ~/.copilot/
  (x) project  - .codex/, .github/

📥 Installing formatter to codex, copilot (project scope)...
```

---

## TUI管理画面 (`plm managed`)

### 画面構成

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
│    View on GitHub          ← GitRepo.github_web_url()           │
│    Back to plugin list                                          │
└─────────────────────────────────────────────────────────────────┘
```

### タブ構成

| タブ | 内容 |
|------|------|
| Discover | マーケットプレイスから利用可能なプラグイン検索・インストール |
| Installed | インストール済みプラグイン管理 |
| Marketplaces | 登録済みマーケットプレイス一覧・管理 |
| Errors | エラー・警告一覧 |

### アクション一覧

| アクション | 説明 | 実装 |
|------------|------|------|
| Disable/Enable plugin | プラグインの有効/無効切替 | キャッシュ更新 |
| Mark for update | 更新対象としてマーク | バッチ更新用 |
| Update now | 即座に更新 | GitHub API → キャッシュ更新 |
| Uninstall | プラグイン削除 | ファイル削除 + キャッシュ更新 |
| View on GitHub | リポジトリページを開く | `GitRepo.github_web_url()` |

---

## アーキテクチャ

### ディレクトリ構成

```
plm/
├── Cargo.toml
├── src/
│   ├── main.rs
│   ├── cli.rs                    # Clap CLI定義
│   ├── commands/
│   │   ├── install.rs            # インストール処理
│   │   ├── uninstall.rs          # 削除処理
│   │   ├── list.rs               # 一覧表示
│   │   ├── info.rs               # 詳細情報
│   │   ├── enable.rs             # 有効化
│   │   ├── disable.rs            # 無効化
│   │   ├── update.rs             # 更新処理
│   │   ├── target.rs             # ターゲット環境管理
│   │   ├── marketplace.rs        # マーケットプレイス管理
│   │   ├── init.rs               # テンプレート作成
│   │   ├── pack.rs               # パッケージ化
│   │   ├── sync.rs               # 環境間同期
│   │   └── import.rs             # Claude Plugin インポート
│   ├── tui/                      # TUI管理画面
│   │   ├── app.rs                # アプリケーション状態
│   │   ├── ui.rs                 # UI描画
│   │   ├── tabs/                 # 各タブ
│   │   │   ├── discover.rs
│   │   │   ├── installed.rs
│   │   │   ├── marketplaces.rs
│   │   │   └── errors.rs
│   │   └── widgets/              # 再利用可能ウィジェット
│   ├── targets/                  # AI環境アダプター
│   │   ├── trait.rs              # 共通インターフェース
│   │   ├── codex.rs              # OpenAI Codex
│   │   └── copilot.rs            # VSCode Copilot
│   ├── components/               # コンポーネント種別
│   │   ├── trait.rs              # 共通インターフェース
│   │   ├── skill.rs              # Skills
│   │   ├── agent.rs              # Agents
│   │   ├── prompt.rs             # Prompts
│   │   └── instruction.rs        # Instructions
│   ├── marketplace/              # マーケットプレイス
│   │   ├── registry.rs           # マーケットプレイス登録管理
│   │   └── fetcher.rs            # marketplace.json取得
│   ├── plugin/                   # プラグイン
│   │   ├── manifest.rs           # plugin.json パーサー
│   │   ├── cache.rs              # プラグインキャッシュ管理
│   │   └── deployer.rs           # 自動展開ロジック
│   ├── source/                   # プラグインソース
│   │   ├── trait.rs              # PluginSource トレイト
│   │   └── github.rs             # GitHub実装
│   ├── parser/                   # ファイルパーサー
│   │   ├── skill_md.rs           # SKILL.md パーサー
│   │   ├── agent_md.rs           # .agent.md パーサー
│   │   ├── prompt_md.rs          # .prompt.md パーサー
│   │   └── plugin_json.rs        # plugin.json パーサー
│   └── config.rs                 # 設定管理
├── tests/
└── README.md
```

### 依存クレート

```toml
[dependencies]
# CLI
clap = { version = "4", features = ["derive"] }

# 非同期
tokio = { version = "1", features = ["full"] }

# HTTP
reqwest = { version = "0.12", features = ["json", "stream"] }

# シリアライズ
serde = { version = "1", features = ["derive"] }
serde_json = "1"
toml = "0.8"
serde_yaml = "0.9"

# ファイル操作
zip = "2"
dirs = "5"
walkdir = "2"
glob = "0.3"

# TUI
ratatui = "0.29"
crossterm = "0.28"

# ターミナルUI
owo-colors = "4"
indicatif = "0.17"
comfy-table = "7"

# その他
chrono = { version = "0.4", features = ["serde"] }
thiserror = "2"
regex = "1"
```

---

## コア設計

### Component Trait（コンポーネント種別）

```rust
/// コンポーネント種別の共通インターフェース
pub trait Component {
    /// 種別名（"skill", "agent", "prompt", "instruction"）
    fn kind(&self) -> ComponentKind;

    /// ファイル名パターン
    fn file_pattern(&self) -> &str;

    /// メタデータをパース
    fn parse_metadata(&self, content: &str) -> Result<ComponentMetadata>;

    /// バリデーション
    fn validate(&self, path: &Path) -> Result<()>;
}

#[derive(Debug, Clone, Copy)]
pub enum ComponentKind {
    Skill,
    Agent,
    Prompt,
    Instruction,
}
```

### Target Trait（環境アダプター）

```rust
/// AI環境の共通インターフェース
pub trait Target {
    /// ターゲット名（"codex", "copilot"）
    fn name(&self) -> &str;

    /// サポートするコンポーネント種別
    fn supported_components(&self) -> Vec<ComponentKind>;

    /// コンポーネントのインストール先パス
    fn component_path(&self, kind: ComponentKind, scope: Scope) -> Option<PathBuf>;

    /// コンポーネントをインストール
    fn install(&self, component: &InstalledComponent, scope: Scope) -> Result<()>;

    /// コンポーネントを削除
    fn uninstall(&self, name: &str, kind: ComponentKind, scope: Scope) -> Result<()>;

    /// インストール済み一覧
    fn list(&self, kind: Option<ComponentKind>, scope: Scope) -> Result<Vec<InstalledComponent>>;
}

pub enum Scope {
    Personal,  // ~/.codex/skills/ など
    Project,   // .codex/skills/ など
}
```

### GitRepo 構造体

```rust
/// Gitリポジトリ参照（GitHub/GitLab/Bitbucket等で共通利用可能）
#[derive(Debug, Clone)]
pub struct GitRepo {
    pub owner: String,
    pub repo: String,
    pub git_ref: Option<String>,
    /// パース前の生の入力文字列
    pub raw: String,
}

impl GitRepo {
    /// 新しいGitRepoを作成
    pub fn new(owner: impl Into<String>, repo: impl Into<String>) -> Self;

    /// refを指定してGitRepoを作成
    pub fn with_ref(owner, repo, git_ref) -> Self;

    /// "owner/repo" または "owner/repo@ref" 形式をパース
    pub fn parse(input: &str) -> Result<Self>;

    // GitHub API URLs
    pub fn github_repo_url(&self) -> String;           // リポジトリ情報
    pub fn github_zipball_url(&self, ref) -> String;   // zipダウンロード
    pub fn github_commit_url(&self, ref) -> String;    // コミットSHA取得
    pub fn github_contents_url(&self, path, ref) -> String; // ファイル取得

    // Web URLs
    pub fn github_web_url(&self) -> String;            // ブラウザ用

    // ユーティリティ
    pub fn full_name(&self) -> String;                 // "owner/repo"
    pub fn ref_or_default(&self) -> &str;              // refまたは"HEAD"
}
```

---

## データ構造

### 設定ファイル（`~/.plm/config.toml`）

```toml
[general]
default_scope = "personal"  # personal | project

[targets]
enabled = ["codex", "copilot"]

[targets.codex]
skills_personal = "~/.codex/skills"
skills_project = ".codex/skills"
agents_personal = "~/.codex/agents"
agents_project = ".codex/agents"
instructions_personal = "~/.codex/AGENTS.md"
instructions_project = "AGENTS.md"

[targets.copilot]
skills_project = ".github/skills"
agents_personal = "~/.copilot/agents"
agents_project = ".github/agents"
prompts_project = ".github/prompts"
instructions_project = ".github/copilot-instructions.md"

[marketplaces]

[marketplaces.anthropic]
source = "github:anthropics/claude-code"
subdir = "plugins"

[marketplaces.company-tools]
source = "github:company/claude-plugins"
```

### プラグインキャッシュ（`~/.plm/plugins.json`）

```json
{
  "version": 1,
  "plugins": [
    {
      "name": "code-formatter",
      "source": "company/claude-plugins@v2.1.0",
      "version": "2.1.0",
      "status": "enabled",
      "marketplace": "company-tools",
      "installed_at": "2025-01-15T10:30:00Z",
      "installed_sha": "abc123def456",
      "author": {
        "name": "Dev Team",
        "email": "dev@company.com"
      },
      "components": {
        "skills": ["code-formatter"],
        "agents": ["formatter-agent"],
        "commands": ["format"],
        "hooks": []
      },
      "deployments": {
        "codex": {
          "scope": "personal",
          "enabled": true,
          "paths": ["~/.codex/skills/code-formatter"]
        },
        "copilot": {
          "scope": "project",
          "enabled": true,
          "paths": [".github/skills/code-formatter"]
        }
      }
    }
  ]
}
```

### マーケットプレイスキャッシュ（`~/.plm/cache/marketplaces/<name>.json`）

```json
{
  "name": "company-tools",
  "fetched_at": "2025-01-15T10:00:00Z",
  "source": "github:company/claude-plugins",
  "owner": {
    "name": "Company Dev Team",
    "email": "dev@company.com"
  },
  "plugins": [
    {
      "name": "code-formatter",
      "source": "./plugins/code-formatter",
      "description": "Automatic code formatting",
      "version": "2.1.0"
    }
  ]
}
```

---

## キャッシュアーキテクチャ

### 全体構成

```
┌─────────────────────────────────────────────────────────────────┐
│                        plm managed (TUI)                        │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                        PluginCache                              │
│                    (~/.plm/plugins.json)                        │
│  ┌────────────────────────────────────────────────────────────┐ │
│  │ CachedPlugin                                               │ │
│  │  - name: String                                            │ │
│  │  - source: String (GitRepo.raw)  ──┐                       │ │
│  │  - version: String                 │                       │ │
│  │  - status: Enabled/Disabled        │                       │ │
│  │  - installed_sha: String           │                       │ │
│  │  - components: [...]               │                       │ │
│  │  - deployments: {...}              │                       │ │
│  └────────────────────────────────────│───────────────────────┘ │
└───────────────────────────────────────│─────────────────────────┘
                                        │
                                        ▼ GitRepo::parse()
                              ┌─────────────────────┐
                              │      GitRepo        │
                              │  - owner            │
                              │  - repo             │
                              │  - git_ref          │
                              │  - raw              │
                              │                     │
                              │  github_web_url()   │──→ ブラウザで開く
                              │  github_*_url()     │──→ API呼び出し
                              └─────────────────────┘
                                        │
                                        ▼
                              ┌─────────────────────┐
                              │    GitHub API       │
                              │  - 更新チェック     │
                              │  - ダウンロード     │
                              └─────────────────────┘
```

### キャッシュの役割

| 役割 | 説明 |
|------|------|
| オフライン表示 | TUI起動時にネットワーク不要で一覧表示 |
| 状態管理 | Enabled/Disabled、バージョン情報 |
| 更新検知 | installed_sha と最新を比較 |
| 永続化 | `source` (raw) からいつでも `GitRepo` を復元可能 |

---

## Claude Code Plugin/Marketplace 構造

### プラグイン構造

```
plugin-name/
├── .claude-plugin/
│   └── plugin.json          # マニフェスト（必須）
├── commands/                 # スラッシュコマンド
│   └── command-name.md
├── agents/                   # カスタムエージェント
│   └── agent-name.md
├── skills/                   # Skills
│   └── skill-name/
│       └── SKILL.md
├── hooks/                    # イベントハンドラ
│   └── hooks.json
├── .mcp.json                # MCPサーバー設定
└── .lsp.json                # LSPサーバー設定
```

### plugin.json スキーマ

```json
{
  "name": "plugin-name",
  "version": "1.2.0",
  "description": "Brief plugin description",
  "author": {
    "name": "Author Name",
    "email": "author@example.com",
    "url": "https://github.com/author"
  },
  "homepage": "https://docs.example.com/plugin",
  "repository": "https://github.com/author/plugin",
  "license": "MIT",
  "keywords": ["keyword1", "keyword2"],
  "commands": ["./commands/"],
  "agents": "./agents/",
  "skills": "./skills/",
  "hooks": "./hooks/hooks.json",
  "mcpServers": "./.mcp.json",
  "lspServers": "./.lsp.json"
}
```

### マーケットプレイス構造

```
marketplace-repo/
├── .claude-plugin/
│   └── marketplace.json      # マーケットプレイス定義
└── plugins/
    ├── plugin-a/
    │   ├── .claude-plugin/
    │   │   └── plugin.json
    │   └── ...
    └── plugin-b/
        ├── .claude-plugin/
        │   └── plugin.json
        └── ...
```

### marketplace.json スキーマ

```json
{
  "name": "marketplace-name",
  "owner": {
    "name": "Organization Name",
    "email": "contact@example.com"
  },
  "plugins": [
    {
      "name": "plugin-a",
      "source": "./plugins/plugin-a",
      "description": "Plugin A description",
      "version": "1.0.0",
      "author": { "name": "Author" }
    },
    {
      "name": "plugin-b",
      "source": {
        "source": "github",
        "repo": "other-org/plugin-b"
      }
    }
  ]
}
```

---

## 自動展開マッピング

```
プラグイン内のディレクトリ:
├── skills/
│   └── skill-name/
│       └── SKILL.md
│           ↓ 展開先
│           Codex:   ~/.codex/skills/skill-name/ または .codex/skills/skill-name/
│           Copilot: .github/skills/skill-name/
│
├── agents/
│   └── agent-name.md
│           ↓ 展開先
│           Codex:   ~/.codex/agents/agent-name.agent.md または .codex/agents/agent-name.agent.md
│                    ※将来対応を見越して配置（現時点では未サポート）
│           Copilot: ~/.copilot/agents/agent-name.agent.md または .github/agents/agent-name.agent.md
│
├── prompts/
│   └── prompt-name.prompt.md
│           ↓ 展開先
│           Copilot: .github/prompts/prompt-name.prompt.md
│           Codex:   展開対象外（未サポート）
│
└── commands/, hooks/, .mcp.json, .lsp.json
            ↓
            展開対象外（Claude Code専用）
```

---

## ターゲット環境の設定読み込み仕様

### OpenAI Codex CLI

公式ドキュメント: [Custom instructions with AGENTS.md](https://developers.openai.com/codex/guides/agents-md/)

#### 読み込みパスと優先順位

| スコープ | パス | 自動読み込み | 備考 |
|---------|------|--------------|------|
| Global (override) | `~/.codex/AGENTS.override.md` | ✅ | 最優先 |
| Global | `~/.codex/AGENTS.md` | ✅ | Personal対応 |
| Project | `./AGENTS.override.md` | ✅ | ディレクトリ毎 |
| Project | `./AGENTS.md` | ✅ | ディレクトリ毎 |
| Skills (Global) | `~/.codex/skills/` | ✅ | Personal |
| Skills (Project) | `./.codex/skills/` | ✅ | Project |

#### 読み込み順序

1. **Global scope**: `~/.codex/` (または `$CODEX_HOME`) をチェック
   - `AGENTS.override.md` があればそれを使用、なければ `AGENTS.md`
2. **Project scope**: リポジトリルートから現在ディレクトリまで走査
   - 各ディレクトリで `AGENTS.override.md` → `AGENTS.md` → fallback の順
3. **マージ**: ルートから現在ディレクトリに向かって連結（上限: `project_doc_max_bytes` = 32KiB）

### VSCode GitHub Copilot

公式ドキュメント: [Use custom instructions in VS Code](https://code.visualstudio.com/docs/copilot/customization/custom-instructions)

#### 読み込みパスと優先順位

| スコープ | パス | 自動読み込み | 備考 |
|---------|------|--------------|------|
| Project | `.github/copilot-instructions.md` | ✅ | メインの指示ファイル |
| Project | `.github/instructions/*.instructions.md` | ❌ | 手動指定が必要 |
| User | VSCode設定の `file` プロパティ | ✅ | 設定で外部ファイル参照 |
| Prompts | `.github/prompts/*.prompt.md` | ❌ | 手動呼び出し |

#### 重要な制約

- **Copilotはグローバルファイル（`~/.copilot/`等）を直接読み込まない**
- Personal スコープは VSCode 設定経由で外部ファイルを参照する形式
- Issue: [Global files outside workspace の要望](https://github.com/microsoft/vscode-copilot-release/issues/3129)

#### VSCode設定での外部ファイル参照

```json
// settings.json (User または Workspace)
{
  "github.copilot.chat.codeGeneration.instructions": [
    {
      "file": "/path/to/personal-instructions.md"
    }
  ],
  "github.copilot.chat.codeGeneration.useInstructionFiles": true
}
```

### PLMでの対応方針

| ターゲット | Personal インストール | 追加アクション |
|-----------|----------------------|----------------|
| Codex | `~/.codex/` に配置 | 不要（自動読み込み） |
| Copilot | ファイル配置 + VSCode設定追記 | `settings.json` への参照追加が必要 |

---

## ファイル形式リファレンス

### SKILL.md

```markdown
---
name: skill-name
description: スキルの説明（500文字以内）
metadata:
  short-description: 短い説明
---

# Skill Name

スキルの詳細な指示...
```

### *.agent.md

```markdown
---
name: agent-name
description: エージェントの説明
tools: ['search', 'fetch', 'edit']
---

# Agent Instructions

エージェントの指示...
```

### *.prompt.md

```markdown
---
name: prompt-name
description: プロンプトの説明
---

# Prompt

プロンプトの内容...
```

### AGENTS.md

```markdown
# Project Guidelines

プロジェクト固有のコーディング規約やワークフロー...
```

---

## 処理フロー

### インストールフロー

```
1. plm install owner/repo@v1.0.0
2. GitRepo::parse("owner/repo@v1.0.0")
3. repo.github_zipball_url("v1.0.0") でダウンロード
4. ~/.plm/cache/plugins/<name>/ に展開
5. plugin.json パース
6. ターゲットへ自動展開
7. CachedPlugin作成（source = repo.raw）
8. plugins.json に保存
```

### TUI表示フロー

```
1. plm managed
2. PluginCache::load() で plugins.json 読み込み
3. 一覧表示（ネットワーク不要）
4. 選択時: CachedPlugin.git_repo() で GitRepo 復元
5. "View on GitHub": repo.github_web_url() でブラウザ起動
```

### 更新フロー

```
1. TUIで "Update now" 選択
2. CachedPlugin.git_repo() で GitRepo 復元
3. repo.github_commit_url("HEAD") で最新SHA取得
4. installed_sha と比較
5. 差分あれば repo.github_zipball_url() でダウンロード
6. 再展開
7. CachedPlugin更新、plugins.json 保存
```

---

## 使用例

### 初期セットアップ

```bash
$ plm target add codex
✅ Added target: codex
   Supports: skills, agents, instructions

$ plm target add copilot
✅ Added target: copilot
   Supports: skills, agents, prompts, instructions

$ plm target list
📍 Active targets:
   • codex   (skills, agents, instructions)
   • copilot (skills, agents, prompts, instructions)
```

### Skillのインストール

```bash
$ plm install doi/html-educational-material
📥 Fetching doi/html-educational-material...
🔍 Detected: skill
📦 Installing to codex (personal)... ✅
📦 Installing to copilot (project)... ✅
✅ Installed skill: html-educational-material v1.0.0
```

### マーケットプレイスの登録

```bash
$ plm marketplace add company/claude-plugins --name company-tools
📥 Fetching marketplace.json...
✅ Added marketplace: company-tools
   Available plugins: 5
```

### プラグインのインストール

```bash
$ plm install code-formatter@company-tools
📥 Fetching code-formatter from company-tools...
📦 Installing to codex... ✅
📦 Installing to copilot... ✅
✅ Installed plugin: code-formatter v2.1.0
   Components:
   • skills: code-formatter
   • agents: formatter-agent
```

### 一覧表示

```bash
$ plm list
┌────────────────────────────┬─────────┬────────┬───────────────┬─────────────┐
│ Name                       │ Version │ Type   │ Targets       │ Marketplace │
├────────────────────────────┼─────────┼────────┼───────────────┼─────────────┤
│ html-educational-material  │ 1.0.0   │ skill  │ codex,copilot │ -           │
│ code-formatter             │ 2.1.0   │ plugin │ codex,copilot │ company     │
│ code-reviewer              │ 0.1.0   │ agent  │ copilot       │ -           │
└────────────────────────────┴─────────┴────────┴───────────────┴─────────────┘
```

### テンプレート作成

```bash
$ plm init my-skill --type skill
📁 Created my-skill/
   └── SKILL.md

$ plm init my-agent --type agent
📁 Created my-agent.agent.md
```

### 環境間の同期

```bash
$ plm sync --from codex --to copilot --type skill
🔄 Syncing skills from codex to copilot...
   ✓ html-educational-material (already synced)
   + frontend-design (installing...)
✅ Synced 1 skill to copilot
```

---

## 実装フェーズ

### Phase 1: 基盤構築 ✅

- [x] Cargoプロジェクト初期化
- [x] CLI引数パーサー（clap）
- [x] 基本的なエラーハンドリング
- [x] GitRepo構造体（raw保持、URL生成）

### Phase 2: Target/Component 実装

- [ ] Target trait 定義
- [ ] Component trait 定義
- [ ] Codexターゲット実装
- [ ] Copilotターゲット実装
- [ ] `plm target` コマンド

### Phase 3: パーサー実装

- [ ] SKILL.md パーサー（YAML frontmatter）
- [ ] .agent.md パーサー
- [ ] .prompt.md パーサー
- [ ] plugin.json パーサー

### Phase 4: GitHubダウンロード・インストール

- [ ] GitHubリポジトリダウンロード
- [ ] ZIP展開
- [ ] コンポーネント種別の自動検出
- [ ] `plm install` コマンド
- [ ] 自動展開ロジック

### Phase 5: キャッシュ基盤

- [ ] `CachedPlugin` 構造体定義
- [ ] `PluginCache` 読み書き実装
- [ ] `git_repo()` メソッド実装

### Phase 6: マーケットプレイス機能

- [ ] `plm marketplace add/remove/list`
- [ ] marketplace.json パーサー
- [ ] マーケットプレイスキャッシュ管理

### Phase 7: 管理機能

- [ ] `plm list` コマンド
- [ ] `plm info` コマンド
- [ ] `plm uninstall` コマンド（展開先も削除）
- [ ] `plm enable/disable` コマンド

### Phase 8: 更新・同期機能

- [ ] `plm update` コマンド
- [ ] `plm sync` コマンド
- [ ] バージョン/SHA比較ロジック

### Phase 9: 作成・配布機能

- [ ] `plm init` コマンド（テンプレート生成）
- [ ] `plm pack` コマンド（ZIP作成）

### Phase 10: インポート機能

- [ ] Claude Code Plugin構造の解析
- [ ] コンポーネント抽出
- [ ] `plm import` コマンド

### Phase 11: TUI基盤

- [ ] ratatui 依存追加
- [ ] 基本レイアウト（タブ、リスト、詳細）
- [ ] キーバインド設計

### Phase 12: TUIタブ実装

- [ ] Installedタブ（プラグイン一覧、詳細、View on GitHub）
- [ ] Discoverタブ（マーケットプレイス検索・インストール）
- [ ] Marketplacesタブ
- [ ] Errorsタブ

### Phase 13: TUIアクション実装

- [ ] Enable/Disable 実装
- [ ] Uninstall 実装
- [ ] Update now 実装
- [ ] Mark for update（バッチ更新）

### Phase 14: UX改善

- [ ] プログレスバー（indicatif）
- [ ] カラー出力（owo-colors）
- [ ] テーブル表示（comfy-table）
- [ ] エラーメッセージ改善
- [ ] ヘルプ・ドキュメント

---

## 技術選定

### TUIライブラリ

| ライブラリ | 選定理由 |
|------------|----------|
| **ratatui** | Rust製TUIのデファクト、活発なメンテナンス |
| crossterm | クロスプラットフォームターミナル操作 |

### ブラウザ起動

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

---

## 将来の拡張

### 追加ターゲット候補

- Cursor（.cursor/）
- Windsurf
- Aider
- Gemini CLI
- その他SKILL.md対応ツール

### GitLab/Bitbucket対応

```rust
impl GitRepo {
    // 将来追加
    pub fn gitlab_repo_url(&self) -> String;
    pub fn gitlab_web_url(&self) -> String;

    pub fn bitbucket_repo_url(&self) -> String;
    pub fn bitbucket_web_url(&self) -> String;
}
```

### 追加機能候補

- プラグイン検索（`plm search`）
- プラグイン依存関係解決
- バージョン固定（lockfile）
- ローカルプラグイン開発支援（`plm dev`）
- プラグインバリデーション（`plm validate`）
- CI/CD統合（GitHub Actions）
- プラグインのホスティング機能

---

## 参考リンク

### Agent Skills

- [Agent Skills Specification](https://github.com/anthropics/skills)
- [Skills Marketplace](https://skillsmp.com)

### OpenAI Codex

- [Codex Skills](https://developers.openai.com/codex/skills/)
- [AGENTS.md Guide](https://developers.openai.com/codex/guides/agents-md/)

### VSCode Copilot

- [Custom Instructions](https://code.visualstudio.com/docs/copilot/customization/custom-instructions)
- [Custom Agents](https://docs.github.com/en/copilot/how-tos/use-copilot-agents/coding-agent/create-custom-agents)
- [Prompt Files](https://code.visualstudio.com/docs/copilot/customization/overview)

### Claude Code

- [Plugins Documentation](https://code.claude.com/docs/en/plugins)
- [Skills Documentation](https://code.claude.com/docs/en/skills)
- [anthropics/claude-code plugins](https://github.com/anthropics/claude-code/tree/main/plugins)

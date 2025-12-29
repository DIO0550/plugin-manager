# plm - Plugin Manager CLI 実装計画 v2

GitHubからAI開発環境向けのプラグインをダウンロードし、複数のAI環境を統一的に管理するRust製CLIツール

> **バージョン**: v2（マーケットプレイス方式）
> **前バージョン**: [plm-plan.md](./plm-plan.md)（コンポーネント単位方式）

## 概要

### v1からの変更点

| 項目 | v1（コンポーネント単位） | v2（マーケットプレイス方式） |
|------|--------------------------|------------------------------|
| インストール単位 | Skills, Agents等を個別に | プラグイン単位 |
| マーケットプレイス | なし | あり（marketplace.json対応） |
| 展開方式 | 手動指定 | 自動展開 |
| メタデータ | 基本情報のみ | 詳細情報も保持 |

### 背景

- Claude CodeはPluginとマーケットプレイスでskills, agents, commands, hooksを統合管理
- OpenAI CodexやVSCode CopilotもAgent Skills仕様に対応
- Claude Codeのマーケットプレイス方式を他環境にも適用することで統一的な管理が可能

### 目標

- GitHubベースのマーケットプレイスからプラグインをインストール
- プラグイン内のコンポーネントを自動的にCodex/Copilotへ展開
- 詳細なプラグインメタデータの保持

---

## 環境別ファイル形式仕様

### 共通規格

| 規格 | 説明 | 参照 |
|------|------|------|
| **AGENTS.md** | カスタム指示ファイル（Linux Foundation管轄のオープン標準） | https://agents.md |
| **SKILL.md** | スキル定義（Anthropicがオープン標準として公開、OpenAI/Microsoftが採用） | - |

### OpenAI Codex

| 種別 | ファイル形式 | Personal | Project |
|------|-------------|----------|---------|
| Skills | `SKILL.md` | `~/.codex/skills/<name>/` | `.codex/skills/<name>/` |
| Agents | `*.agent.md` | `~/.codex/agents/` | `.codex/agents/` |
| Instructions | `AGENTS.md` | `~/.codex/AGENTS.md` | `AGENTS.md` |

> **注**: Codexは現時点で`.agent.md`を公式サポートしていないが、将来対応を見越して配置する

### GitHub Copilot / VSCode

| 種別 | ファイル形式 | Personal | Project |
|------|-------------|----------|---------|
| Skills | `SKILL.md` | - | `.github/skills/<name>/` |
| Agents | `*.agent.md` | `~/.copilot/agents/` | `.github/agents/` |
| Prompts | `*.prompt.md` | - | `.github/prompts/` |
| Instructions | `AGENTS.md` | - | `AGENTS.md` |
| Instructions | `copilot-instructions.md` | - | `.github/copilot-instructions.md` |

### ファイル形式詳細

#### SKILL.md（共通）

```markdown
---
name: skill-name
description: スキルの説明（500文字以内）
---

# Skill Name

スキルの詳細な指示...
```

#### *.agent.md（Copilot、将来的にCodex）

```markdown
---
name: agent-name
description: エージェントの説明
tools: ["read", "edit", "search"]  # オプション
---

# Agent Instructions

エージェントの指示...
```

#### AGENTS.md（共通）

```markdown
# Project Guidelines

プロジェクト固有のコーディング規約やワークフロー...
```

---

## コマンド設計

### マーケットプレイス管理

```bash
# マーケットプレイス一覧
plm marketplace list

# マーケットプレイス追加
plm marketplace add owner/repo                    # GitHub
plm marketplace add owner/repo --name my-market   # 名前指定
plm marketplace add https://gitlab.com/org/repo   # フルURL

# マーケットプレイス削除
plm marketplace remove my-market

# マーケットプレイス更新（キャッシュリフレッシュ）
plm marketplace update
plm marketplace update my-market
```

### プラグインインストール

```bash
# ターゲット指定
plm install formatter@my-market --target codex
plm install formatter@my-market --target copilot

# 複数ターゲット指定
plm install formatter@my-market --target codex --target copilot

# ターゲット未指定 → インタラクティブ選択UI表示
plm install formatter@my-market
# ? Select target(s) to deploy:
# > [x] codex
#   [x] copilot
# (スペースで選択、Enterで確定)

# 全ターゲットに展開（選択UIをスキップ）
plm install formatter@my-market --all-targets

# 最初に見つかったマーケットプレイスから
plm install formatter --target codex

# スコープ指定
plm install formatter@my-market --target codex --scope personal
plm install formatter@my-market --target copilot --scope project
```

### インタラクティブ選択の動作

`--target`未指定時、有効なターゲット（`plm target list`で表示されるもの）から選択UIを表示：

優先順位：
1. `--target` 指定あり → そのターゲットを使用
2. `--all-targets` 指定 → 全有効ターゲットに展開
3. 上記なし → インタラクティブ選択UI表示

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

### プラグイン管理

```bash
# インストール済みプラグイン一覧
plm list
plm list --target codex

# プラグイン詳細
plm info formatter

# プラグイン更新
plm update                  # 全プラグイン
plm update formatter        # 特定プラグイン

# プラグイン削除
plm uninstall formatter
plm uninstall formatter --target codex  # 特定ターゲットのみ

# 有効/無効
plm enable formatter
plm disable formatter
plm enable formatter --target codex
```

### ターゲット環境管理

```bash
# ターゲット一覧
plm target list

# ターゲット追加/削除
plm target add codex
plm target add copilot
plm target remove copilot
```

---

## アーキテクチャ

### ディレクトリ構成（更新）

```
plm/
├── src/
│   ├── main.rs
│   ├── cli.rs                    # Clap CLI定義
│   ├── commands/
│   │   ├── mod.rs
│   │   ├── marketplace.rs        # 【新規】マーケットプレイス管理
│   │   ├── install.rs            # 【更新】プラグイン単位インストール
│   │   ├── uninstall.rs
│   │   ├── list.rs
│   │   ├── enable.rs
│   │   ├── disable.rs
│   │   ├── update.rs
│   │   ├── info.rs
│   │   └── target.rs
│   ├── marketplace/              # 【新規】
│   │   ├── mod.rs
│   │   ├── registry.rs           # マーケットプレイス登録管理
│   │   └── fetcher.rs            # marketplace.json取得
│   ├── plugin/                   # 【新規】
│   │   ├── mod.rs
│   │   ├── manifest.rs           # plugin.json パーサー
│   │   ├── cache.rs              # プラグインキャッシュ管理
│   │   └── deployer.rs           # 自動展開ロジック
│   ├── targets/
│   │   ├── mod.rs
│   │   ├── trait.rs
│   │   ├── codex.rs
│   │   └── copilot.rs
│   ├── github/
│   │   ├── mod.rs
│   │   └── fetcher.rs
│   └── config.rs
└── ...
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
agents_personal = "~/.codex/agents"       # 将来対応を見越して配置
agents_project = ".codex/agents"          # 将来対応を見越して配置
instructions_personal = "~/.codex/AGENTS.md"
instructions_project = "AGENTS.md"

[targets.copilot]
skills_project = ".github/skills"
agents_personal = "~/.copilot/agents"
agents_project = ".github/agents"
prompts_project = ".github/prompts"
instructions_project = ".github/copilot-instructions.md"

# 【新規】マーケットプレイス設定
[marketplaces]

[marketplaces.anthropic]
source = "github:anthropics/claude-code"
subdir = "plugins"  # オプション

[marketplaces.company-tools]
source = "github:company/claude-plugins"
```

### プラグイン管理ファイル（`~/.plm/plugins.json`）【新規】

```json
{
  "version": 1,
  "plugins": [
    {
      "name": "code-formatter",
      "version": "2.1.0",
      "description": "Automatic code formatting",
      "marketplace": "company-tools",
      "source": "github:company/claude-plugins/plugins/code-formatter",
      "author": {
        "name": "Dev Team",
        "email": "dev@company.com",
        "url": "https://github.com/company"
      },
      "homepage": "https://docs.company.com/formatter",
      "repository": "https://github.com/company/claude-plugins",
      "license": "MIT",
      "keywords": ["formatter", "code-style"],
      "installed_at": "2025-01-15T10:30:00Z",
      "updated_at": "2025-01-15T10:30:00Z",
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
          "paths": {
            "skills": ["~/.codex/skills/code-formatter"],
            "agents": ["~/.codex/agents/formatter-agent.agent.md"]
          }
        },
        "copilot": {
          "scope": "project",
          "enabled": true,
          "paths": {
            "skills": [".github/skills/code-formatter"],
            "agents": [".github/agents/formatter-agent.agent.md"]
          }
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
    },
    {
      "name": "test-runner",
      "source": "./plugins/test-runner",
      "description": "Run tests with AI assistance",
      "version": "1.0.0"
    }
  ]
}
```

---

## 処理フロー

### マーケットプレイス追加

```
1. plm marketplace add company/claude-plugins --name company-tools
2. GitHubリポジトリから .claude-plugin/marketplace.json を取得
3. パースしてキャッシュに保存 (~/.plm/cache/marketplaces/company-tools.json)
4. config.toml に登録
```

### プラグインインストール

```
1. plm install code-formatter@company-tools --scope personal
2. マーケットプレイスキャッシュからプラグイン情報取得
3. GitHubからプラグインディレクトリをダウンロード
4. ~/.plm/cache/plugins/code-formatter/ に展開
5. .claude-plugin/plugin.json をパース
6. 自動展開:
   - skills/ → ~/.codex/skills/ (Codex), .github/skills/ (Copilot)
   - agents/ → ~/.codex/agents/ (Codex※), ~/.copilot/agents/ または .github/agents/ (Copilot)
   - prompts/ → .github/prompts/ (Copilotのみ)
   ※Codexは将来対応を見越して配置
7. plugins.json に記録
```

### 自動展開マッピング

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

## Claude Code Plugin/Marketplace 構造（参照）

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

## 実装フェーズ

### Phase 1: CLI拡張・データ構造

- [ ] `cli.rs` に marketplace サブコマンド追加
- [ ] `commands/marketplace.rs` 作成
- [ ] config.toml のマーケットプレイス設定対応
- [ ] plugins.json スキーマ定義・読み書き

### Phase 2: マーケットプレイス機能

- [ ] `plm marketplace add/remove/list`
- [ ] marketplace.json パーサー
- [ ] GitHubリポジトリからmarketplace.json取得
- [ ] マーケットプレイスキャッシュ管理

### Phase 3: プラグインインストール

- [ ] `plm install <plugin>@<marketplace>`
- [ ] plugin.json 詳細パーサー
- [ ] プラグインキャッシュ（~/.plm/cache/plugins/）
- [ ] コンポーネント検出ロジック

### Phase 4: 自動展開

- [ ] Target trait 拡張（deploy メソッド追加）
- [ ] Codexへの自動展開
- [ ] Copilotへの自動展開
- [ ] deployments情報の記録

### Phase 5: 管理機能

- [ ] `plm list` - プラグイン単位での一覧表示
- [ ] `plm info` - プラグイン詳細（展開先含む）
- [ ] `plm uninstall` - プラグイン削除（展開先も削除）
- [ ] `plm enable/disable` - プラグイン有効/無効

### Phase 6: 更新機能

- [ ] `plm update` - プラグイン更新
- [ ] `plm marketplace update` - マーケットプレイスキャッシュ更新
- [ ] バージョン比較ロジック

### Phase 7: UX改善

- [ ] プログレスバー（indicatif）
- [ ] カラー出力（owo-colors）
- [ ] テーブル表示（comfy-table）
- [ ] エラーメッセージ改善

---

## 将来の拡張

### 追加ターゲット候補

- Cursor（.cursor/）
- Windsurf
- Aider
- Gemini CLI

### 追加機能候補

- プラグイン検索（`plm search`）
- プラグイン依存関係解決
- ローカルプラグイン開発支援（`plm dev`）
- プラグインバリデーション（`plm validate`）

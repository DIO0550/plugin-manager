# plm - Plugin Manager CLI 実装計画

GitHubからAI開発環境向けのプラグイン（Skills, Agents, Prompts, Instructions）をダウンロードし、複数のAI環境を統一的に管理するRust製CLIツール

> **リポジトリ名**: `plm`

## 概要

### 背景

- Claude CodeはPluginという単位でskills, agents, commands, hooksをまとめて管理
- OpenAI CodexやVSCode CopilotもAgent Skills仕様に対応し始めている
- しかし、Claude Code以外にはマーケットプレイス機能がない
- GitHubからプラグインコンポーネントをダウンロードして管理する統一CLIが必要

### 目標

- GitHubリポジトリからプラグインコンポーネントを簡単にインストール
- 複数のAI環境（Codex、VSCode Copilot）を統一的に管理
- Skills, Agents, Prompts, Instructions の全コンポーネントに対応
- Claude Code Pluginからのコンポーネント抽出にも対応

---

## 対応環境とコンポーネント

### コンポーネント種別

| 種別 | 説明 | ファイル形式 |
|------|------|-------------|
| **Skills** | 専門的な知識・ワークフロー | `SKILL.md` (YAML frontmatter) |
| **Agents** | カスタムエージェント定義 | `.agent.md` / `AGENTS.md` |
| **Prompts** | 再利用可能なプロンプト | `.prompt.md` |
| **Instructions** | コーディング規約・カスタム指示 | `copilot-instructions.md` / `.instructions.md` |

### 環境別の配置場所

#### OpenAI Codex

| 種別 | Personal | Project |
|------|----------|---------|
| Skills | `~/.codex/skills/` | `.codex/skills/` |
| Instructions | `~/.codex/AGENTS.md` | `AGENTS.md` |

#### VSCode Copilot

| 種別 | Personal | Project |
|------|----------|---------|
| Skills | N/A | `.github/skills/` |
| Agents | `~/.copilot/agents/` | `.github/agents/` |
| Prompts | N/A | `.github/prompts/` |
| Instructions | N/A | `.github/copilot-instructions.md` |
| Instructions (複数) | N/A | `.github/instructions/*.instructions.md` |

#### Claude Code Plugin 構造（参考・インポート元）

```
plugin-name/
├── .claude-plugin/
│   └── plugin.json       # プラグインメタデータ
├── skills/               # Skills
│   └── skill-name/
│       └── SKILL.md
├── agents/               # Agents
│   └── agent-name.md
├── commands/             # Slash Commands
│   └── command-name.md
└── hooks/                # Hooks
    └── hooks.json
```

---

## コマンド設計

### ターゲット環境管理

```bash
# 現在のターゲット一覧
plm target

# 環境を追加/削除
plm target add codex
plm target add copilot
plm target remove copilot
```

### プラグイン/コンポーネントのインストール

```bash
# GitHubからインストール（自動検出）
plm install owner/repo

# コンポーネント種別を指定
plm install owner/repo --type skill
plm install owner/repo --type agent
plm install owner/repo --type prompt
plm install owner/repo --type instruction

# 特定環境のみにインストール
plm install owner/repo --target codex

# Claude Code Pluginからコンポーネントを抽出してインストール
plm install owner/plugin-repo --from-plugin

# スコープ指定（personal/project）
plm install owner/repo --scope project
```

### コンポーネント一覧・情報

```bash
# 全環境の一覧
plm list

# 種別でフィルタ
plm list --type skill
plm list --type agent

# 特定環境のみ
plm list --target codex

# 詳細情報
plm info component-name
```

### コンポーネントの管理

```bash
# 有効/無効切り替え
plm enable component-name --target codex
plm disable component-name --target copilot

# 削除
plm uninstall component-name              # 全環境から
plm uninstall component-name --target codex  # 特定環境のみ

# 更新
plm update                                # 全コンポーネント
plm update component-name                 # 特定コンポーネント
```

### コンポーネント作成・配布

```bash
# 新規テンプレート作成
plm init my-skill --type skill
plm init my-agent --type agent
plm init my-prompt --type prompt

# 配布用パッケージ作成
plm pack ./my-component
```

### 環境間同期

```bash
# コンポーネントを別環境にコピー
plm sync --from codex --to copilot

# 特定種別のみ同期
plm sync --from codex --to copilot --type skill
```

### Claude Code Plugin からのインポート

```bash
# Pluginリポジトリから特定コンポーネントを抽出
plm import owner/claude-plugin --component skills/pdf
plm import owner/claude-plugin --component agents/reviewer

# Plugin内の全skillsをインポート
plm import owner/claude-plugin --type skill
```

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
│   │   ├── mod.rs
│   │   ├── install.rs            # インストール処理
│   │   ├── uninstall.rs          # 削除処理
│   │   ├── list.rs               # 一覧表示
│   │   ├── enable.rs             # 有効化
│   │   ├── disable.rs            # 無効化
│   │   ├── update.rs             # 更新処理
│   │   ├── info.rs               # 情報表示
│   │   ├── init.rs               # テンプレート作成
│   │   ├── pack.rs               # パッケージ化
│   │   ├── target.rs             # ターゲット環境管理
│   │   ├── sync.rs               # 環境間同期
│   │   └── import.rs             # Claude Plugin インポート
│   ├── targets/                  # AI環境アダプター
│   │   ├── mod.rs
│   │   ├── trait.rs              # 共通インターフェース
│   │   ├── codex.rs              # OpenAI Codex
│   │   └── copilot.rs            # VSCode Copilot
│   ├── components/               # コンポーネント種別
│   │   ├── mod.rs
│   │   ├── trait.rs              # 共通インターフェース
│   │   ├── skill.rs              # Skills
│   │   ├── agent.rs              # Agents
│   │   ├── prompt.rs             # Prompts
│   │   └── instruction.rs        # Instructions
│   ├── registry/
│   │   ├── mod.rs
│   │   └── state.rs              # components.json管理
│   ├── github/
│   │   ├── mod.rs
│   │   └── fetcher.rs            # GitHubダウンロード
│   ├── parser/
│   │   ├── mod.rs
│   │   ├── skill_md.rs           # SKILL.md パーサー
│   │   ├── agent_md.rs           # .agent.md パーサー
│   │   ├── prompt_md.rs          # .prompt.md パーサー
│   │   └── plugin_json.rs        # plugin.json パーサー
│   └── config.rs                 # 設定管理
├── tests/
│   └── ...
└── README.md
```

### 依存クレート

```toml
[package]
name = "plm"
version = "0.1.0"
edition = "2021"

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

# UI
colored = "2"
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

### Codexターゲット実装

```rust
pub struct CodexTarget;

impl Target for CodexTarget {
    fn name(&self) -> &str { 
        "codex" 
    }
    
    fn supported_components(&self) -> Vec<ComponentKind> {
        vec![ComponentKind::Skill, ComponentKind::Instruction]
    }
    
    fn component_path(&self, kind: ComponentKind, scope: Scope) -> Option<PathBuf> {
        match (kind, scope) {
            (ComponentKind::Skill, Scope::Personal) => 
                Some(dirs::home_dir()?.join(".codex/skills")),
            (ComponentKind::Skill, Scope::Project) => 
                Some(PathBuf::from(".codex/skills")),
            (ComponentKind::Instruction, Scope::Personal) => 
                Some(dirs::home_dir()?.join(".codex")),  // AGENTS.md
            (ComponentKind::Instruction, Scope::Project) => 
                Some(PathBuf::from(".")),  // AGENTS.md
            _ => None,
        }
    }
    
    // ...
}
```

### Copilotターゲット実装

```rust
pub struct CopilotTarget;

impl Target for CopilotTarget {
    fn name(&self) -> &str { 
        "copilot" 
    }
    
    fn supported_components(&self) -> Vec<ComponentKind> {
        vec![
            ComponentKind::Skill,
            ComponentKind::Agent,
            ComponentKind::Prompt,
            ComponentKind::Instruction,
        ]
    }
    
    fn component_path(&self, kind: ComponentKind, scope: Scope) -> Option<PathBuf> {
        match (kind, scope) {
            (ComponentKind::Skill, Scope::Project) => 
                Some(PathBuf::from(".github/skills")),
            (ComponentKind::Agent, Scope::Personal) => 
                Some(dirs::home_dir()?.join(".copilot/agents")),
            (ComponentKind::Agent, Scope::Project) => 
                Some(PathBuf::from(".github/agents")),
            (ComponentKind::Prompt, Scope::Project) => 
                Some(PathBuf::from(".github/prompts")),
            (ComponentKind::Instruction, Scope::Project) => 
                Some(PathBuf::from(".github")),  // copilot-instructions.md
            _ => None,
        }
    }
    
    // ...
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
instructions_personal = "~/.codex/AGENTS.md"
instructions_project = "AGENTS.md"

[targets.copilot]
skills_project = ".github/skills"
agents_personal = "~/.copilot/agents"
agents_project = ".github/agents"
prompts_project = ".github/prompts"
instructions_project = ".github/copilot-instructions.md"
```

### コンポーネント管理ファイル（`~/.plm/components.json`）

```json
{
  "version": 1,
  "components": [
    {
      "name": "html-educational-material",
      "kind": "skill",
      "source": "github:doi/html-educational-material",
      "version": "1.0.0",
      "commit": "abc1234",
      "installed_at": "2025-01-15T10:30:00Z",
      "updated_at": "2025-01-15T10:30:00Z",
      "targets": {
        "codex": {
          "scope": "personal",
          "enabled": true,
          "path": "~/.codex/skills/html-educational-material"
        },
        "copilot": {
          "scope": "project",
          "enabled": true,
          "path": ".github/skills/html-educational-material"
        }
      }
    },
    {
      "name": "code-reviewer",
      "kind": "agent",
      "source": "github:doi/code-reviewer",
      "version": "0.1.0",
      "targets": {
        "copilot": {
          "scope": "project",
          "enabled": true,
          "path": ".github/agents/code-reviewer.agent.md"
        }
      }
    }
  ]
}
```

---

## 使用例

### 初期セットアップ

```bash
$ plm target add codex
✅ Added target: codex
   Supports: skills, instructions

$ plm target add copilot
✅ Added target: copilot
   Supports: skills, agents, prompts, instructions

$ plm target
📍 Active targets:
   • codex   (skills, instructions)
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

### Agentのインストール

```bash
$ plm install doi/code-reviewer --type agent
📥 Fetching doi/code-reviewer...
📦 Installing to copilot (project)... ✅
⚠️  codex does not support agents (skipped)
✅ Installed agent: code-reviewer v0.1.0
```

### Claude Code Pluginからインポート

```bash
$ plm import anthropics/claude-code-plugins/frontend-design --type skill
📥 Fetching anthropics/claude-code-plugins...
🔍 Found plugin: frontend-design
📦 Extracting skills...
   • frontend-design
📦 Installing to codex... ✅
📦 Installing to copilot... ✅
✅ Imported 1 skill from plugin
```

### 一覧表示

```bash
$ plm list
┌────────────────────────────┬─────────┬───────┬───────────────┬────────┐
│ Name                       │ Version │ Type  │ Targets       │ Source │
├────────────────────────────┼─────────┼───────┼───────────────┼────────┤
│ html-educational-material  │ 1.0.0   │ skill │ codex,copilot │ github │
│ code-reviewer              │ 0.1.0   │ agent │ copilot       │ github │
│ pr-template                │ 0.2.0   │ prompt│ copilot       │ github │
└────────────────────────────┴─────────┴───────┴───────────────┴────────┘

$ plm list --type skill
┌────────────────────────────┬─────────┬───────────────┬────────┐
│ Name                       │ Version │ Targets       │ Source │
├────────────────────────────┼─────────┼───────────────┼────────┤
│ html-educational-material  │ 1.0.0   │ codex,copilot │ github │
│ frontend-design            │ 1.2.0   │ codex,copilot │ plugin │
└────────────────────────────┴─────────┴───────────────┴────────┘
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

### Phase 1: 基盤構築（Day 1-2）

- [ ] Cargoプロジェクト初期化
- [ ] CLI引数パーサー（clap）
- [ ] Component trait定義
- [ ] Target trait定義
- [ ] 設定ファイル読み書き
- [ ] 基本的なエラーハンドリング

### Phase 2: ターゲット実装（Day 2-3）

- [ ] Codexターゲット実装
- [ ] Copilotターゲット実装
- [ ] `plm target` コマンド

### Phase 3: パーサー実装（Day 3-4）

- [ ] SKILL.md パーサー（YAML frontmatter）
- [ ] .agent.md パーサー
- [ ] .prompt.md パーサー
- [ ] plugin.json パーサー（Claude Code Plugin用）

### Phase 4: GitHubダウンロード（Day 4-5）

- [ ] GitHubリポジトリURLパース
- [ ] リリースアセット or デフォルトブランチZIPダウンロード
- [ ] ZIP展開
- [ ] コンポーネント種別の自動検出
- [ ] `plm install` コマンド

### Phase 5: 管理機能（Day 5-6）

- [ ] `plm list` コマンド
- [ ] `plm info` コマンド
- [ ] `plm uninstall` コマンド
- [ ] `plm enable/disable` コマンド

### Phase 6: インポート機能（Day 6-7）

- [ ] Claude Code Plugin構造の解析
- [ ] コンポーネント抽出
- [ ] `plm import` コマンド

### Phase 7: 更新・同期（Day 7-8）

- [ ] コミットハッシュ/タグ比較
- [ ] `plm update` コマンド
- [ ] `plm sync` コマンド

### Phase 8: 作成・配布（Day 8-9）

- [ ] `plm init` コマンド（テンプレート生成）
- [ ] `plm pack` コマンド（ZIP作成）

### Phase 9: UX改善（Day 9-10）

- [ ] プログレスバー（indicatif）
- [ ] カラー出力（colored）
- [ ] テーブル表示（comfy-table）
- [ ] エラーメッセージ改善
- [ ] ヘルプ・ドキュメント

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

### .agent.md

```markdown
---
name: agent-name
description: エージェントの説明
tools: ['search', 'fetch', 'edit']
---

# Agent Instructions

エージェントの指示...
```

### .prompt.md

```markdown
---
name: prompt-name
description: プロンプトの説明
---

# Prompt

プロンプトの内容...
```

### plugin.json (Claude Code)

```json
{
  "name": "plugin-name",
  "version": "1.0.0",
  "description": "Plugin description",
  "author": "author-name"
}
```

---

## 将来の拡張

### 追加ターゲット候補

- Cursor
- Windsurf  
- Aider
- Gemini CLI
- その他SKILL.md対応ツール

### 追加機能候補

- プラグインレジストリ（公開インデックス）
- コンポーネントの依存関係解決
- バージョン固定（lockfile）
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

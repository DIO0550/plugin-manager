# Getting Started

PLMのインストールと基本的な使い方を説明します。

## インストール

```bash
# Cargoでインストール
cargo install plm

# または、ソースからビルド
git clone https://github.com/your-org/plm
cd plm
cargo build --release
```

## 初期セットアップ

### 1. ターゲット環境の追加

まず、使用するAI開発環境をターゲットとして追加します。

```bash
$ plm target add codex
✅ Added target: codex
   Supports: skills, agents, instructions, hooks

$ plm target add copilot
✅ Added target: copilot
   Supports: skills, agents, commands, instructions, hooks

$ plm target add antigravity
✅ Added target: antigravity
   Supports: skills

$ plm target add gemini
✅ Added target: gemini
   Supports: skills, instructions

$ plm target add cursor
✅ Added target: cursor
   Supports: skills, agents, commands, instructions, hooks
```

ターゲットの確認:

```bash
$ plm target list
📍 Active targets:
   • antigravity (skills)
   • codex       (skills, agents, instructions, hooks)
   • copilot     (skills, agents, commands, instructions, hooks)
   • cursor      (skills, agents, commands, instructions, hooks)
   • gemini      (skills, instructions)
```

### 2. マーケットプレイスの登録（オプション）

マーケットプレイスを使用する場合は、登録します。

```bash
$ plm marketplace add company/claude-plugins --name company-tools
📥 Fetching marketplace.json...
✅ Added marketplace: company-tools
   Available plugins: 5
```

## 基本的な使い方

### Skillのインストール

GitHubリポジトリから直接インストール:

```bash
$ plm install doi/html-educational-material
📥 Fetching doi/html-educational-material...
🔍 Detected: skill
📦 Installing to codex (personal)... ✅
📦 Installing to copilot (project)... ✅
✅ Installed skill: html-educational-material v1.0.0
```

### マーケットプレイス経由でインストール

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

### インストール済みコンポーネントの確認

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

### TUI管理画面

インタラクティブな管理画面を起動:

```bash
plm managed
```

TUIでは以下の操作が可能です:

- プラグインの有効/無効切替
- 更新チェックと適用
- アンインストール
- GitHubページを開く

## スコープの指定

インストール先のスコープを指定できます:

```bash
# Personal スコープ（~/.codex/, ~/.copilot/）
plm install owner/repo --scope personal

# Project スコープ（.codex/, .github/）- デフォルト
plm install owner/repo --scope project
```

## ターゲットの指定

特定のターゲット環境のみにインストール:

```bash
# Codexのみ
plm install owner/repo --target codex

# Copilotのみ
plm install owner/repo --target copilot

# Cursorのみ
plm install owner/repo --target cursor
```

## インタラクティブ選択

`--target`未指定時は、有効なターゲットから選択UIが表示されます:

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

## 次のステップ

- [コマンドリファレンス](./commands/index.md) - すべてのコマンドの詳細
- [ターゲット環境](./concepts/targets.md) - Codex/Copilot/Cursor等の違い
- [コンポーネント種別](./concepts/components.md) - Skills, Agents, Prompts, Instructions
- [マーケットプレイス](./concepts/marketplace.md) - マーケットプレイスの仕組み

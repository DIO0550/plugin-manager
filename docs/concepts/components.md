# コンポーネント種別

PLMが管理するコンポーネントの種類について説明します。

## コンポーネント一覧

| 種別 | 説明 | ファイル形式 |
|------|------|-------------|
| **Skills** | 専門的な知識・ワークフロー | `SKILL.md` (YAML frontmatter) |
| **Agents** | カスタムエージェント定義 | `*.agent.md` |
| **Commands** | スラッシュコマンド | `*.prompt.md` |
| **Instructions** | コーディング規約・カスタム指示 | `AGENTS.md` / `copilot-instructions.md` / `GEMINI.md` |
| **Hooks** | イベントハンドラ | 任意のスクリプト |

## Skills

専門的な知識やワークフローを定義するコンポーネントです。

### 特徴

- YAML frontmatterでメタデータを定義
- 専門的なタスクを実行するための詳細な指示を含む
- Codex、Copilot、Gemini CLIでサポート（Antigravityも対応）

### ファイル形式

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

### 配置場所

| ターゲット | Personal | Project |
|------------|----------|---------|
| Codex | `~/.codex/skills/<marketplace>/<plugin>/<skill>/` | `.codex/skills/<marketplace>/<plugin>/<skill>/` |
| Copilot | - | `.github/skills/<marketplace>/<plugin>/<skill>/` |
| Antigravity | `~/.gemini/antigravity/skills/<marketplace>/<plugin>/<skill>/` | `.agent/skills/<marketplace>/<plugin>/<skill>/` |
| Gemini CLI | `~/.gemini/skills/<marketplace>/<plugin>/<skill>/` | `.gemini/skills/<marketplace>/<plugin>/<skill>/` |

## Agents

カスタムエージェントを定義するコンポーネントです。

### 特徴

- 特定のタスクに特化したエージェントを定義
- 使用可能なツールを指定可能
- Copilotで公式サポート、Codexは将来対応見込み

### ファイル形式

```markdown
---
name: agent-name
description: エージェントの説明
tools: ['search', 'fetch', 'edit']
---

# Agent Instructions

エージェントの指示...
```

### 配置場所

| ターゲット | Personal | Project |
|------------|----------|---------|
| Codex | `~/.codex/agents/<marketplace>/<plugin>/` | `.codex/agents/<marketplace>/<plugin>/` |
| Copilot | `~/.copilot/agents/<marketplace>/<plugin>/` | `.github/agents/<marketplace>/<plugin>/` |

## Commands

スラッシュコマンドを定義するコンポーネントです。

### 特徴

- Claude Code のスラッシュコマンド（`.prompt.md`形式）を他ターゲットにも展開
- Copilotでのみサポート（Copilot Prompt Files として配置）
- 手動で呼び出して使用

### ファイル形式

```markdown
---
name: command-name
description: コマンドの説明
---

# Command

コマンドの内容...
```

### 配置場所

| ターゲット | Personal | Project |
|------------|----------|---------|
| Codex | - | - |
| Copilot | - | `.github/prompts/<marketplace>/<plugin>/` |

## Instructions

プロジェクト固有のコーディング規約やカスタム指示を定義するコンポーネントです。

### 特徴

- AGENTS.md形式のオープン標準（Linux Foundation管轄）
- プロジェクト全体に適用される指示
- Codex、Copilot、Gemini CLIでサポート（Gemini CLIは`GEMINI.md`で対応）

### ファイル形式

```markdown
# Project Guidelines

プロジェクト固有のコーディング規約やワークフロー...
```

### 配置場所

| ターゲット | Personal | Project |
|------------|----------|---------|
| Codex | `~/.codex/AGENTS.md` | `AGENTS.md` |
| Copilot | - | `.github/copilot-instructions.md`, `AGENTS.md` |
| Gemini CLI | `~/.gemini/GEMINI.md` | `GEMINI.md` |

## ターゲット別サポート状況

| コンポーネント | Codex | Copilot | Antigravity | Gemini CLI |
|----------------|-------|---------|-------------|------------|
| Skills | ✅ | ✅ | ✅ | ✅ |
| Agents | ✅ | ✅ | ❌ | ❌ |
| Commands | ❌ | ✅ | ❌ | ❌ |
| Instructions | ✅ | ✅ | ❌ | ✅* |
| Hooks | ❌ | ❌ | ❌ | ❌ |

> *Gemini CLIは`GEMINI.md`形式で対応（`AGENTS.md`は設定で変更可能）。

## 共通規格

| 規格 | 説明 | 参照 |
|------|------|------|
| **AGENTS.md** | カスタム指示ファイル（Linux Foundation管轄のオープン標準） | https://agents.md |
| **SKILL.md** | スキル定義（Anthropicがオープン標準として公開、OpenAI/Microsoft/Googleが採用） | - |
| **GEMINI.md** | Gemini CLI用のコンテキスト・指示ファイル | [Gemini CLI Docs](https://geminicli.com/docs/cli/gemini-md/) |

## 関連

- [concepts/targets](./targets.md) - ターゲット環境の詳細
- [commands/init](../commands/init.md) - コンポーネントテンプレート作成

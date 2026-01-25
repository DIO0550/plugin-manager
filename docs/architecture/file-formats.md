# ファイルフォーマット仕様

各AI開発環境（Claude Code、Copilot、Codex）のコンポーネントファイル形式を定義します。

## 概要

PLMはClaude Code Pluginからコンポーネントをインポートし、Codex/Copilotへ変換・配置します。
各環境でファイル形式が異なるため、変換が必要です。

| コンポーネント | 形式の違い | 変換要否 |
|---------------|-----------|---------|
| Skills | 共通形式（SKILL.md） | 不要 |
| Agents | 環境ごとに差異あり | **必要** |
| Commands/Prompts | 環境ごとに異なる | **必要** |
| Instructions | 共通形式（AGENTS.md） | 不要 |

---

## Claude Code

### Commands

**パス:** `.claude/commands/<name>.md`

```yaml
---
name: commit-helper
description: Create a git commit with conventional message
allowed-tools: Bash(git add:*), Bash(git commit:*)
argument-hint: [message]
model: haiku
disable-model-invocation: false
user-invocable: true
---

Commit the staged changes with the message: $ARGUMENTS

Use $1 for the first argument, $2 for the second.
```

#### Frontmatter フィールド

| フィールド | 型 | 必須 | 説明 |
|-----------|-----|------|------|
| `name` | string | - | コマンド識別子（未指定時はファイル名） |
| `description` | string | ○ | コマンドの説明 |
| `allowed-tools` | string | - | 使用可能ツール（例: `Bash(git:*), Read, Write`） |
| `argument-hint` | string | - | 引数のヒント（例: `[message]`） |
| `model` | string | - | 使用モデル（`haiku`, `sonnet`, `opus`） |
| `disable-model-invocation` | bool | - | モデルからの自動呼び出しを禁止（デフォルト: false） |
| `user-invocable` | bool | - | ユーザーから呼び出し可能か（デフォルト: true） |

#### 本文内変数

| 変数 | 説明 |
|------|------|
| `$ARGUMENTS` | 全引数 |
| `$1`, `$2`, ... | 位置引数 |

### Skills

**パス:** `.claude/skills/<name>/SKILL.md`

```yaml
---
name: pdf-processing
description: Extract text, fill forms, merge PDFs
allowed-tools: Bash, Read, Write
model: sonnet
context: fork
---

You are a PDF processing expert...
```

#### Frontmatter フィールド

| フィールド | 型 | 必須 | 説明 |
|-----------|-----|------|------|
| `name` | string | ○ | スキル識別子（kebab-case） |
| `description` | string | ○ | スキルの説明 |
| `allowed-tools` | string | - | 使用可能ツール |
| `model` | string | - | 使用モデル |
| `context` | string | - | `fork`: 分離コンテキストで実行 |

### Agents

**パス:** `.claude/agents/<name>.md`

```yaml
---
name: code-reviewer
description: Expert code review specialist
tools: Read, Grep, Glob, Bash
model: opus
---

You are a code review expert...
```

#### Frontmatter フィールド

| フィールド | 型 | 必須 | 説明 |
|-----------|-----|------|------|
| `name` | string | ○ | エージェント識別子 |
| `description` | string | ○ | エージェントの説明 |
| `tools` | string | - | 使用可能ツール（カンマ区切り） |
| `model` | string | - | 使用モデル |

---

## Copilot

### Prompts

**パス:** `.github/prompts/<name>.prompt.md`

```yaml
---
name: commit-helper
description: Create a git commit with conventional message
tools: ['githubRepo', 'codebase']
hint: "Enter commit message"
model: GPT-4o
agent: coding-agent
---

Create a commit with the message: ${message}

Reference files using ${file:path/to/file.ts}
```

#### Frontmatter フィールド

| フィールド | 型 | 必須 | 説明 |
|-----------|-----|------|------|
| `name` | string | - | プロンプト名（未指定時はファイル名） |
| `description` | string | - | プロンプトの説明 |
| `tools` | string[] | - | 使用可能ツール（配列形式） |
| `hint` | string | - | 入力フィールドのヒント |
| `model` | string | - | 使用モデル（`GPT-4o`, `GPT-4o-mini`, `o1`） |
| `agent` | string | - | 参照するエージェント名 |

#### 本文内変数

| 変数 | 説明 |
|------|------|
| `${variableName}` | 名前付き変数 |
| `${file:path}` | ファイル参照 |
| `#tool:<tool-name>` | ツール参照 |

### Skills

**パス:** `.github/skills/<name>/SKILL.md`

Codex/Claude Codeと共通形式。

### Agents

**パス:** `.github/agents/<name>.agent.md`

```yaml
---
name: code-reviewer
description: Expert code review specialist
tools: ['codebase', 'githubRepo']
model: GPT-4o
target: vscode
handoffs:
  - agent: fixer
    label: "Fix issues"
    prompt: "Fix the issues found"
    send: true
---

You are a code review expert...
```

#### Frontmatter フィールド

| フィールド | 型 | 必須 | 説明 |
|-----------|-----|------|------|
| `name` | string | - | エージェント名 |
| `description` | string | - | エージェントの説明 |
| `tools` | string[] | - | 使用可能ツール（配列形式） |
| `model` | string | - | 使用モデル |
| `target` | string | - | 対象環境（`vscode`, `github-copilot`） |
| `handoffs` | object[] | - | ワークフロー遷移定義 |

#### handoffs フィールド

| フィールド | 型 | 説明 |
|-----------|-----|------|
| `agent` | string | 遷移先エージェント名 |
| `label` | string | ボタンラベル |
| `prompt` | string | 遷移時のプロンプト |
| `send` | bool | 自動送信するか |

---

## Codex

### Prompts (Custom Prompts)

**パス:** `~/.codex/prompts/<name>.md`

```yaml
---
description: Create a git commit with conventional message
---

Commit the staged changes with the provided message.
```

#### Frontmatter フィールド

| フィールド | 型 | 必須 | 説明 |
|-----------|-----|------|------|
| `description` | string | - | プロンプトの説明 |

> Note: Codexのカスタムプロンプトはシンプルな形式で、Claude Codeほどのフィールドを持たない。

### Skills

**パス:** `~/.codex/skills/<name>/SKILL.md` または `.codex/skills/<name>/SKILL.md`

```yaml
---
name: pdf-processing
description: Extract text, fill forms, merge PDFs
metadata:
  short-description: PDF processing utilities
---

You are a PDF processing expert...
```

#### Frontmatter フィールド

| フィールド | 型 | 必須 | 説明 |
|-----------|-----|------|------|
| `name` | string | ○ | スキル識別子 |
| `description` | string | ○ | スキルの説明 |
| `metadata` | object | - | 追加メタデータ |

### Agents

Codexは現時点で`.agent.md`形式を公式サポートしていない。
`AGENTS.md`による指示ファイルのみ対応。

---

## 変換マッピング

Claude Code形式から各環境への変換時のフィールド割り当てを定義します。

### Command → Copilot Prompt

| Claude Code | Copilot | 変換方法 |
|-------------|---------|----------|
| `name` | `name` | そのまま |
| `description` | `description` | そのまま |
| `allowed-tools` | `tools` | カンマ区切り → 配列、ツール名変換 |
| `argument-hint` | `hint` | `[message]` → `"Enter message"` 形式に |
| `model` | `model` | モデル名変換（後述） |
| `disable-model-invocation` | - | 削除（Copilot非対応） |
| `user-invocable` | - | 削除（Copilot非対応） |
| - | `agent` | 設定しない |
| 本文 `$ARGUMENTS` | 本文 `${arguments}` | 変数形式変換 |
| 本文 `$1`, `$2` | 本文 `${arg1}`, `${arg2}` | 変数形式変換 |

**変換例:**

```yaml
# Claude Code (入力)
---
allowed-tools: Bash(git:*), Read, Write
argument-hint: [commit message]
model: haiku
---
Commit with message: $ARGUMENTS

# Copilot (出力)
---
tools: ['githubRepo', 'codebase']
hint: "Enter commit message"
model: GPT-4o-mini
---
Commit with message: ${arguments}
```

### Command → Codex Prompt

| Claude Code | Codex | 変換方法 |
|-------------|-------|----------|
| `name` | - | ファイル名として使用 |
| `description` | `description` | そのまま |
| `allowed-tools` | - | 削除（Codex非対応） |
| `argument-hint` | - | 削除（Codex非対応） |
| `model` | - | 削除（Codex非対応） |
| `disable-model-invocation` | - | 削除 |
| `user-invocable` | - | 削除 |
| 本文 `$ARGUMENTS` | 本文 | そのまま（Codexは変数未対応） |

> Note: Codexのカスタムプロンプトは最小限のフィールドのみ対応。

### Agent → Copilot Agent

| Claude Code | Copilot | 変換方法 |
|-------------|---------|----------|
| `name` | `name` | そのまま |
| `description` | `description` | そのまま |
| `tools` | `tools` | カンマ区切り → 配列、ツール名変換 |
| `model` | `model` | モデル名変換（後述） |
| - | `target` | `vscode` を設定 |
| - | `handoffs` | 設定しない |

**変換例:**

```yaml
# Claude Code (入力)
---
name: code-reviewer
description: Expert code review specialist
tools: Read, Grep, Glob, Bash
model: opus
---

# Copilot (出力)
---
name: code-reviewer
description: Expert code review specialist
tools: ['codebase', 'search/codebase', 'terminal']
model: o1
target: vscode
---
```

### Agent → Codex

Codexは`.agent.md`形式を公式サポートしていないため、変換不可。
`AGENTS.md`への追記として対応する場合は、本文のみを使用。

---

## ツール名対応表

| Claude Code | Copilot | 説明 |
|-------------|---------|------|
| `Read` | `codebase` | ファイル読み取り |
| `Write` | `codebase` | ファイル書き込み |
| `Edit` | `codebase` | ファイル編集 |
| `Grep` | `search/codebase` | コード検索 |
| `Glob` | `search/codebase` | ファイル検索 |
| `Bash` | `terminal` | シェル実行 |
| `Bash(git:*)` | `githubRepo` | Git操作 |
| `WebFetch` | `fetch` | HTTP取得 |
| `WebSearch` | `websearch` | Web検索 |

## モデル名対応表

| Claude Code | Copilot | Codex | 特性 |
|-------------|---------|-------|------|
| `haiku` | `GPT-4o-mini` | `gpt-4.1-mini` | 高速・低コスト |
| `sonnet` | `GPT-4o` | `gpt-4.1` | バランス型 |
| `opus` | `o1` | `o3` | 高性能 |

---

## 参考リンク

### Claude Code
- [Slash Commands](https://code.claude.com/docs/en/slash-commands)
- [Skills](https://code.claude.com/docs/en/skills)

### Copilot
- [Prompt Files](https://code.visualstudio.com/docs/copilot/customization/prompt-files)
- [Custom Agents](https://docs.github.com/en/copilot/how-tos/use-copilot-agents/coding-agent/create-custom-agents)

### Codex
- [Custom Prompts](https://developers.openai.com/codex/custom-prompts/)
- [Agent Skills](https://developers.openai.com/codex/skills/)
- [AGENTS.md](https://developers.openai.com/codex/guides/agents-md)

---

## 関連Issue

- [#8 [Epic] Phase 3: パーサー実装](https://github.com/DIO0550/plugin-manager/issues/8)
- [#9 .agent.md パーサー実装](https://github.com/DIO0550/plugin-manager/issues/9)
- [#10 .prompt.md パーサー実装](https://github.com/DIO0550/plugin-manager/issues/10)
- [#11 SKILL.md パーサー実装](https://github.com/DIO0550/plugin-manager/issues/11)
- [#12 plugin.json パーサー実装](https://github.com/DIO0550/plugin-manager/issues/12)

## 関連ドキュメント

- [concepts/components](../concepts/components.md) - コンポーネント種別
- [concepts/targets](../concepts/targets.md) - ターゲット環境
- [overview](./overview.md) - アーキテクチャ概要

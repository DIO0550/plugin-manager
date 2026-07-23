# ファイルフォーマット仕様

各AI開発環境（Claude Code、Copilot、Codex、Gemini CLI、Cursor）のコンポーネントファイル形式を定義します。

## 概要

PLMはClaude Code Pluginからコンポーネントをインポートし、Codex/Copilot/Gemini CLI/Cursorへ変換・配置します。
各環境でファイル形式が異なるため、変換が必要です。

| コンポーネント | 形式の違い | 変換要否 |
|---------------|-----------|---------|
| Skills | 共通形式（SKILL.md） | 不要（Gemini CLIは frontmatter 削減あり） |
| Agents | 環境ごとに差異あり | **必要**（Cursorは内容無変換・拡張子のみ） |
| Commands/Prompts | 環境ごとに異なる | **必要**（Cursorは内容無変換・拡張子のみ） |
| Instructions | 共通形式（AGENTS.md） | 不要 |
| Hooks | 環境ごとに差異あり | **必要**（Cursor含む） |

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

## Gemini CLI

v0.23.0（2026年1月）でAgent Skills（実験的機能）が追加。Claude Code Skillsと同じ`SKILL.md`形式を採用。

### Skills

**パス:** `.gemini/skills/<marketplace>/<plugin>/<skill>/SKILL.md`（Workspace） / `~/.gemini/skills/<marketplace>/<plugin>/<skill>/SKILL.md`（User）

```yaml
---
name: pdf-processing
description: Extract text, fill forms, merge PDFs
---

You are a PDF processing expert...
```

#### Frontmatter フィールド

| フィールド | 型 | 必須 | 説明 |
|-----------|-----|------|------|
| `name` | string | ○ | スキル識別子（小文字英数字、ハイフン） |
| `description` | string | ○ | スキルの説明（アクティベーション判定に使用） |

> Note: Gemini CLI SkillsのfrontmatterはClaude Code/Codexより最小限。`allowed-tools`、`model`、`context`等のフィールドは非対応。

### Instructions (GEMINI.md)

**パス:** `~/.gemini/GEMINI.md`（Global） / `./GEMINI.md`（Project、階層走査）

```markdown
# Project Guidelines

プロジェクト固有の指示やコーディング規約...
```

- frontmatterなし（Markdownのみ）
- 親ディレクトリからプロジェクトルート（`.git`フォルダ）まで走査して連結
- `@file.md` 構文で他ファイルの内容をインポート可能
- `.gemini/settings.json` の `contextFileName` でファイル名を変更可能（例: `"AGENTS.md"`）

### Agents / Prompts

Gemini CLIは `.agent.md`、`.prompt.md` 形式を公式サポートしていない。

---

## Cursor

Cursor は Claude Code 互換の Skills / Agents / Commands / AGENTS.md / Hooks をサポートする。PLM は `.cursor/` 配下へフラット配置する。

### Skills

**パス:** `~/.cursor/skills/<flattened_name>/SKILL.md` / `.cursor/skills/<flattened_name>/SKILL.md`

Claude Code / Codex と同じ `SKILL.md`（YAML frontmatter + Markdown）。内容変換は不要。

### Agents

**パス:** `~/.cursor/agents/<flattened_name>.md` / `.cursor/agents/<flattened_name>.md`

プレーン Markdown（YAML frontmatter 可）。PLM 内部の `.agent.md` サフィックスは Cursor では認識されないため、配置時に `.md` へリネームする（内容は無変換）。

### Commands

**パス:** `~/.cursor/commands/<flattened_name>.md` / `.cursor/commands/<flattened_name>.md`

プレーン Markdown。`.prompt.md` は `.md` へリネーム（内容無変換）。`/migrate-to-skills` による Skills 移行が進んでいるが、`.cursor/commands/` は引き続き動作する。

### Instructions (AGENTS.md)

**パス:** プロジェクトルートの `AGENTS.md`（Project スコープのみ）

Personal（User Rules）はアプリ設定画面管理のため PLM 対象外。Codex の Project `AGENTS.md` と同一ファイルを共有する。

### Hooks

**パス:** `~/.cursor/hooks.json` / `.cursor/hooks.json`

```json
{
  "version": 1,
  "hooks": {
    "sessionStart": [{ "command": "./scripts/setup.sh" }],
    "preToolUse": [{ "command": "./scripts/validate.sh", "timeout": 15 }]
  }
}
```

イベント名は camelCase。Claude Code から変換して配置する（詳細は変換マッピング参照）。

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

### Skill → Codex Skill

`SKILL.md` の frontmatter は `name` / `description` / `metadata` のみ保持し、それ以外の
top-level フィールドはデプロイ時に削除する（ディレクトリ内の他ファイルはそのままコピー）。

| Claude Code | Codex | 変換方法 |
|-------------|-------|----------|
| `name` | `name` | そのまま |
| `description` | `description` | そのまま |
| `metadata` | `metadata` | そのまま（ネストを保持） |
| `allowed-tools` | - | 削除（Codex非対応） |
| `argument-hint` | - | 削除（Codex非対応） |
| `model` | - | 削除（Codex非対応） |
| `disable-model-invocation` | - | 削除（Codex非対応） |
| `context` | - | 削除（Codex非対応） |

> Note: 削除は frontmatter を YAML 再パースせず**行単位**で行う。これは、非対応フィールドが
> 不正な YAML 値（例: `argument-hint: [threshold] [min-lines]` はフローシーケンスとして壊れる）
> を含む場合でも、該当行を安全に取り除き Codex 側の読み込みエラーを防ぐためである。

### Skill → Gemini CLI Skill

| Claude Code | Gemini CLI | 変換方法 |
|-------------|-----------|----------|
| `name` | `name` | そのまま |
| `description` | `description` | そのまま |
| `metadata` | - | 削除（Gemini CLI非対応） |
| `allowed-tools` | - | 削除（Gemini CLI非対応） |
| `model` | - | 削除（Gemini CLI非対応） |
| `context` | - | 削除（Gemini CLI非対応） |
| `disable-model-invocation` | - | 削除（Gemini CLI非対応） |
| `argument-hint` | - | 削除（Gemini CLI非対応） |

> Note: Gemini CLI Skills の frontmatter は Claude Code / Codex より限定的で、`name` と
> `description` の **2 フィールドのみ**対応する。それ以外の top-level フィールド（`metadata`
> を含む）はすべて削除する。Codex（`metadata` を保持）との違いに注意。削除は行ベースで行う
> （「Skill → Codex Skill」と同方式）。

### Command → Gemini CLI

Gemini CLIはCommand/Prompt相当の機能を持たないため、変換不可。

### Agent → Gemini CLI

Gemini CLIは`.agent.md`形式をサポートしていないため、変換不可。

### Skill / Agent / Command / Instructions → Cursor

Cursor は Claude Code 互換パス（`.claude/skills/` 等）も読むため、**内容変換は原則不要**。

| コンポーネント | Claude Code | Cursor（PLM 配置） | 変換方法 |
|---------------|-------------|-------------------|----------|
| Skills | `SKILL.md` | `SKILL.md` | 無変換。`<flattened_name>/` 配下に配置 |
| Agents | `*.agent.md` / `*.md` | `<flattened_name>.md` | 内容無変換。`.agent.md` サフィックスはプレーン `.md` にリネーム |
| Commands | `*.prompt.md` / `*.md` | `<flattened_name>.md` | 内容無変換。`.prompt.md` サフィックスはプレーン `.md` にリネーム |
| Instructions | `AGENTS.md` | `AGENTS.md` | 無変換。Project スコープのみ（プロジェクトルート） |

> Note: Cursor 公式は Agents / Commands の再帰走査を明記していないため、PLM はフラット配置を採用する。

### Hooks → Cursor

Claude Code hooks を Cursor `hooks.json`（`version: 1` + camelCase イベント）へ変換する。

| Claude Code | Cursor | 変換方法 |
|-------------|--------|----------|
| イベント名（PascalCase） | camelCase | `SessionStart`→`sessionStart`、`UserPromptSubmit`→`beforeSubmitPrompt` 等 |
| `command` / `timeout` | 同名 | 保持 |
| `type` | `type` | 保持（command / http / prompt / agent） |
| `async` / `once` / `bash` | - | 削除＋警告 |
| `statusMessage` / `comment` | - | 削除＋警告 |
| `timeoutSec` | `timeout` | リネーム |
| ルート構造 | `{"version": 1, "hooks": {...}}` | Copilot CLI に近い単一ファイル形式 |

配置先はスコープごとに単一ファイル（`~/.cursor/hooks.json` / `.cursor/hooks.json`）。フルマージ未実装のため、既存の非管理ファイル上書きと複数 Hook コンポーネントの同時配置は拒否する。

詳細なイベント・ツール対応は `docs/hooks-conversion/` および `src/hooks/converter/cursor.rs` を参照。

---

## Skill 付属リソース

Skill ディレクトリ（直下に `SKILL.md` を持つディレクトリ）内のエントリのうち、PLM が別 Component（Skill / Agent / Command / Instruction / Hook）としてスキャン・登録しないファイル・フォルダは、**当該 Skill の付属リソース**として扱う。

### 契約

| 項目 | 内容 |
|------|------|
| 対象 | Skill ディレクトリ内の全エントリ（`SKILL.md` 以外の直下ファイル、任意名サブフォルダとその中身） |
| フォルダ名 | 制約なし（`references/` / `assets/` / `templates/` / `examples/` / `docs/` / 任意名）。ホワイトリスト方式は採らない |
| 配置 | `deploy_skill` が `replace_dir` で Skill ディレクトリごとコピーし、相対構造をターゲット側に再現する |
| frontmatter 変換 | ターゲットがサポートしないフィールドの除去は **`target_path/SKILL.md` のみ**。付属 md は触らない |
| stale 掃除 | `replace_dir` により、更新時に source から消えた付属ファイルはターゲット側からも削除される |
| スキャン | Skill 採用後は配下に潜らない。配下の `SKILL.md` は別 Skill として登録しない（付属としてコピーはする） |

```text
skills/spec-driven-dev/
├── SKILL.md                 # Skill 本体
├── notes.md                 # 付属（直下ファイル）
├── references/              # 付属（任意名フォルダ）
│   └── exploration.md
└── assets/templates/
    └── review.html
```

配置後（例: Codex）も同じ相対パスがターゲット配下に存在する。

### 境界・既知制限

- **Plugin 直下**（例: `plugins/<plugin>/references/`）の未認識ファイル/フォルダは本節の対象外。
- **symlink** のコピー挙動は保証外。
- **Cursor** は skills ルートを再帰走査するため、Skill 配下にネストした `SKILL.md` があると実行時に別 Skill として見える可能性がある。PLM はこれを別 Component にせず、変換・除外もしない。

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

| Claude Code | Copilot | Codex | Gemini CLI | 特性 |
|-------------|---------|-------|------------|------|
| `haiku` | `GPT-4o-mini` | `gpt-4.1-mini` | `gemini-2.0-flash` | 高速・低コスト |
| `sonnet` | `GPT-4o` | `gpt-4.1` | `gemini-2.5-pro` | バランス型 |
| `opus` | `o1` | `o3` | `gemini-2.5-pro` | 高性能 |

> Note: Gemini CLI Skillsはfrontmatterで`model`フィールドを持たないため、モデル変換は参考情報。

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

### Gemini CLI
- [Agent Skills](https://geminicli.com/docs/cli/skills/)
- [Getting Started with Agent Skills](https://geminicli.com/docs/cli/tutorials/skills-getting-started/)
- [GEMINI.md Context Files](https://geminicli.com/docs/cli/gemini-md/)
- [Configuration](https://geminicli.com/docs/get-started/configuration/)

### Cursor
- [Agent Skills](https://cursor.com/docs/context/skills)
- [Subagents](https://cursor.com/docs/agent/subagents)
- [Rules / AGENTS.md](https://cursor.com/docs/context/rules)
- [Hooks](https://cursor.com/docs/agent/hooks)

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

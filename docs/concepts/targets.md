# ターゲット環境

PLMがサポートするAI開発環境（ターゲット）について説明します。

## 対応ターゲット

| ターゲット | 説明 | 状態 |
|------------|------|------|
| **codex** | OpenAI Codex CLI | ✅ 対応済み |
| **copilot** | VSCode GitHub Copilot | ✅ 対応済み |
| **antigravity** | Google Antigravity（IDE / CLI・Hooks 共通） | ✅ 対応済み |
| **gemini** | Gemini CLI（ターミナルベースAIエージェント） | ✅ 対応済み |
| **cursor** | Cursor（IDE / CLI） | ✅ 対応済み |
| **opencode** | OpenCode（ターミナル AI エージェント） | ✅ 対応済み |

> **上流仕様の追随状況**: 各ターゲットの公式仕様は継続的に更新される。最終調査日と未対応 TODO は
> [`docs/reference/upstream-spec-updates.md`](../reference/upstream-spec-updates.md) を参照（最終調査: 2026-08-20）。

## サポートするコンポーネント

| コンポーネント | Codex | Copilot | Antigravity | Gemini CLI | Cursor | OpenCode |
|----------------|-------|---------|-------------|------------|--------|----------|
| Skills | ✅ | ✅ | ✅ | ✅ | ✅ | ✅******* |
| Agents | ✅ | ✅ | ❌* | ❌ | ✅ | ✅******* |
| Commands | ❌ | ✅ | ❌* | ❌ | ✅ | ✅******* |
| Instructions | ✅ | ✅ | ❌* | ✅** | ✅*** | ✅******** |
| Hooks | ✅ | ✅ | ✅**** | ❌****** | ✅***** | ❌********* |

> *Antigravity は公式に Agents（`agent.md`）/ Workflows（Commands 相当）/ Rules・`GEMINI.md`・`AGENTS.md`（Instructions 相当）をサポートする。PLM 実装は未着手（調査: [#400](https://github.com/DIO0550/plugin-manager/issues/400)）。
> **Gemini CLIは`GEMINI.md`による階層的な指示システムを持ちます。
> ***CursorのInstructionsはProjectスコープ（`AGENTS.md`）のみ。Personalスコープの指示（User Rules）はアプリ設定画面で管理されるため対象外。
> ****Antigravity（IDE / CLI 共通）Hooks は公式 5 イベント対応の変換・配置を実装済み。単一 `hooks.json`・複数 Hook 同時配置は拒否（フルマージ未実装）。stdin/stdout ラッパーは未生成（インライン command）。
> *****CursorのHooksは単一の `hooks.json` に配置する。既存の非管理ファイルの上書きと、同一インストール内の複数 Hook コンポーネントは拒否する（フルマージは未実装）。
> ******Gemini CLI の Hooks は非対応。一般向けは Antigravity CLI へ移行済みで、Hooks は Antigravity（IDE / CLI 共通）仕様として扱う。Enterprise 向け `gemini` ターゲットはレガシーとして維持する。
> *******OpenCode の Skills / Agents / Commands はファイルベースで配置する（Skills は `original_name`。Epic [#416](https://github.com/DIO0550/plugin-manager/issues/416)）。
> ********OpenCode の Instructions は Personal（`~/.config/opencode/AGENTS.md`）と Project（`AGENTS.md`）の両方。
> *********OpenCode の拡張は JSON Hooks ではなく TypeScript/JavaScript Plugin。PLM の Hook コンポーネントとはモデルが異なるため対象外。

## OpenAI Codex

### 読み込みパスと優先順位

公式ドキュメント: [Custom instructions with AGENTS.md](https://developers.openai.com/codex/guides/agents-md/)

| スコープ | パス | 自動読み込み | 備考 |
|---------|------|--------------|------|
| Global (override) | `~/.codex/AGENTS.override.md` | ✅ | 最優先 |
| Global | `~/.codex/AGENTS.md` | ✅ | Personal対応 |
| Project | `./AGENTS.override.md` | ✅ | ディレクトリ毎 |
| Project | `./AGENTS.md` | ✅ | ディレクトリ毎 |
| Skills (Global) | `~/.codex/skills/` | ✅ | Personal |
| Skills (Project) | `./.codex/skills/` | ✅ | Project |

### 読み込み順序

1. **Global scope**: `~/.codex/` (または `$CODEX_HOME`) をチェック
   - `AGENTS.override.md` があればそれを使用、なければ `AGENTS.md`
2. **Project scope**: リポジトリルートから現在ディレクトリまで走査
   - 各ディレクトリで `AGENTS.override.md` → `AGENTS.md` → fallback の順
3. **マージ**: ルートから現在ディレクトリに向かって連結（上限: `project_doc_max_bytes` = 32KiB）

### コンポーネント配置場所

| 種別 | ファイル形式 | Personal | Project |
|------|-------------|----------|---------|
| Skills | `SKILL.md` | `~/.codex/skills/<marketplace>/<plugin>/<skill>/` | `.codex/skills/<marketplace>/<plugin>/<skill>/` |
| Agents | `*.agent.md` | `~/.codex/agents/<marketplace>/<plugin>/` | `.codex/agents/<marketplace>/<plugin>/` |
| Instructions | `AGENTS.md` | `~/.codex/AGENTS.md` | `AGENTS.md` |

### Hooks（10 イベント対応）

公式ドキュメント: [Codex Hooks](https://developers.openai.com/codex/hooks)（現行 URL: [learn.chatgpt.com/docs/hooks](https://learn.chatgpt.com/docs/hooks)。URL 移行対応は [#463](https://github.com/DIO0550/plugin-manager/issues/463)）

> **TODO（2026-08-20 調査）**: 上流に `SessionEnd` イベントと `async` フィールドが追加されたが PLM 未対応（[#455](https://github.com/DIO0550/plugin-manager/issues/455)）。
> また hooks は上流で既定有効になり `codex_hooks` は deprecated alias となったため、`[features] codex_hooks = true` の自動追記を見直す（[#456](https://github.com/DIO0550/plugin-manager/issues/456)）。

Codex CLI は PascalCase 命名の hooks イベントを 10 種サポートし、PLM の `CodexEventMap` はそれらをすべて変換対象として保持する（イベント名は変換時にそのまま維持）。

| イベント | scope |
|----------|-------|
| `SessionStart` | thread |
| `PreToolUse` | turn |
| `PermissionRequest` | turn |
| `PostToolUse` | turn |
| `UserPromptSubmit` | turn |
| `Stop` | turn |
| `PreCompact` | turn |
| `PostCompact` | turn |
| `SubagentStop` | turn |
| `SubagentStart` | subagent-start |

詳細なスキーマ対応は `docs/reference/hooks-schema-mapping.md` を参照。

## VSCode GitHub Copilot

### 読み込みパスと優先順位

公式ドキュメント: [Use custom instructions in VS Code](https://code.visualstudio.com/docs/copilot/customization/custom-instructions)

| スコープ | パス | 自動読み込み | 備考 |
|---------|------|--------------|------|
| Project | `.github/copilot-instructions.md` | ✅ | メインの指示ファイル |
| Project | `.github/instructions/*.instructions.md` | ❌ | 手動指定が必要 |
| User | VSCode設定の `file` プロパティ | ✅ | 設定で外部ファイル参照 |
| Prompts | `.github/prompts/*.prompt.md` | ❌ | 手動呼び出し |

### 重要な制約

- **Copilotはグローバルファイル（`~/.copilot/`等）を直接読み込まない**（Instructions / Prompts の話。Skills は上流で `~/.copilot/skills/` が追加済み → [#457](https://github.com/DIO0550/plugin-manager/issues/457)）
- Personal スコープは VSCode 設定経由で外部ファイルを参照する形式
- Issue: [Global files outside workspace の要望](https://github.com/microsoft/vscode-copilot-release/issues/3129)

### VSCode設定での外部ファイル参照

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

### コンポーネント配置場所

| 種別 | ファイル形式 | Personal | Project |
|------|-------------|----------|---------|
| Skills | `SKILL.md` | -（TODO: [#457](https://github.com/DIO0550/plugin-manager/issues/457)） | `.github/skills/<marketplace>/<plugin>/<skill>/` |
| Agents | `*.agent.md` | `~/.copilot/agents/<marketplace>/<plugin>/` | `.github/agents/<marketplace>/<plugin>/` |
| Prompts | `*.prompt.md` | - | `.github/prompts/<marketplace>/<plugin>/` |
| Instructions | `AGENTS.md` | - | `AGENTS.md` |
| Instructions | `copilot-instructions.md` | - | `.github/copilot-instructions.md` |
| Hooks | `*.json` | `~/.copilot/hooks/<marketplace>/<plugin>/` | `.github/hooks/<marketplace>/<plugin>/` |

### Hooks（Preview）

VSCode Copilot Agent Modeでは、エージェントセッションのライフサイクルイベントに対してシェルコマンドを実行するHooksをサポートしています（Preview機能）。

公式ドキュメント: [Agent hooks in Visual Studio Code](https://code.visualstudio.com/docs/copilot/customization/hooks)

#### イベント種別

| イベント | タイミング | 用途 |
|---------|-----------|------|
| `PreToolUse` | ツール実行前 | 危険操作のブロック、承認要求 |
| `PostToolUse` | ツール実行後 | フォーマッタ実行、ログ記録 |
| `SessionStart` | セッション開始時 | リソース初期化、状態検証 |
| `Stop` | セッション終了時 | レポート生成、後片付け |
| `UserPromptSubmit` | プロンプト送信時 | 監査、コンテキスト注入 |
| `PreCompact` | コンテキスト圧縮前 | 重要コンテキストの退避 |
| `SubagentStart` | サブエージェント開始時 | 追跡 |
| `SubagentStop` | サブエージェント終了時 | クリーンアップ |

> **対応済み（[#458](https://github.com/DIO0550/plugin-manager/issues/458)）**: `PostToolUseFailure` / `PreCompact` / `SubagentStart` を
> Copilot CLI の `postToolUseFailure` / `preCompact` / `subagentStart` へ変換する。
> Personal スコープの Skills パス `~/.copilot/skills/` も上流で追加済み（[#457](https://github.com/DIO0550/plugin-manager/issues/457)）。

#### 設定形式

```json
{
  "hooks": {
    "PreToolUse": [
      {
        "type": "command",
        "command": "./scripts/validate.sh",
        "timeout": 15
      }
    ],
    "PostToolUse": [
      {
        "type": "command",
        "command": "npx prettier --write \"$TOOL_INPUT_FILE_PATH\""
      }
    ]
  }
}
```

#### Copilot CLI / Coding Agent との互換性

GitHub Copilot CLI（camelCase形式）の hooks 設定も VSCode で利用可能です。VSCode は camelCase → PascalCase の自動変換を行います。

| 項目 | VSCode | Copilot CLI |
|------|--------|------------|
| イベント名 | PascalCase (`PreToolUse`) | camelCase (`preToolUse`) |
| version フィールド | 不要 | `"version": 1` 必須 |
| コマンド指定 | `command`, `windows`, `linux`, `osx` | `bash`, `powershell` |
| タイムアウト | `timeout` | `timeoutSec` |

#### I/O プロトコル

Hooks は stdin で JSON を受け取り、stdout で JSON を返します。

```json
// 出力例（PreToolUse）
{
  "continue": true,
  "hookSpecificOutput": {
    "hookEventName": "PreToolUse",
    "permissionDecision": "allow",
    "permissionDecisionReason": "Validated tool input"
  }
}
```

終了コード: `0` = 成功、`2` = ブロッキングエラー、その他 = 非ブロッキング警告。

## Google Antigravity

> **実装状況**: Skills / Hooks 配置は実装済み。Agents / Commands（Workflows）/ Instructions（Rules・context files）は公式サポート確認済みだが PLM 未実装（調査 [#400](https://github.com/DIO0550/plugin-manager/issues/400)）。

### 概要

Google Antigravityはエージェント指向の開発プラットフォーム（IDE / CLI / AGY）。Anthropic 発の Agent Skills open standard（`SKILL.md`）に加え、Custom Subagents（`agent.md`）、Workflows（スラッシュコマンド）、Rules / `GEMINI.md` / `AGENTS.md`、Hooks をファイルベースでサポートする。

公式ドキュメント:
- [Skills](https://antigravity.google/docs/skills)
- [Rules & Workflows](https://antigravity.google/docs/rules-workflows)
- [Subagents](https://antigravity.google/docs/subagents)
- [Hooks](https://antigravity.google/docs/hooks)
- [CLI Best Practices（GEMINI.md / AGENTS.md）](https://antigravity.google/docs/cli/best-practices)
- [Getting Started with Google Antigravity](https://codelabs.developers.google.com/getting-started-google-antigravity)
- [Authoring Google Antigravity Skills](https://codelabs.developers.google.com/getting-started-with-antigravity-skills)

### 読み込みパスと優先順位（公式）

| 種別 | スコープ | パス | 自動読み込み | 備考 |
|------|---------|------|--------------|------|
| Skills | Global（推奨） | `~/.gemini/config/skills/` | ✅ | AGY / IDE / CLI 共通で認識 |
| Skills | Global（IDE 互換） | `~/.gemini/antigravity/skills/` | ✅* | *AGY CLI では非認識。PLM 現状パス |
| Skills | Workspace | `.agents/skills/` | ✅ | `.agent/skills` は後方互換。PLM 現状は `.agent/skills/` |
| Agents | Global | `~/.gemini/config/agents/<name>/agent.md` | ✅ | Custom Subagents |
| Agents | Workspace | `.agents/agents/<name>/agent.md` | ✅ | または `.agents/agents/<name>.md` |
| Workflows（Commands 相当） | Global | `~/.gemini/config/global_workflows/<name>.md` | ✅ | `/name` で呼び出し。公式 docs 未記載パスは実機検証で確定 |
| Workflows | Workspace | `.agents/workflows/<name>.md` | ✅ | 同上 |
| Rules（Instructions） | Global | `~/.gemini/GEMINI.md` | ✅ | 単一ファイル |
| Rules | Workspace | `.agents/rules/*.md` | ✅ | Manual / Always On / Model Decision / Glob |
| Context（Instructions） | Workspace root | `GEMINI.md` / `AGENTS.md` | ✅ | セッション開始時に読み込み |

### 重要な特徴

- **ディレクトリベースのSkillsパッケージ**: 各Skillは独立したディレクトリとして管理
- **Progressive Disclosure**: Skillは必要時のみコンテキストにロードされる（コンテキスト肥大化を防止）
- **SKILL.md形式**: Anthropic発祥のAgent Skills open standardを採用
- **Custom Subagents**: YAML frontmatter 付き `agent.md`（`name` / `description` / `tools` / `model` 等）
- **Workflows**: 保存済みプロンプト列を `/workflow-name` で実行（PLM Commands に相当）
- **Rules + AGENTS.md**: 永続指示。Changelog で `AGENTS.md` 読み込みが追加済み

### コンポーネント配置場所

#### PLM 実装済み（Skills / Hooks）

| 種別 | ファイル形式 | Personal | Project |
|------|-------------|----------|---------|
| Skills | `SKILL.md` | `~/.gemini/antigravity/skills/<marketplace>/<plugin>/<skill>/` | `.agent/skills/<marketplace>/<plugin>/<skill>/` |
| Hooks | `hooks.json` | `~/.gemini/config/hooks.json` | `.agents/hooks.json` |

> Skills の公式推奨パスは Personal `~/.gemini/config/skills/`、Project `.agents/skills/`。PLM 現状パスは IDE 互換だが CLI 非対応のため、パス移行は別 feature（[#460](https://github.com/DIO0550/plugin-manager/issues/460)、[#402](https://github.com/DIO0550/plugin-manager/issues/402) 連携）で扱う。
> **TODO（2026-08-20 調査）**: 上流は `.agents/skills` を既定とし `.agent/skills` は後方互換扱いになった（[#460](https://github.com/DIO0550/plugin-manager/issues/460)）。
>
> Hooks は Claude Code 形式から命名フックマップへ変換して単一 `hooks.json` に配置する（[#309](https://github.com/DIO0550/plugin-manager/issues/309)）。非管理ファイルの上書きと複数 Hook 同時配置は拒否。スキーマ詳細は [hooks-schema-mapping.md](../reference/hooks-schema-mapping.md)。

#### 公式サポートあり・PLM 未実装（#400）

| 種別 | ファイル形式 | Personal（想定） | Project（想定） |
|------|-------------|------------------|-----------------|
| Agents | `agent.md` | `~/.gemini/config/agents/<name>/agent.md` | `.agents/agents/<name>/agent.md` |
| Commands（Workflows） | `*.md` | `~/.gemini/config/global_workflows/<name>.md` | `.agents/workflows/<name>.md` |
| Instructions | `GEMINI.md` / `AGENTS.md` | `~/.gemini/GEMINI.md` | ルート `AGENTS.md`（または `GEMINI.md`）。複数ルールは `.agents/rules/` |

配置パス・形式の詳細は上記表と [#400](https://github.com/DIO0550/plugin-manager/issues/400) を参照。

### 制約事項

- **PLM は現状 Skills + Hooks**: Agents / Commands / Instructions は公式サポート確認済みだが未実装
- **Hooks は単一ファイル**: Personal / Project とも 1 つの `hooks.json`。フルマージ未実装のため複数 Hook 同時配置は拒否
- **Agent 形式は Claude Code 非互換**: `*.agent.md` ではなく `agent.md` + Antigravity 固有 frontmatter
- **Workflows パスは公式 docs 未記載**: 実装前に IDE 実機で再確認すること
- **Personal Instruction は Gemini CLI と `~/.gemini/GEMINI.md` を共有**しうる
- Skillsはタスク終了後にコンテキストから解放される（エフェメラル）

## Gemini CLI

### 概要

Gemini CLIはGoogleのターミナルベースAIエージェントツール。v0.23.0（2026年1月7日）でAgent Skills（実験的機能）が追加された。Claude Code Skillsと同じ`SKILL.md`形式を採用しており、既存のSkillsをそのまま再利用可能。

公式ドキュメント:
- [Agent Skills | Gemini CLI](https://geminicli.com/docs/cli/skills/)
- [Getting Started with Agent Skills](https://geminicli.com/docs/cli/tutorials/skills-getting-started/)

### 読み込みパスと優先順位

| スコープ | パス | 自動読み込み | 備考 |
|---------|------|--------------|------|
| Workspace | `.gemini/skills/` | ✅ | プロジェクト固有、VCS管理推奨 |
| User | `~/.gemini/skills/` | ✅ | 個人用、全ワークスペースで利用可能 |
| Extension | 拡張機能に同梱 | ✅ | 拡張機能パッケージ内 |
| Instructions (Global) | `~/.gemini/GEMINI.md` | ✅ | 全プロジェクト共通の指示 |
| Instructions (Project) | `./GEMINI.md` | ✅ | 親ディレクトリまで走査 |

### 優先順位

同名Skillが複数スコープに存在する場合: Workspace > User > Extension

### Skills のアクティベーション

Gemini CLI SkillsはProgressive Disclosure方式を採用:

1. **Discovery**: セッション開始時にSkillの名前と説明のみをシステムプロンプトに注入
2. **Activation**: タスクにマッチするSkillを検出すると `activate_skill` ツールを呼び出す
3. **Consent**: ユーザーにSkill名・目的・ディレクトリパスを表示して確認を求める
4. **Injection**: `SKILL.md` の本文とフォルダ構造を会話に追加
5. **Execution**: 専門知識がアクティブな状態でタスクを実行

### 管理コマンド

**セッション内** (`/skills`):
- `/skills list` - 発見されたSkill一覧
- `/skills disable <name>` - Skillを無効化
- `/skills enable <name>` - Skillを再有効化
- `/skills reload` - Skill検出を再実行

**ターミナル** (`gemini skills`):
- `gemini skills list` - 全Skill表示
- `gemini skills install <source>` - Skill追加（Gitリポジトリ、ローカルパス、`.skill`ファイル対応）
- `gemini skills uninstall <name>` - Skill削除
- `gemini skills enable/disable <name>` - 有効/無効切替

### Instructions システム（GEMINI.md）

Gemini CLIは `GEMINI.md` ファイルによる階層的な指示システムを持つ:

- **Global**: `~/.gemini/GEMINI.md` - 全プロジェクト共通の指示
- **Project**: カレントディレクトリからプロジェクトルート（`.git`フォルダ）まで走査し、各ディレクトリの `GEMINI.md` を連結
- **ファイル名設定**: `.gemini/settings.json` で `contextFileName` を変更可能（例: `"contextFileName": "AGENTS.md"`）
- **モジュラーインポート**: `@file.md` 構文で他ファイルの内容をインポート可能

### コンポーネント配置場所

| 種別 | ファイル形式 | Personal | Project |
|------|-------------|----------|---------|
| Skills | `SKILL.md` | `~/.gemini/skills/<marketplace>/<plugin>/<skill>/` | `.gemini/skills/<marketplace>/<plugin>/<skill>/` |
| Instructions | `GEMINI.md` | `~/.gemini/GEMINI.md` | `GEMINI.md` |

### 制約事項

- ~~**実験的機能**: `/settings` で Agent Skills を `true` に設定して有効化が必要~~
  → **TODO（2026-08-20 調査）**: 上流で GA 済み。`.agents/skills` エイリアスや管理コマンドの追加も含めて記載を更新する（[#461](https://github.com/DIO0550/plugin-manager/issues/461)）
- **Agents非対応**: `.agent.md` 形式はサポートしない
- **Prompts非対応**: `.prompt.md` 形式はサポートしない
- **Hooks 非対応**: Gemini CLI 単体の hooks 公式仕様は追わず、Antigravity（IDE / CLI）共通仕様へ一本化する。一般向け製品移行の詳細は [Transitioning Gemini CLI to Antigravity CLI](https://github.com/google-gemini/gemini-cli/discussions/27274) を参照

## Cursor

> **実装状況**: Skills / Agents / Commands / Instructions / Hooks 配置は実装済み（Epic [#356](https://github.com/DIO0550/plugin-manager/issues/356)）。

### 概要

CursorはAnysphere社のAIコードエディタ。エディタに加えてターミナルから使えるCursor CLI（`cursor-agent`）を持つ。Cursor 2.4でAgent Skills（Anthropic発のopen standard、`SKILL.md`形式）をエディタ・CLIの両方でサポートした。サブエージェント、カスタムスラッシュコマンド、`AGENTS.md`、Hooksもサポートする。

公式ドキュメント:
- [Agent Skills | Cursor Docs](https://cursor.com/docs/context/skills)
- [Subagents | Cursor Docs](https://cursor.com/docs/agent/subagents)
- [Rules / AGENTS.md | Cursor Docs](https://cursor.com/docs/context/rules)
- [Hooks | Cursor Docs](https://cursor.com/docs/agent/hooks)
- [Cursor 2.4 Changelog（Subagents / Skills）](https://cursor.com/changelog/2-4)

### 読み込みパスと優先順位

| 種別 | スコープ | パス | 自動読み込み | 備考 |
|------|---------|------|--------------|------|
| Skills | User | `~/.cursor/skills/`, `~/.agents/skills/` | ✅ | 互換パスとして `~/.claude/skills/`, `~/.codex/skills/` も読む |
| Skills | Project | `.cursor/skills/`, `.agents/skills/` | ✅ | 互換パスとして `.claude/skills/`, `.codex/skills/` も読む。skillsルートを**再帰走査**して `SKILL.md` を発見 |
| Agents | User | `~/.cursor/agents/` | ✅ | 互換: `~/.claude/agents/`, `~/.codex/agents/`（同名時は `.cursor/` 優先） |
| Agents | Project | `.cursor/agents/` | ✅ | 互換: `.claude/agents/`, `.codex/agents/` |
| Commands | User | `~/.cursor/commands/` | ✅ | プレーンMarkdown |
| Commands | Project | `.cursor/commands/` | ✅ | `/` 入力で一覧表示 |
| Rules | Project | `.cursor/rules/*.mdc` | ✅ | frontmatter（`alwaysApply` / `description` / `globs`）付き |
| Instructions | Project | `AGENTS.md` | ✅ | プロジェクトルート＋サブディレクトリのネスト対応（深い階層が優先） |
| Hooks | User | `~/.cursor/hooks.json` | ✅ | 単一ファイル |
| Hooks | Project | `.cursor/hooks.json` | ✅ | 単一ファイル |

### 重要な特徴

- **SKILL.md形式**: frontmatterは `name`（必須、小文字・数字・ハイフンのみ、フォルダ名と一致）と `description`（必須）に加え、`paths`（globで適用範囲を制限）、`disable-model-invocation`（trueで明示的スラッシュコマンド専用化）、`metadata` をサポート
  - **TODO（2026-08-20 調査）**: 上流に `icon` / `color`（Custom Mode 表示用）が追加済み（[#459](https://github.com/DIO0550/plugin-manager/issues/459)）
- **skillsルートの再帰走査**: ネストしたディレクトリ内の `SKILL.md` も発見されるため、PLMの `<marketplace>/<plugin>/<skill>/` 階層はそのまま読み込まれる見込み
- **Agents（サブエージェント）**: YAMLフロントマター（`name`, `description`, `model`, `readonly`, `is_background`）付きMarkdown。エディタ・CLI・Cloud Agentsで利用可能
- **CommandsはSkillsへ移行中**: `/migrate-to-skills` により既存のCommandsは `disable-model-invocation: true` 付きSkillsへ変換される方向。`.cursor/commands/` 自体は引き続き動作する
- **Claude Code互換**: `.claude/skills/` / `.claude/agents/` を互換パスとして読むため、コンポーネントのフォーマット変換はほぼ不要（`CommandFormat::ClaudeCode` / `AgentFormat::ClaudeCode`）

### Hooks

設定は単一の `hooks.json`（`{"version": 1, "hooks": {"<event>": [{"command": "..."}]}}`）。イベント名は**camelCase**で、Copilot CLI形式（camelCase + `"version": 1`）に近い。

主なイベント: `sessionStart`, `sessionEnd`, `preToolUse`, `postToolUse`, `postToolUseFailure`, `subagentStart`, `subagentStop`, `beforeShellExecution`, `afterShellExecution`, `beforeMCPExecution`, `afterMCPExecution`, `beforeReadFile`, `afterFileEdit`, `beforeSubmitPrompt`, `preCompact`, `stop`, `afterAgentResponse` など。

> **TODO（2026-08-20 調査）**: 上流に `afterAgentThought`・Tab hooks（`beforeTabFileRead` / `afterTabFileEdit`）・`workspaceOpen` が追加され、
> hook 単位のフィールドにも `type`（`command` / `prompt`）・`loop_limit`・`failClosed`・`matcher` が加わった。
> 設定スコープの優先順位も Enterprise → Team → Project → User へ拡張されている（[#459](https://github.com/DIO0550/plugin-manager/issues/459)）。

Claude Code側に対応イベントがないもの（`beforeShellExecution` 等のCursor固有イベント）は変換対象外。PLM は Claude Code → Cursor 変換（camelCase + `version: 1`）を行い、単一の `hooks.json` として配置する。既存の非管理 `hooks.json` の上書きと、同一インストール内の複数 Hook コンポーネントは拒否する（フルマージは将来対応）。

### コンポーネント配置場所（PLM 実装）

Agents / Commands / Hooks は他ターゲットと同様に `flatten_name(plugin, original)` により `{plugin}_{original}` へ平坦化して配置する。
**Skills のみ** frontmatter `name` と親フォルダ名の一致要件に合わせ、元のスキル名（`original_name`）で配置する（#377）。
同名スキルの衝突時はエラー。旧 `{plugin}_{skill}` ディレクトリは install / uninstall 時にフォールバック削除する。

| 種別 | ファイル形式 | Personal | Project |
|------|-------------|----------|---------|
| Skills | `SKILL.md` | `~/.cursor/skills/<original_name>/` | `.cursor/skills/<original_name>/` |
| Agents | `<flattened_name>.md` | `~/.cursor/agents/<flattened_name>.md` | `.cursor/agents/<flattened_name>.md` |
| Commands | `<flattened_name>.md` | `~/.cursor/commands/<flattened_name>.md` | `.cursor/commands/<flattened_name>.md` |
| Instructions | `AGENTS.md` | - | `AGENTS.md` |
| Hooks | `hooks.json`（単一ファイル） | `~/.cursor/hooks.json` | `.cursor/hooks.json` |

### 制約事項

- **Instructions は Project スコープのみ**: Personalスコープの指示（User Rules）はアプリ設定画面で管理され、ファイルベースのグローバルパスがない（Copilotと同型の制約）
- **`AGENTS.md` は Codex ターゲットと同一ファイルを共有**: 両ターゲット有効時は同一パスを参照する
- **Hooks は単一設定ファイル**: ディレクトリ配置ではなく `hooks.json` へ書き込む。フルマージ未実装のため、非管理ファイルの上書きと複数 Hook の同時配置は拒否する
- **Skills は元名配置**: プラグイン接頭辞が無いため、同名スキルを持つ別プラグインとの衝突時はエラーになる
- **sync と Cursor Skills**: Cursor の Skill 名キーは元名、他ターゲットはフラット化名のため、現状 Cursor を含む Skill sync は名前不一致になりうる（既知制限・追跡: [#384](https://github.com/DIO0550/plugin-manager/issues/384)）
- **Cursor 固有ロジックの集約**: install / import / intent への `TargetKind::Cursor` 分岐散在は [#385](https://github.com/DIO0550/plugin-manager/issues/385) で Target trait フックへ寄せる予定

### 検証結果

- **Agents のファイル名**: Cursor は `<name>.md` を期待する。PLM の `.agent.md` サフィックスは Cursor では認識されないため、配置時はプレーン `.md` にリネームする（`AgentFormat::ClaudeCode` → `AgentFormat::ClaudeCode` のコピーで内容変換は不要）
- **Agents / Commands のディレクトリ階層**: Cursor 公式ドキュメントに再帰走査の明記がないため、Skills と同様のフラット配置（`agents/<flattened_name>.md` / `commands/<flattened_name>.md`）を採用
- **Commands の配置先**: `/migrate-to-skills` による Skills 移行が進行中だが、`.cursor/commands/` は引き続き動作するため、当面は Commands として配置する
- **Hooks 変換**: Claude Code 形式から Cursor 形式（`version: 1` + camelCase イベント、`command` / `timeout` フィールド）へ変換して配置する

### Cursor CLI Hooks の実機検証記録

検証日: **2026-08-25** / Cursor CLI: **`cursor-agent 2026.08.22`** / 実行環境:
Linux x86_64 のターミナルから起動した Cursor CLI（エディタ UI、Tab 補完、Cloud Agent は使用しない）。

#### 再現手順

1. `cursor-agent --version` で上記バージョンを確認する。
2. Project スコープの `.cursor/hooks.json` に、対象イベントごとに受け取った stdin を別々の
   JSON Lines ファイルへ追記する command hook を設定する。
3. `cursor-agent` を対象 workspace で起動し、プロンプト送信、ファイル読み取り・編集、shell・MCP
   ツール実行、subagent 起動、compact、通常停止を順に行う。
4. CLI 終了後に各ログの有無と `hook_event_name` を確認する。イベントを一つずつ設定した場合と、
   全イベントを同時に設定した場合の両方で同じ結果になることを確認する。

ここで「発火」は hook command が実行され、対応するログが記録されたことを表す。「発火せず」は
上記 CLI 操作を行ってもログが作られなかったことを表し、Cursor エディタでも発火しないという意味ではない。

#### Agent イベント

実行環境はいずれも **Cursor CLI**。次のイベントは、括弧内の操作により CLI で発火確認が可能だった。

| イベント | CLI での結果 | 確認操作 |
|---------|--------------|----------|
| `sessionStart` / `sessionEnd` | 発火 | セッションの開始 / 正常終了 |
| `beforeSubmitPrompt` | 発火 | プロンプト送信 |
| `preToolUse` / `postToolUse` / `postToolUseFailure` | 発火 | 成功するツールと失敗するツールの実行 |
| `beforeShellExecution` / `afterShellExecution` | 発火 | shell command の実行 |
| `beforeMCPExecution` / `afterMCPExecution` | 発火 | MCP tool の実行 |
| `beforeReadFile` / `afterFileEdit` | 発火 | Agent によるファイルの読み取り / 編集 |
| `subagentStart` / `subagentStop` | 発火 | subagent の起動 / 完了 |
| `preCompact` | 発火 | `/compact` の実行 |
| `stop` / `afterAgentResponse` | 発火 | Agent 応答の完了 |
| `afterAgentThought` | **発火** | ファイル調査を伴う Agent タスクの実行 |

`afterAgentThought` は Cursor CLI で実機確認済み。ただし Claude Code に対応イベントが存在しないため、
Claude Code 形式を入力とする PLM の Cursor Hooks 変換対象には追加しない。

#### Tab イベント

| イベント | 結果 | 検証対象の実行環境 |
|---------|------|--------------------|
| `beforeTabFileRead` | **CLI では検証不能** | Cursor Tab を持たない Cursor CLI |
| `afterTabFileEdit` | **CLI では検証不能** | Cursor Tab を持たない Cursor CLI |

両イベントは Cursor 公式仕様で存在を確認済みだが、Tab 補完を提供する **Cursor エディタでは実機未検証**。
したがって「公式仕様確認済み・実機未検証」とし、CLI で発火しなかったイベントとしては扱わない。
また、いずれも Claude Code に対応イベントが存在しないため、PLM の変換対象には影響しない。

#### App ライフサイクルイベント

| イベント | 結果 | 検証対象の実行環境 |
|---------|------|--------------------|
| `workspaceOpen` | **CLI では検証不能** | App の workspace ライフサイクルを持たない Cursor CLI |

`workspaceOpen` は Cursor 公式仕様で存在を確認済みだが、**Cursor エディタでは実機未検証**。
「公式仕様確認済み・実機未検証」として残す。Claude Code に対応イベントが存在しないため、
この結果も PLM の変換対象には影響しない。

## OpenCode

> **実装状況**: Skills / Agents / Commands / Instructions 配置は実装済み（Epic [#416](https://github.com/DIO0550/plugin-manager/issues/416)）。Hooks / Plugins（JS/TS）は対象外。計画: [`docs/architecture/opencode-target-plan.md`](../architecture/opencode-target-plan.md)。

### 概要

OpenCode はターミナルベースの AI コーディングエージェント。Agent Skills open standard（`SKILL.md`）、Markdown エージェント、カスタムスラッシュコマンド、`AGENTS.md` をファイルベースでサポートする。Claude Code 互換パス（`.claude/` / `~/.claude/`）も読み込むが、PLM の配置先は OpenCode ネイティブパス（`.opencode/` / `~/.config/opencode/`）を正とする。

公式ドキュメント:
- [Agent Skills | OpenCode](https://opencode.ai/docs/skills/)
- [Agents | OpenCode](https://opencode.ai/docs/agents/)
- [Commands | OpenCode](https://opencode.ai/docs/commands/)
- [Rules / AGENTS.md | OpenCode](https://opencode.ai/docs/rules/)
- [Config | OpenCode](https://opencode.ai/docs/config/)
- [Plugins | OpenCode](https://opencode.ai/docs/plugins/)

### 読み込みパスと優先順位

| 種別 | スコープ | パス | 自動読み込み | 備考 |
|------|---------|------|--------------|------|
| Skills | Global | `~/.config/opencode/skills/<name>/SKILL.md` | ✅ | XDG: `$XDG_CONFIG_HOME/opencode/skills/`（未設定時 `~/.config`） |
| Skills | Global（互換） | `~/.claude/skills/`、`~/.agents/skills/` | ✅ | PLM 配置先には使わない |
| Skills | Project | `.opencode/skills/<name>/SKILL.md` | ✅ | cwd から git worktree まで上方走査 |
| Skills | Project（互換） | `.claude/skills/`、`.agents/skills/` | ✅ | PLM 配置先には使わない |
| Agents | Global | `~/.config/opencode/agents/<name>.md` | ✅ | ファイル名がエージェント名 |
| Agents | Project | `.opencode/agents/<name>.md` | ✅ | 同上 |
| Commands | Global | `~/.config/opencode/commands/<name>.md` | ✅ | ネスト可（`team/review.md` → `/team/review`） |
| Commands | Project | `.opencode/commands/<name>.md` | ✅ | 同上 |
| Instructions | Global | `~/.config/opencode/AGENTS.md` | ✅ | Personal 対応（Cursor との差分） |
| Instructions | Project | `AGENTS.md`（cwd〜プロジェクトルート） | ✅ | `CLAUDE.md` は AGENTS.md 無し時のフォールバック |
| Plugins（Hooks 相当） | Global / Project | `~/.config/opencode/plugins/`、`.opencode/plugins/` | ✅ | **JS/TS モジュール**。PLM Hook 対象外 |

### 重要な特徴

- **SKILL.md 形式**: frontmatter は `name`（必須）と `description`（必須）。`license` / `compatibility` / `metadata` は任意。未知フィールドは無視
- **`name` 制約**: 1–64 文字、`^[a-z0-9]+(-[a-z0-9]+)*$`、**親ディレクトリ名と一致必須**（Cursor Skills と同型）
- **Skills 発見は 1 階層**: 公式は `skills/*/SKILL.md`。`<marketplace>/<plugin>/<skill>/` の深いネストは発見されないため、PLM は **`original_name` フラット配置**（Cursor #377 と同型）
- **Agents**: YAML frontmatter（`description`, `mode`, `model`, `permission` 等）付き Markdown。`mode` は `primary` / `subagent` / `all`
- **Commands**: 本文がプロンプトテンプレート。`$ARGUMENTS` / `$1` と `` !`shell` `` / `@file` をサポート
- **Instructions**: Personal + Project の両方。Project `AGENTS.md` は Codex / Cursor と同一パスを共有しうる
- **Claude Code 互換読み込み**: OpenCode 自体は `.claude/` も読むが、PLM はネイティブパスへ配置して責務を明確化する

### Hooks / Plugins（対象外）

OpenCode の拡張点は Claude Code / Cursor 系の JSON Hooks ではなく、**TypeScript/JavaScript Plugin**（`tool.execute.before` 等のフック関数を export）。

| 項目 | Claude Code / Cursor Hooks | OpenCode Plugins |
|------|---------------------------|------------------|
| 形式 | JSON（`hooks.json`） | `.ts` / `.js` モジュール |
| 配置 | 単一 JSON or ディレクトリ | `plugins/` ディレクトリ |
| 実行 | シェルコマンド | Bun 上の JS 関数 |

PLM の `ComponentKind::Hook` 変換パイプラインとはモデルが根本的に異なるため、**OpenCode 対応では Hooks をサポートしない**（`supported_components` に含めない）。将来の Plugin 配置・生成は別 Epic とする。

### コンポーネント配置場所（PLM 実装）

Skills は frontmatter `name` と親フォルダ名の一致要件に合わせ、**元のスキル名（`original_name`）**で配置する（Cursor #377 と同型）。同名スキルの衝突時はエラー。

Agents / Commands は他ターゲットと同様に `flatten_name(plugin, original)` により `{plugin}_{original}` へ平坦化し、プレーン `.md` として配置する。

| 種別 | ファイル形式 | Personal | Project |
|------|-------------|----------|---------|
| Skills | `SKILL.md` | `~/.config/opencode/skills/<original_name>/` | `.opencode/skills/<original_name>/` |
| Agents | `<flattened_name>.md` | `~/.config/opencode/agents/<flattened_name>.md` | `.opencode/agents/<flattened_name>.md` |
| Commands | `<flattened_name>.md` | `~/.config/opencode/commands/<flattened_name>.md` | `.opencode/commands/<flattened_name>.md` |
| Instructions | `AGENTS.md` | `~/.config/opencode/AGENTS.md` | `AGENTS.md` |
| Hooks | — | ❌ 対象外 | ❌ 対象外 |

### フォーマット変換方針

| コンポーネント | 変換 | 備考 |
|----------------|------|------|
| Skills | 不要 | open standard のまま。未知 frontmatter は OpenCode が無視 |
| Agents | 拡張子のみ（`.agent.md` → `.md`） | 内容は Claude Code 互換をコピー。OpenCode 固有 `mode` / `permission` は v1 で自動付与しない |
| Commands | 拡張子のみ（`.prompt.md` → `.md`） | `$ARGUMENTS` / `$1` は OpenCode ネイティブ互換。Claude 固有 frontmatter は残っても実害は小さい想定 |
| Instructions | 不要 | `AGENTS.md` 共通 |
| Hooks | — | 対象外 |

### 制約事項

- **Skills は元名配置**: プラグイン接頭辞が無いため、同名スキルを持つ別プラグインとの衝突時はエラー（Cursor と同型）
- **Skills は 1 階層のみ発見**: 深いネスト配置は OpenCode に読まれない
- **Personal Instruction あり**: Cursor と異なり `~/.config/opencode/AGENTS.md` をサポートする
- **Project `AGENTS.md` は Codex / Cursor と共有**: 複数ターゲット有効時は同一パスを参照する
- **Hooks 非対応**: JS/TS Plugin モデルのため別設計が必要
- **XDG**: Personal ルートは `$XDG_CONFIG_HOME/opencode`（デフォルト `~/.config/opencode`）。環境変数を尊重する
- **sync と OpenCode Skills**: Cursor と同様、Skill 名キーが元名のため他ターゲット（フラット化名）との sync 名不一致に注意（既知パターン: [#384](https://github.com/DIO0550/plugin-manager/issues/384)）
- **opt-in**: デフォルト有効ターゲットには含めない。`plm target add opencode` で有効化する

### 検証結果（仕様調査）

- Skills / Agents / Commands / AGENTS.md のパスは公式 docs（opencode.ai）で確認済み（2026-08）
- Global Skills は `~/.config/opencode/skills/` であり、`~/.opencode/skills/` ではない（第三者記事の誤りに注意）
- Plugins は Hook 相当だが JSON 変換対象外と決定

### 未対応・将来検討

- Commands のネスト配置（`commands/team/review.md`）の保持（v1 はフラット配置のみ）
- `OPENCODE_CONFIG_DIR` 指定時の追加探索パス（v1 では標準パスのみ）
- OpenCode Plugins（JS/TS）の生成・配置

## PLMでの対応方針

| ターゲット | Personal インストール | 追加アクション |
|-----------|----------------------|----------------|
| Codex | `~/.codex/` に配置 | Hook 配置時のみ `~/.codex/config.toml` に `[features] codex_hooks = true` を自動追記（`--no-enable-flag` で抑止可、`codex_hooks = false` 既設定時は警告のみでスキップ） |
| Copilot | ファイル配置 + VSCode設定追記 | `settings.json` への参照追加が必要 |
| Antigravity | Skills: `~/.gemini/antigravity/`。Hooks: `~/.gemini/config/hooks.json`（実装済み・[#309](https://github.com/DIO0550/plugin-manager/issues/309)）。Agents / Workflows / Instructions は未実装（[#400](https://github.com/DIO0550/plugin-manager/issues/400)） | Skills / Hooks は自動読み込み。Hooks は単一 `hooks.json`（上書きガードあり） |
| Gemini CLI | `~/.gemini/skills/` に配置 | 不要（自動読み込み、要Settings有効化） |
| Cursor | `~/.cursor/` に配置（Skills / Agents / Commands / Hooks） | 不要（自動読み込み）。Hooksは単一 `hooks.json` へ変換配置（上書きガードあり） |
| OpenCode | `~/.config/opencode/` に配置（Skills / Agents / Commands / Instructions） | 不要（自動読み込み）。Hooks/Plugins は対象外 |

## 将来の拡張候補

- Claude Code（計画中）
- OpenCode Plugins（JS/TS）対応（Hooks 相当の別モデル）
- Windsurf
- Aider
- その他SKILL.md対応ツール

## 関連

- [concepts/components](./components.md) - コンポーネント種別
- [concepts/scopes](./scopes.md) - Personal/Projectスコープ
- [commands/target](../commands/target.md) - ターゲット管理コマンド

# ターゲット環境

PLMがサポートするAI開発環境（ターゲット）について説明します。

## 対応ターゲット

| ターゲット | 説明 | 状態 |
|------------|------|------|
| **codex** | OpenAI Codex CLI | ✅ 対応済み |
| **copilot** | VSCode GitHub Copilot | ✅ 対応済み |
| **antigravity** | Google Antigravity IDE | ✅ 対応済み |
| **gemini** | Gemini CLI（ターミナルベースAIエージェント） | ✅ 対応済み |
| **cursor** | Cursor（IDE / CLI） | ✅ 対応済み |

## サポートするコンポーネント

| コンポーネント | Codex | Copilot | Antigravity | Gemini CLI | Cursor |
|----------------|-------|---------|-------------|------------|--------|
| Skills | ✅ | ✅ | ✅ | ✅ | ✅ |
| Agents | ✅ | ✅ | ❌ | ❌ | ✅ |
| Commands | ❌ | ✅ | ❌ | ❌ | ✅ |
| Instructions | ✅ | ✅ | ❌* | ✅** | ✅*** |
| Hooks | ✅ | ✅ | ❌ | ❌ | ✅**** |

> *AntigravityはSkills専用の設計で、Instructionsは別途設定で管理します。
> **Gemini CLIは`GEMINI.md`による階層的な指示システムを持ちます。
> ***CursorのInstructionsはProjectスコープ（`AGENTS.md`）のみ。Personalスコープの指示（User Rules）はアプリ設定画面で管理されるため対象外。
> ****CursorのHooksは単一の `hooks.json` に配置する。既存の非管理ファイルの上書きと、同一インストール内の複数 Hook コンポーネントは拒否する（フルマージは未実装）。

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

公式ドキュメント: [Codex Hooks](https://developers.openai.com/codex/hooks)

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

- **Copilotはグローバルファイル（`~/.copilot/`等）を直接読み込まない**
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
| Skills | `SKILL.md` | - | `.github/skills/<marketplace>/<plugin>/<skill>/` |
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

### 概要

Google AntigravityはGemini 3 Pro搭載のエージェント指向IDE。2026年1月13日にAnthropicのAgent Skills open standard（SKILL.md形式）を正式採用。

公式ドキュメント:
- [Getting Started with Google Antigravity](https://codelabs.developers.google.com/getting-started-google-antigravity)
- [Authoring Google Antigravity Skills](https://codelabs.developers.google.com/getting-started-with-antigravity-skills)

### 読み込みパスと優先順位

| スコープ | パス | 自動読み込み | 備考 |
|---------|------|--------------|------|
| Global | `~/.gemini/antigravity/skills/` | ✅ | Personal対応 |
| Workspace | `<workspace-root>/.agent/skills/` | ✅ | Project対応 |

### 重要な特徴

- **ディレクトリベースのSkillsパッケージ**: 各Skillは独立したディレクトリとして管理
- **Progressive Disclosure**: Skillは必要時のみコンテキストにロードされる（コンテキスト肥大化を防止）
- **SKILL.md形式**: Anthropic発祥のAgent Skills open standardを採用

### コンポーネント配置場所

| 種別 | ファイル形式 | Personal | Project |
|------|-------------|----------|---------|
| Skills | `SKILL.md` | `~/.gemini/antigravity/skills/<marketplace>/<plugin>/<skill>/` | `.agent/skills/<marketplace>/<plugin>/<skill>/` |

### 制約事項

- **Skills専用**: Agents、Prompts、Instructionsは別のシステムで管理
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

- **実験的機能**: `/settings` で Agent Skills を `true` に設定して有効化が必要
- **Agents非対応**: `.agent.md` 形式はサポートしない
- **Prompts非対応**: `.prompt.md` 形式はサポートしない

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
- **skillsルートの再帰走査**: ネストしたディレクトリ内の `SKILL.md` も発見されるため、PLMの `<marketplace>/<plugin>/<skill>/` 階層はそのまま読み込まれる見込み
- **Agents（サブエージェント）**: YAMLフロントマター（`name`, `description`, `model`, `readonly`, `is_background`）付きMarkdown。エディタ・CLI・Cloud Agentsで利用可能
- **CommandsはSkillsへ移行中**: `/migrate-to-skills` により既存のCommandsは `disable-model-invocation: true` 付きSkillsへ変換される方向。`.cursor/commands/` 自体は引き続き動作する
- **Claude Code互換**: `.claude/skills/` / `.claude/agents/` を互換パスとして読むため、コンポーネントのフォーマット変換はほぼ不要（`CommandFormat::ClaudeCode` / `AgentFormat::ClaudeCode`）

### Hooks

設定は単一の `hooks.json`（`{"version": 1, "hooks": {"<event>": [{"command": "..."}]}}`）。イベント名は**camelCase**で、Copilot CLI形式（camelCase + `"version": 1`）に近い。

主なイベント: `sessionStart`, `sessionEnd`, `preToolUse`, `postToolUse`, `postToolUseFailure`, `subagentStart`, `subagentStop`, `beforeShellExecution`, `afterShellExecution`, `beforeMCPExecution`, `afterMCPExecution`, `beforeReadFile`, `afterFileEdit`, `beforeSubmitPrompt`, `preCompact`, `stop`, `afterAgentResponse` など。

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
- **sync と Cursor Skills**: Cursor の Skill 名キーは元名、他ターゲットはフラット化名のため、現状 Cursor を含む Skill sync は名前不一致になりうる（既知制限）

### 検証結果

- **Agents のファイル名**: Cursor は `<name>.md` を期待する。PLM の `.agent.md` サフィックスは Cursor では認識されないため、配置時はプレーン `.md` にリネームする（`AgentFormat::ClaudeCode` → `AgentFormat::ClaudeCode` のコピーで内容変換は不要）
- **Agents / Commands のディレクトリ階層**: Cursor 公式ドキュメントに再帰走査の明記がないため、Skills と同様のフラット配置（`agents/<flattened_name>.md` / `commands/<flattened_name>.md`）を採用
- **Commands の配置先**: `/migrate-to-skills` による Skills 移行が進行中だが、`.cursor/commands/` は引き続き動作するため、当面は Commands として配置する
- **Hooks 変換**: Claude Code 形式から Cursor 形式（`version: 1` + camelCase イベント、`command` / `timeout` フィールド）へ変換して配置する

### 未検証

- Cursor CLI（`cursor-agent`）でどのhooksイベントが発火するか

## PLMでの対応方針

| ターゲット | Personal インストール | 追加アクション |
|-----------|----------------------|----------------|
| Codex | `~/.codex/` に配置 | Hook 配置時のみ `~/.codex/config.toml` に `[features] codex_hooks = true` を自動追記（`--no-enable-flag` で抑止可、`codex_hooks = false` 既設定時は警告のみでスキップ） |
| Copilot | ファイル配置 + VSCode設定追記 | `settings.json` への参照追加が必要 |
| Antigravity | `~/.gemini/antigravity/` に配置 | 不要（自動読み込み） |
| Gemini CLI | `~/.gemini/skills/` に配置 | 不要（自動読み込み、要Settings有効化） |
| Cursor | `~/.cursor/` に配置（Skills / Agents / Commands / Hooks） | 不要（自動読み込み）。Hooksは単一 `hooks.json` へ変換配置（上書きガードあり） |

## 将来の拡張候補

- Claude Code（計画中）
- Windsurf
- Aider
- その他SKILL.md対応ツール

## 関連

- [concepts/components](./components.md) - コンポーネント種別
- [concepts/scopes](./scopes.md) - Personal/Projectスコープ
- [commands/target](../commands/target.md) - ターゲット管理コマンド

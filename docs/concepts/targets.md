# ターゲット環境

PLMがサポートするAI開発環境（ターゲット）について説明します。

## 対応ターゲット

| ターゲット | 説明 |
|------------|------|
| **codex** | OpenAI Codex CLI |
| **copilot** | VSCode GitHub Copilot |
| **antigravity** | Google Antigravity IDE |
| **gemini** | Gemini CLI（ターミナルベースAIエージェント） |

## サポートするコンポーネント

| コンポーネント | Codex | Copilot | Antigravity | Gemini CLI |
|----------------|-------|---------|-------------|------------|
| Skills | ✅ | ✅ | ✅ | ✅ |
| Agents | ✅ | ✅ | ❌ | ❌ |
| Commands | ❌ | ✅ | ❌ | ❌ |
| Instructions | ✅ | ✅ | ❌* | ✅** |
| Hooks | ✅ | ✅ | ❌*** | ❌**** |

> *AntigravityはSkills専用の設計で、Instructionsは別途設定で管理します。
> **Gemini CLIは`GEMINI.md`による階層的な指示システムを持ちます。
> ***Antigravity 2.0 は公式に hooks をサポートしますが、PLM 側の変換は未実装です（後述）。
> ****Gemini CLI 単体の hooks 公式仕様は未確認で、PLM では現状非対応です。

> **Hooks の凡例**: ここでの ✅ は「公式仕様が存在し、かつ PLM が変換・配置を実装済み」を意味します。
> 「公式仕様の最新状況」と「PLM の実装状況（実装済み/未実装）」は各ターゲットの節で区別して記載します。

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
| Hooks | `hooks.json` | `~/.codex/hooks.json` | `.codex/hooks.json` |

### Hooks

Codex CLI はエージェントのライフサイクルイベントに対してコマンドを実行する hooks を公式にサポートしています。PLM もこの変換・配置を実装済みです。

公式ドキュメント:
- [Codex Hooks](https://developers.openai.com/codex/hooks)
- [Codex Config (Advanced)](https://developers.openai.com/codex/config-advanced)

> 注: Codex の公式ドメインは bot 対策により直接取得できないため、検索結果・GitHub Issue・公式フォーラム等とクロスチェックした情報です。細部は将来変更される可能性があります。

#### 公式仕様（最新）

- 設定方法は2通りで、同じイベントスキーマを共有します:
  1. `hooks.json`（JSON）
  2. `config.toml` 内のインライン `[hooks]` テーブル（TOML）。`~/.codex/config.toml` 等のアクティブな config レイヤーに配置。
  - プラグインは plugin manifest または `hooks/hooks.json` でライフサイクル設定をバンドル可能。
- 有効化には `config.toml` の feature flag が必要です。

  ```toml
  [features]
  codex_hooks = true
  ```

- 構造は Claude Code と同じく **PascalCase イベント名 + matcher グループ（`matcher` + `hooks[]`）** を保持します。
- サポートイベント（公式 10 種）:

  | イベント | scope |
  |---|---|
  | `PreToolUse` | turn |
  | `PermissionRequest` | turn |
  | `PostToolUse` | turn |
  | `PreCompact` | turn |
  | `PostCompact` | turn |
  | `UserPromptSubmit` | turn |
  | `SubagentStop` | turn |
  | `Stop` | turn |
  | `SessionStart` | thread |
  | `SubagentStart` | subagent-start |

- 実行されるハンドラは `type: "command"` のみです（`prompt` / `agent` ハンドラはパースされますがスキップされます）。
- フィールド: `matcher`（正規表現）, `type`（`"command"`）, `command`, `timeout`, `statusMessage`。Windows 用に `command_windows` / `commandWindows`。
- 別系統の `notify` 設定（`agent-turn-complete` のみ）はデスクトップ通知/webhook 用で、hooks とは別物です。

#### PLM の実装状況

- Codex は Hook コンポーネントを **サポート済み** です。
- 配置先は Personal / Project とも `hooks.json`（JSON inline）で、スクリプト生成は行いません。
- 対応イベントは **6 種のみ**: `SessionStart`, `PreToolUse`, `PostToolUse`, `UserPromptSubmit`, `Stop`, `PermissionRequest`。
  - 公式にある `PreCompact` / `PostCompact` / `SubagentStop` / `SubagentStart` は **未対応**（変換時に除外されます）。
- matcher グループ構造は保持します（`version` / `disableAllHooks` は除去）。
- フィールド正規化: `timeoutSec`（Copilot 形式）→ `timeout`、`comment`（Copilot 形式）→ `statusMessage`。`async` / `once` / `bash` は削除し警告。command hook には `"type": "command"` を自動挿入。
- **未対応の項目:**
  - 複数 Hook コンポーネントのマージ（1 スコープにつき単一 `hooks.json` のみ。複数配置は conflict エラー）
  - `config.toml` 形式の入出力（JSON のみ対応）
  - `command_windows` / `commandWindows`
  - feature flag（`[features] codex_hooks = true`）の案内・自動設定

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

- **Skills専用（PLM 観点）**: PLM 経由で配置できるのは現状 Skills のみ。Agents、Prompts、Instructionsは別のシステムで管理されます。
- Skillsはタスク終了後にコンテキストから解放される（エフェメラル）

### Hooks

Antigravity 2.0 は主要機能の一つとして event-driven automation の hooks を **公式サポート** するようになりました。ただし **PLM 側の変換は現状未実装** です（今後対応予定。関連 issue は別途登録されます）。

公式ドキュメント:
- [Antigravity Hooks](https://antigravity.google/docs/hooks)
- [antigravity-sdk-python (hooks/README.md)](https://github.com/google-antigravity/antigravity-sdk-python)
- [Hooks in Antigravity（公式フォーラム）](https://discuss.ai.google.dev/t/hooks-in-antigravity/120458)

> 注: Antigravity の公式ドメインは bot 対策により直接取得できないため、検索結果・SDK リポジトリ・公式フォーラム等とクロスチェックした情報です。細部は将来変更される可能性があります。

#### 公式仕様（最新・確認できた範囲）

- フォーマット: `hooks.json`（JSON）。
- 配置場所:
  - Project（workspace）スコープ: `.agents/` ディレクトリ
  - Global（personal）スコープ: `~/.gemini/config/hooks.json`
- サポートイベント（確認できた範囲）: `PreToolUse`, `PostToolUse`, `Stop`, `SubagentStop`, `SessionStart`, `SessionEnd`, `UserPromptSubmit`, `PreCompact`, `Notification`。
- 構造は Claude Code に酷似しますが、**トップレベルが「フック名 → イベント設定」のマップ** になる点が異なります（Claude Code は直接 `hooks.<Event>`）。

  ```json
  {
    "my-linter-hook": {
      "PostToolUse": [
        { "matcher": "run_command",
          "hooks": [ { "type": "command", "command": "./scripts/lint.sh", "timeout": 10 } ] }
      ]
    }
  }
  ```

- I/O: stdin で JSON を受け取り、stdout で JSON（`allow` / `deny` / `ask`）を返します。`enabled: false` で個別 hook を無効化できます。

#### PLM の実装状況

- Antigravity は Hook コンポーネント **非対応**。変換 converter も未実装です。
- 公式が hooks をサポートし始めたため、PLM 側でも今後の対応が必要です。

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
- **Hooks非対応**: Gemini CLI 単体の hooks 公式仕様は確認できておらず（Antigravity 経由が実態の可能性）、PLM でも現状 Hook コンポーネントは非対応です。

## PLMでの対応方針

| ターゲット | Personal インストール | 追加アクション |
|-----------|----------------------|----------------|
| Codex | `~/.codex/` に配置 | 不要（自動読み込み） |
| Copilot | ファイル配置 + VSCode設定追記 | `settings.json` への参照追加が必要 |
| Antigravity | `~/.gemini/antigravity/` に配置 | 不要（自動読み込み） |
| Gemini CLI | `~/.gemini/skills/` に配置 | 不要（自動読み込み、要Settings有効化） |

## 将来の拡張候補

- Cursor（.cursor/）
- Windsurf
- Aider
- その他SKILL.md対応ツール

## 関連

- [concepts/components](./components.md) - コンポーネント種別
- [concepts/scopes](./scopes.md) - Personal/Projectスコープ
- [commands/target](../commands/target.md) - ターゲット管理コマンド

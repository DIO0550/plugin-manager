# Claude Code ↔ 各ターゲット Hooks スキーマ対応表

PLM でフック変換ツールを実装する際のリファレンス。
Claude Code を入力元とし、Copilot CLI / Codex CLI / Google Antigravity への変換仕様を対比する。

各セクションでは **公式仕様** と **PLM 実装状況** を区別して記載する。
Antigravity の詳細は [セクション 10](#10-google-antigravity)（実装 Issue [#309](https://github.com/DIO0550/plugin-manager/issues/309) の単一リファレンス）。

## 公式ドキュメント

| 環境 | URL |
|------|-----|
| Claude Code Hooks | https://docs.anthropic.com/en/docs/claude-code/hooks |
| Claude Code Plugins | https://docs.anthropic.com/en/docs/claude-code/plugins |
| Copilot CLI Hooks 設定 | https://docs.github.com/en/copilot/reference/hooks-configuration |
| Copilot CLI Hooks ガイド | https://docs.github.com/en/copilot/how-tos/copilot-cli/customize-copilot/use-hooks |
| Copilot CLI Hooks チュートリアル | https://docs.github.com/en/copilot/tutorials/copilot-cli-hooks |
| Codex CLI Hooks | https://developers.openai.com/codex/hooks |
| Antigravity Hooks | https://antigravity.google/docs/hooks |
| Antigravity Hooks フォーラム | https://discuss.ai.google.dev/t/hooks-in-antigravity/120458 |

---

## 1. 設定ファイル構造

### Claude Code

```json
{
  "hooks": {
    "<PascalCaseEvent>": [
      {
        "matcher": "<regex>",
        "hooks": [
          {
            "type": "command",
            "command": "<shell command>",
            "timeout": 600,
            "statusMessage": "Processing..."
          }
        ]
      }
    ]
  }
}
```

**配置場所:**
- `~/.claude/settings.json`（ユーザーレベル）
- `.claude/settings.json`（プロジェクトレベル、Git 共有可）
- `.claude/settings.local.json`（プロジェクトレベル、ローカル専用）
- プラグイン内: `hooks/hooks.json`

### Copilot CLI

```json
{
  "version": 1,
  "hooks": {
    "<camelCaseEvent>": [
      {
        "type": "command",
        "bash": "<shell command>",
        "powershell": "<shell command>",
        "cwd": "<optional working directory>",
        "env": { "<KEY>": "<value>" },
        "timeoutSec": 30,
        "comment": "optional documentation"
      }
    ]
  }
}
```

**配置場所:**
- `.github/hooks/*.json`（プロジェクトレベル）

### Google Antigravity（要約）

詳細は [セクション 10](#10-google-antigravity)。トップレベルは **命名済みフック → イベント設定** のマップ。

```json
{
  "my-linter-hook": {
    "enabled": true,
    "PostToolUse": [
      {
        "matcher": "run_command",
        "hooks": [
          { "type": "command", "command": "./scripts/lint.sh", "timeout": 10 }
        ]
      }
    ]
  }
}
```

**配置場所:**
- Project: `.agents/hooks.json`
- Global（Personal）: `~/.gemini/config/hooks.json`

### 構造差分

| 項目 | Claude Code | Copilot CLI | Antigravity |
|------|-------------|-------------|-------------|
| トップレベル | `"hooks": { <Event>: ... }` | `"version": 1` + `"hooks"` | **命名フック → イベント設定** のマップ |
| ネスト深度 | event → matcher group → hooks[] | event → hooks[]（フラット） | 命名フック → event →（ToolUse のみ）matcher group → hooks[] |
| matcher | matcher group の `"matcher"`（regex） | なし（スクリプト内で判定） | `PreToolUse` / `PostToolUse` のみ。他イベントはフラット handlers |
| コマンドキー | `"command"` | `"bash"` / `"powershell"` | `"command"` |
| タイムアウト | `"timeout"`（秒、デフォルト 600） | `"timeoutSec"`（秒、デフォルト 30） | `"timeout"`（秒、デフォルト 30） |
| 作業ディレクトリ | なし（`CLAUDE_PROJECT_DIR`） | `"cwd"` | なし（stdin の `workspacePaths`） |
| 環境変数 | `CLAUDE_*` | `"env"` オブジェクト | フック専用 env なし（stdin JSON） |
| フック種別 | `command` / `http` / `prompt` / `agent` | `command` / `prompt` | **`command` のみ** |
| 無効化 | `"disableAllHooks": true` | なし | 命名フック単位の `"enabled": false` |

**変換時の注意:**
- Claude Code の matcher グループ構造を Copilot CLI のフラット構造に展開する必要がある
- Claude Code の `http` / `agent` フックは Copilot CLI に直接変換できない（`command` ラッパーが必要）
- Copilot CLI の `powershell` キーは Claude Code に対応がない
- Antigravity はトップレベルを命名フックでラップし、非 ToolUse イベントは matcher グループではなくフラット handlers にする必要がある（詳細はセクション 10）

---

## 2. イベント名マッピング

### 双方向対応（変換可能）

| Claude Code (PascalCase) | Copilot CLI (camelCase) | 備考 |
|--------------------------|------------------------|------|
| `SessionStart` | `sessionStart` | `source` の値域が異なる（後述） |
| `SessionEnd` | `sessionEnd` | `reason` フィールドの値域が異なる |
| `PreToolUse` | `preToolUse` | stdin/stdout 構造が異なる |
| `PostToolUse` | `postToolUse` | `toolResult` の型が異なる |
| `UserPromptSubmit` | `userPromptSubmitted` | イベント名の末尾が異なる (`Submit` vs `Submitted`) |
| `Stop` | `agentStop` | Claude Code は `Stop`、Copilot CLI は `agentStop` |
| `SubagentStop` | `subagentStop` | ほぼ同等 |

### Claude Code 固有（Copilot CLI に対応なし）

| Claude Code | 近似手段 |
|-------------|---------|
| `PostToolUseFailure` | `postToolUse` で `toolResult.resultType === "failure"` を判定 |
| `PreCompact` / `PostCompact` | なし |
| `PermissionRequest` | `preToolUse` で部分的に代替 |
| `Notification` | なし |
| `SubagentStart` | なし |
| `TeammateIdle` | なし |
| `TaskCompleted` | なし |
| `InstructionsLoaded` | なし |
| `ConfigChange` | なし |
| `WorktreeCreate` / `WorktreeRemove` | なし |
| `Elicitation` / `ElicitationResult` | なし |

### Copilot CLI 固有（Claude Code に対応なし）

| Copilot CLI | 近似手段 |
|-------------|---------|
| `errorOccurred` | `PostToolUseFailure` で部分的に代替 |

### Codex CLI（Claude Code と 1:1 対応）

Codex hooks は Claude Code と同じ PascalCase 命名で 10 イベントをサポートし、PLM の `CodexEventMap` は変換時にイベント名をそのまま保持する。

| Claude Code / Codex | scope | PLM 対応 |
|---------------------|-------|----------|
| `SessionStart` | thread | ✅ |
| `PreToolUse` | turn | ✅ |
| `PermissionRequest` | turn | ✅ |
| `PostToolUse` | turn | ✅ |
| `UserPromptSubmit` | turn | ✅ |
| `Stop` | turn | ✅ |
| `PreCompact` | turn | ✅ |
| `PostCompact` | turn | ✅ |
| `SubagentStop` | turn | ✅ |
| `SubagentStart` | subagent-start | ✅ |

出典: <https://developers.openai.com/codex/hooks>

### Codex CLI コマンドフックのフィールドマッピング

Codex の command フックは POSIX シェル向けの `command` と Windows 向けの `command_windows` を持つ。PLM は Claude Code 由来の表記揺れ（camelCase `commandWindows`）を Codex 仕様の snake_case (`command_windows`) に正規化する。

| 入力キー (Claude Code 形式) | 出力キー (Codex 形式) | 挙動 |
|---|---|---|
| `command_windows` | `command_windows` | command 型なら保持。command 型以外では削除して警告 |
| `commandWindows` | `command_windows` | snake_case にリネーム。`command_windows` と併存時は snake_case を優先し camelCase 側を警告付きで破棄 |

**表記揺れ正規化の理由**: Codex 公式の `config.toml` は snake_case を採用するため、PLM は入力に `commandWindows`（camelCase）が含まれていれば自動で `command_windows` にリネームする。両キーが同時に存在する場合は仕様準拠側（snake_case）を優先し、`ConversionWarning::RemovedField` で重複を通知する。command 以外の hook 型（`http` など）に付与された Windows コマンドフィールドは意味を持たないため削除し警告する。

### Codex CLI における prompt / agent ハンドラの扱い

Codex CLI のフック実装は `type: "command"` のみ実行し、`type: "prompt"` / `type: "agent"` のハンドラはパースした上でスキップする（出典: <https://developers.openai.com/codex/hooks>）。

PLM が Claude Code 形式のプラグインを Codex 向けに変換する際の挙動:

| 入力 hook 型 | 変換結果 | 警告 |
|---|---|---|
| `command` | inline 保持（command フィールドが Codex 設定にそのまま残る） | なし（フィールド正規化警告は別途） |
| `http` | 出力から除外 | `ConversionWarning::UnsupportedHookType { hook_type: "http" }` |
| `prompt` | **inline 保持**（元の `prompt` エントリが hooks 配列にそのまま残り、Codex 実行時にスキップされる） | `ConversionWarning::PromptAgentHookStub { hook_type: "prompt", event }` を**ハンドラごとに 1 件** |
| `agent` | **inline 保持**（元の `agent` エントリが hooks 配列にそのまま残り、Codex 実行時にスキップされる） | `ConversionWarning::PromptAgentHookStub { hook_type: "agent", event }` を**ハンドラごとに 1 件** |

**設計判断**: prompt / agent ハンドラは Codex CLI 自身がスキップするため、変換結果から除外しても害は小さい。一方で inline 保持することで、利用者が後から手動で `command` ハンドラに書き換える際に元の設定を参照しやすく、警告とセットで「Codex では実行されない」事実を明確に伝えられる。同一イベント内に prompt/agent が複数ある場合や command と混在する場合も、command は通常変換され、prompt/agent はそれぞれ独立に警告 + inline 保持される。

### Codex CLI hooks の feature flag 自動有効化（実装メモ）

Codex CLI で hooks を有効化するには `config.toml` に以下のフラグが必要:

```toml
[features]
codex_hooks = true
```

PLM は Codex hook を配置すると同時に、scope に応じた `config.toml` の `[features] codex_hooks = true` を自動追記する:

- `--scope personal` → `~/.codex/config.toml`
- `--scope project` → `<project_root>/.codex/config.toml`

挙動:

- 既存ファイルのコメント・キー順・改行は `toml_edit` クレートにより保持される
- ファイル未存在時は親ディレクトリと共に新規作成
- `codex_hooks = true` 既設定 → 何もしない（冪等）
- `codex_hooks = false` 明示設定 → ユーザー意思を尊重し警告のみでスキップ
- 1 回の `plm install` / `plm import` で 1 scope につき 1 回のみ実行（複数 Hook を配置しても重複適用なし）
- TOML パースエラーや書き込み失敗時は hook 配置自体は成功扱いとし、警告を stderr に出力

抑止方法:

- `plm install <pkg> --target codex --no-enable-flag` / `plm import <repo> --target codex --no-enable-flag`
- スキップ時は手動で `[features] codex_hooks = true` を追記する必要がある

注: `features.codex_hooks` は公式ドキュメント上 deprecated alias と明記されており、将来名前が変わる可能性がある（出典: <https://developers.openai.com/codex/config-advanced>、確認日付: 2026-06-29）。実装側は `src/target/env/codex/feature_flag.rs` でテーブル名・キー名を定数化しており、名前変更時は 1 箇所修正で済む。

### 3 ターゲット横断のイベント対応表（要約）

○=公式サポート、×=非対応、〜=近似マッピング候補。詳細は各節・セクション 10。

| Claude Code | Copilot CLI | Codex CLI | Antigravity（公式） |
|-------------|-------------|-----------|---------------------|
| `SessionStart` | `sessionStart` | `SessionStart` | 〜 `PreInvocation` |
| `SessionEnd` | `sessionEnd` | × | 〜 `Stop` |
| `PreToolUse` | `preToolUse` | `PreToolUse` | `PreToolUse` |
| `PostToolUse` | `postToolUse` | `PostToolUse` | `PostToolUse` |
| `UserPromptSubmit` | `userPromptSubmitted` | `UserPromptSubmit` | 〜 `PreInvocation` |
| `Stop` | `agentStop` | `Stop` | `Stop` |
| `SubagentStop` | `subagentStop` | `SubagentStop` | × |
| `PermissionRequest` | （`preToolUse` 近似） | `PermissionRequest` | ×（`ask` / `force_ask` で部分代替） |
| `PreCompact` / `PostCompact` | × | ○ | × |
| `Notification` | × | × | × |
| — | — | — | `PreInvocation` / `PostInvocation`（Antigravity 固有） |

> **注:** Issue #309 / 初期調査では Antigravity に `SessionStart` / `SessionEnd` / `SubagentStop` / `UserPromptSubmit` / `PreCompact` / `Notification` が列挙されていたが、**2026-07-26 時点の公式ドキュメント**（https://antigravity.google/docs/hooks）ではサポートイベントは `PreToolUse` / `PostToolUse` / `PreInvocation` / `PostInvocation` / `Stop` の 5 種。本表は公式を正とする。

---

## 3. stdin スキーマ

### 共通フィールド

| フィールド | Claude Code | Copilot CLI |
|-----------|-------------|-------------|
| セッション識別 | `session_id` (string) | なし |
| タイムスタンプ | なし | `timestamp` (Unix ms) |
| 作業ディレクトリ | `cwd` | `cwd` |
| トランスクリプト | `transcript_path` | なし |
| 権限モード | `permission_mode` | なし |
| イベント名 | `hook_event_name` | なし（暗黙） |
| エージェント | `agent_id`, `agent_type` | なし |

### PreToolUse / preToolUse

**Claude Code:**

```jsonc
{
  "session_id": "abc123",
  "transcript_path": "/path/to/transcript.jsonl",
  "cwd": "/project",
  "permission_mode": "default",
  "hook_event_name": "PreToolUse",
  "tool_name": "Bash",              // PascalCase
  "tool_use_id": "toolu_...",       // ツール呼び出し ID
  "tool_input": {                    // オブジェクト（そのまま）
    "command": "npm test",
    "description": "Run tests"
  }
}
```

**Copilot CLI:**

```jsonc
{
  "timestamp": 1704614600000,
  "cwd": "/project",
  "toolName": "bash",               // 小文字
  "toolArgs": "{\"command\":\"npm test\",\"description\":\"Run tests\"}"
                                     // JSON 文字列（要パース）
}
```

**フィールドマッピング:**

| Claude Code | Copilot CLI | 変換 |
|-------------|-------------|------|
| `tool_name` (PascalCase) | `toolName` (小文字) | ケース変換 + ツール名マッピング（セクション7参照） |
| `tool_input` (object) | `toolArgs` (JSON string) | `JSON.stringify()` / `JSON.parse()` |
| `tool_use_id` | なし | 削除 |
| `session_id` | なし | 削除 |
| なし | `timestamp` | `Date.now()` で生成 |

### PostToolUse / postToolUse

**Claude Code:**

```jsonc
{
  "session_id": "abc123",
  "cwd": "/project",
  "tool_name": "Bash",
  "tool_use_id": "toolu_...",
  "tool_input": { "command": "npm test" },
  "tool_response": { /* ツール固有のレスポンスオブジェクト */ }
}
```

**Copilot CLI:**

```jsonc
{
  "timestamp": 1704614700000,
  "cwd": "/project",
  "toolName": "bash",
  "toolArgs": "{\"command\":\"npm test\"}",
  "toolResult": {
    "resultType": "success",             // "success" | "failure" | "denied"
    "textResultForLlm": "All tests passed"
  }
}
```

**注意:**
- Claude Code の `tool_response` はツール固有のオブジェクト、Copilot CLI の `toolResult` は `resultType` + `textResultForLlm` の固定構造
- Claude Code の `PostToolUseFailure` は別イベントだが、Copilot CLI では `postToolUse` の `resultType: "failure"` で表現

### SessionStart / sessionStart

**Claude Code:**

```jsonc
{
  "session_id": "abc123",
  "cwd": "/project",
  "source": "startup",    // "startup" | "resume" | "clear" | "compact"
  "model": "claude-sonnet-4-20250514"
}
```

**Copilot CLI:**

```jsonc
{
  "timestamp": 1704614400000,
  "cwd": "/project",
  "source": "new",        // "new" | "resume" | "startup"
  "initialPrompt": "fix the bug in auth.ts"
}
```

**`source` 値のマッピング:**

| Claude Code | Copilot CLI | 備考 |
|-------------|-------------|------|
| `startup` | `new` | 新規セッション |
| `resume` | `resume` | 既存セッション再開 |
| `clear` | `new` | コンテキストクリア → 新規扱い |
| `compact` | — | 圧縮イベント（Copilot CLI に対応なし） |
| — | `startup` | Copilot CLI 固有（プロセス起動） |

### SessionEnd / sessionEnd

**Claude Code:**

```jsonc
{
  "session_id": "abc123",
  "cwd": "/project",
  "reason": "prompt_input_exit"
  // "clear" | "logout" | "prompt_input_exit" | "bypass_permissions_disabled" | "other"
}
```

**Copilot CLI:**

```jsonc
{
  "timestamp": 1704618000000,
  "cwd": "/project",
  "reason": "complete"
  // "complete" | "error" | "abort" | "timeout" | "user_exit"
}
```

### UserPromptSubmit / userPromptSubmitted

**Claude Code:**

```jsonc
{
  "session_id": "abc123",
  "cwd": "/project",
  "prompt": "fix the auth bug"
}
```

**Copilot CLI:**

```jsonc
{
  "timestamp": 1704614500000,
  "cwd": "/project",
  "prompt": "fix the auth bug"
}
```

構造は類似。`session_id` ↔ `timestamp` の差のみ。

### errorOccurred（Copilot CLI 固有）

```jsonc
{
  "timestamp": 1704614800000,
  "cwd": "/project",
  "error": {
    "message": "Network timeout",
    "name": "TimeoutError",
    "stack": "TimeoutError: Network timeout\n    at ..."
  }
}
```

Claude Code には対応イベントなし。`PostToolUseFailure` で部分的に代替可能。

---

## 4. stdout スキーマ

### 共通出力フィールド

**Claude Code（全イベント共通）:**

```json
{
  "continue": true,
  "stopReason": "理由（continue が false の場合）",
  "suppressOutput": false,
  "systemMessage": "ユーザーへの警告メッセージ",
  "hookSpecificOutput": { /* イベント固有 */ }
}
```

**Copilot CLI:**
`preToolUse` のみ stdout を処理する。他のイベントでは出力は無視される。

### PreToolUse / preToolUse の応答

**Claude Code:**

```json
{
  "hookSpecificOutput": {
    "hookEventName": "PreToolUse",
    "permissionDecision": "allow",
    "permissionDecisionReason": "Validated tool input",
    "updatedInput": { /* tool_input の修正版（任意） */ },
    "additionalContext": "追加コンテキスト（任意）"
  }
}
```

**Copilot CLI:**

```json
{
  "permissionDecision": "deny",
  "permissionDecisionReason": "Dangerous operation blocked"
}
```

**変換ポイント:**
- Claude Code → Copilot CLI: `hookSpecificOutput` をアンラップし、`hookEventName` を除去
- Copilot CLI → Claude Code: `hookSpecificOutput` でラップし、`hookEventName` を追加
- Claude Code の `updatedInput` / `additionalContext` は Copilot CLI に対応なし
- Copilot CLI では `"deny"` のみが実際に処理される（`"allow"` は出力なし + exit 0 と同等）

### Stop / agentStop の応答

**Claude Code:**

```json
{
  "decision": "block",
  "reason": "Tests not passing yet"
}
```

`"block"` で停止を阻止し処理を続行させる。`"approve"` で停止を許可。

**Copilot CLI:**
`agentStop` の出力は無視される。

### SessionStart の応答

**Claude Code:**

```json
{
  "hookSpecificOutput": {
    "hookEventName": "SessionStart",
    "additionalContext": "Claude へ注入するコンテキスト"
  }
}
```

**Copilot CLI:**
出力は無視される。Side effect のみ。

---

## 5. exit code の意味

### Claude Code（4段階）

| exit code | 意味 | stdout | stderr |
|-----------|------|--------|--------|
| `0` | 成功 | JSON としてパース | 無視 |
| `1` | 非ブロッキングエラー | 無視 | verbose モードで表示 |
| `2` | **ブロッキングエラー** | 無視 | ユーザーにフィードバック |
| その他 | 非ブロッキングエラー | 無視 | verbose モードで表示 |

**exit code 2 のイベント別効果:**

| イベント | 効果 |
|---------|------|
| `PreToolUse` | ツール呼び出しをブロック |
| `PermissionRequest` | 権限を拒否 |
| `UserPromptSubmit` | プロンプト処理をブロック、入力を消去 |
| `Stop` / `SubagentStop` | 停止を阻止し処理を続行 |
| `ConfigChange` | 設定変更をブロック |
| `PostToolUse` | stderr を Claude に表示（ツールは実行済み） |
| `SessionStart` / `SessionEnd` | stderr をユーザーに表示のみ |

### Copilot CLI（2段階）

| exit code | 意味 | 備考 |
|-----------|------|------|
| `0` | 成功 | stdout を JSON としてパース |
| 非ゼロ | エラー | ログに記録しスキップ。**実行をブロックしない** |

> **重要:** Copilot CLI でツール実行を拒否するには、exit code ではなく exit 0 + stdout の JSON で `permissionDecision: "deny"` を返す。

### 変換時の注意

Claude Code の exit code 2（ブロック）を Copilot CLI に変換する場合は、exit 0 + `{"permissionDecision": "deny", "permissionDecisionReason": "<stderr の内容>"}` に変換する必要がある。

---

## 6. 環境変数

### Claude Code がフックに提供する変数

| 変数名 | 利用可能イベント | 説明 |
|--------|----------------|------|
| `CLAUDE_PROJECT_DIR` | 全 command フック | プロジェクトルート |
| `CLAUDE_PLUGIN_ROOT` | プラグインフック | プラグインのルートディレクトリ |
| `CLAUDE_FILE_PATHS` | ツール系イベント | 操作対象ファイルパス |
| `CLAUDE_ENV_FILE` | `SessionStart` のみ | 環境変数永続化用ファイルパス |
| `CLAUDE_CODE_REMOTE` | 全フック | リモート Web 環境では `"true"` |

### Copilot CLI がフックに提供する変数

| 変数名 | 説明 |
|--------|------|
| `COPILOT_MODEL` | 使用中の AI モデル |
| `COPILOT_HOME` | 設定ディレクトリ（デフォルト: `~/.copilot/`） |

加えて、フック定義の `"env"` オブジェクトでカスタム環境変数を注入可能:

```json
{
  "type": "command",
  "bash": "./scripts/hook.sh",
  "env": { "LOG_LEVEL": "INFO", "CUSTOM_KEY": "value" }
}
```

---

## 7. ツール名の対応

### Hooks コンテキスト（stdin の `tool_name` / `toolName`）

Claude Code は PascalCase、Copilot CLI は小文字。

| Claude Code | Copilot CLI | 備考 |
|-------------|-------------|------|
| `Bash` | `bash` | |
| `Read` | `view` | 名前が異なる |
| `Write` | `create` | 名前が異なる |
| `Edit` | `edit` | |
| `MultiEdit` | `edit` | Claude Code 固有ツール → `edit` に統合 |
| `Glob` | `glob` | |
| `Grep` | `grep` | |
| `WebFetch` | `web_fetch` | camelCase → snake_case |
| `WebSearch` | — | Copilot CLI に対応なし |
| `Agent` | `task` | 名前が異なる |
| — | `ask_user` | Copilot CLI 固有 |
| — | `memory` | Copilot CLI 固有 |
| — | `powershell` | Copilot CLI 固有（Windows） |
| `mcp__<server>__<tool>` | — | Claude Code の MCP ツール（Copilot CLI に対応なし） |

### PLM 内部のツール名マッピング（参考）

`src/parser/convert.rs` には Prompt/Agent ファイルの `tools` 配列で使われるツール名の変換がある。これは Hooks の `toolName` とは別のコンテキスト:

| Claude Code | Copilot (Prompt/Agent) | 備考 |
|-------------|----------------------|------|
| `Read` / `Write` / `Edit` | `codebase` | N:1 マッピング |
| `Grep` / `Glob` | `search/codebase` | N:1 マッピング |
| `Bash` | `terminal` | |
| `Bash(git...)` | `githubRepo` | git コマンド限定 |
| `WebFetch` | `fetch` | |
| `WebSearch` | `websearch` | |

---

## 8. フック種別

| 種別 | Claude Code | Copilot CLI | Antigravity |
|------|-------------|-------------|-------------|
| `command` | 全イベントで使用可 | 全イベントで使用可 | **唯一のサポート種別**（省略時も `command`） |
| `http` | HTTP POST。`headers` で `$VAR` 展開可 | **なし** | **なし** |
| `prompt` | LLM 評価フック。`{ok, reason}` | `sessionStart` のみ（自動送信） | **なし** |
| `agent` | サブエージェント調査。`{ok, reason}` | **なし** | **なし** |

**`prompt` の意味の違い:**
- Claude Code: LLM にフック入力を評価させ、`ok: false` でブロックする判定フック
- Copilot CLI: テキストをユーザー入力として自動送信するセットアップ用フック

---

## 9. 変換時のまとめ

### Claude Code → Copilot CLI

1. トップレベルに `"version": 1` を追加
2. イベント名を PascalCase → camelCase に変換（`Stop` → `agentStop`、`UserPromptSubmit` → `userPromptSubmitted` に注意）
3. matcher グループ構造をフラットに展開（matcher の条件はスクリプト内ロジックに移動）
4. `"command"` → `"bash"` にキー名変更
5. `"timeout"` → `"timeoutSec"` にキー名変更
6. `http` / `agent` フックは `command` ラッパースクリプトに変換
7. Copilot CLI に対応のないイベント（`Notification`, `PreCompact` 等）は除外または警告

### Claude Code → Antigravity

1. トップレベルを **命名フック → イベント設定** のマップに包む（プラグイン名などから一意な hook 名を生成）
2. `PreToolUse` / `PostToolUse` は matcher グループ構造を維持（ツール名は Antigravity 名へリマップ）
3. `PreInvocation` / `PostInvocation` / `Stop` は **フラットな handlers 配列**（`{matcher, hooks:[]}` でラップすると無視される）
4. イベント近似: `SessionStart` / `UserPromptSubmit` → `PreInvocation`、`SessionEnd` → `Stop`（意味が一致しない点を警告）
5. 対応のないイベント（`SubagentStop`, `PreCompact`, `Notification` 等）は除外 + 警告
6. `http` / `prompt` / `agent` は `command` へ変換できない場合は除外 + 警告（公式は `command` のみ）
7. stdin/stdout 差分はラッパースクリプトで吸収（`tool_name`/`tool_input` ↔ `toolCall`、`permissionDecision` ↔ `decision`）
8. 配置先: Project `.agents/hooks.json` / Personal `~/.gemini/config/hooks.json`（単一ファイル・マージ戦略は #309 で決定）

詳細はセクション 10。

### Copilot CLI → Claude Code

1. `"version"` フィールドを除去
2. イベント名を camelCase → PascalCase に変換
3. フラット配列を matcher グループ構造にラップ（matcher なしの場合は `"matcher": ""` で全マッチ）
4. `"bash"` → `"command"` にキー名変更（`"powershell"` は除外または警告）
5. `"timeoutSec"` → `"timeout"` にキー名変更
6. `"cwd"` / `"env"` は Claude Code に対応がないため除外または警告
7. `errorOccurred` は除外

---

## 10. Google Antigravity

> **出典:** [Antigravity Hooks](https://antigravity.google/docs/hooks)（本文取得・確認日: 2026-07-26）  
> **PLM 実装状況:** 実装済み。IDE / CLI 共通。Gemini CLI 単体の hooks は追わず本セクション（Antigravity）に一本化する。  
> **関連:** `docs/concepts/targets.md`、`docs/hooks-conversion/index.md`

### 10.1 配置先

| スコープ | パス | 備考 |
|----------|------|------|
| Project（workspace） | `.agents/hooks.json` | AGY / AGY CLI / AGY IDE 共通 |
| Global（Personal） | `~/.gemini/config/hooks.json` | 同上 |

公式は customization directory として `.agents/`（workspace）と `~/.gemini/config/`（global）を明記。ファイル名は `hooks.json`。

> 一部の二次記事では CLI 専用に `~/.gemini/antigravity-cli/hooks.json` と記載されるが、2026-07 時点の公式および実機検証記事（Atamel, 2026-07-16）は **`~/.gemini/config/hooks.json` を Global 正とする**。実装前に最新公式で再確認すること。

### 10.2 設定構造（公式）

トップレベルは **フック名 → イベント設定** のマップ。Claude Code（`hooks.<Event>`）や Copilot（`hooks.<event>`）とは異なり、複数の命名フックを 1 ファイルに同居できる。

```json
{
  "my-linter-hook": {
    "PostToolUse": [
      {
        "matcher": "run_command",
        "hooks": [
          {
            "type": "command",
            "command": "./scripts/lint.sh",
            "timeout": 10
          }
        ]
      }
    ]
  },
  "safety-gate": {
    "enabled": false,
    "PreToolUse": [
      {
        "matcher": "run_command",
        "hooks": [
          {
            "command": "./scripts/safety-check.sh"
          }
        ]
      }
    ]
  },
  "reminder": {
    "PreInvocation": [
      {
        "type": "command",
        "command": "./scripts/reminder.sh"
      }
    ]
  }
}
```

#### 命名フックのフィールド

| フィールド | 型 | 説明 |
|------------|-----|------|
| `enabled` | boolean | 省略時 `true`。`false` で個別無効化（削除不要） |
| `PreToolUse` / `PostToolUse` / `PreInvocation` / `PostInvocation` / `Stop` | array | 各イベントのハンドラ定義 |

#### ハンドラ設定（`hooks` 配列要素）

| フィールド | 型 | 説明 |
|------------|-----|------|
| `type` | string | 省略可。現状 `"command"` のみ。デフォルト `"command"` |
| `command` | string | 必須。実行するシェルコマンド |
| `timeout` | integer | 秒。デフォルト **30**（Claude Code の 600 と異なる） |

#### 構造の非対称性（変換時に重要）

| イベント | 配列要素の形 | matcher |
|----------|--------------|---------|
| `PreToolUse` / `PostToolUse` | `{ "matcher": "<regex>", "hooks": [ ... ] }` | ツール名に対する regex。`""` / `"*"` は全ツール |
| `PreInvocation` / `PostInvocation` / `Stop` | `{ "type": "command", "command": "..." }`（**フラット**） | 無視。`{matcher, hooks:[]}` でラップすると ** silently skip** される報告あり |

### 10.3 サポートイベント

| イベント | 発火タイミング | Matcher |
|----------|----------------|---------|
| `PreToolUse` | ツール実行前 | ツール名（例: `run_command`） |
| `PostToolUse` | ツール完了後 | ツール名 |
| `PreInvocation` | モデル呼び出し前 | N/A |
| `PostInvocation` | ツール呼び出し群の後 | N/A |
| `Stop` | 実行ループ終了時 | N/A |

#### Claude Code からのイベント対応方針（#309 向け）

| Claude Code | Antigravity | 変換方針 |
|-------------|-------------|---------|
| `PreToolUse` | `PreToolUse` | 直接変換。matcher のツール名をリマップ |
| `PostToolUse` | `PostToolUse` | 直接変換。stdout は `{}` 固定寄り |
| `Stop` | `Stop` | 直接変換。出力意味が異なる（後述） |
| `SessionStart` | `PreInvocation`（近似） | 警告付き近似。毎モデル呼び出しで発火する点に注意 |
| `UserPromptSubmit` | `PreInvocation`（近似） | 同上。`invocationNum` 等でフィルタが必要な場合あり |
| `SessionEnd` | `Stop`（近似） | 警告付き近似。`terminationReason` / `fullyIdle` が異なる |
| `SubagentStop` / `SubagentStart` | × | 除外 + 警告 |
| `PreCompact` / `PostCompact` / `Notification` / `PermissionRequest` 等 | × | 除外 + 警告 |
| — | `PostInvocation` | Claude Code 側に直接対応なし（生成しない） |

### 10.4 サポートツール名（matcher 用）

Antigravity のツール名は Claude Code / Copilot と大きく異なる。matcher 変換時に必須。

| カテゴリ | Antigravity ツール名（例） |
|----------|---------------------------|
| ファイル | `view_file`, `write_to_file`, `replace_file_content`, `multi_replace_file_content`, `list_dir`, `find_by_name` |
| 検索 | `grep_search`, `search_web`, `read_url_content` |
| 実行 | `run_command`, `manage_task`, `schedule`, `list_permissions`, `ask_permission` |
| エージェント | `invoke_subagent`, `define_subagent`, `send_message`, `manage_subagents` |
| その他 | `ask_question`, `generate_image` |

#### Claude Code → Antigravity（matcher 用の代表マッピング）

| Claude Code | Antigravity | 備考 |
|-------------|-------------|------|
| `Bash` | `run_command` | |
| `Read` | `view_file` | |
| `Write` | `write_to_file` | |
| `Edit` | `replace_file_content` | |
| `MultiEdit` | `multi_replace_file_content` | |
| `Glob` | `find_by_name` | |
| `Grep` | `grep_search` | |
| `WebFetch` | `read_url_content` | |
| `WebSearch` | `search_web` | |
| `Agent` | `invoke_subagent` | 近似 |

### 10.5 stdin / stdout 契約

フィールド名は **camelCase**。全イベント共通メタデータ:

| フィールド | 型 | 説明 |
|------------|-----|------|
| `conversationId` | string | 会話 UUID |
| `workspacePaths` | string[] | マウント済みワークスペースの絶対パス |
| `transcriptPath` | string | `transcript.jsonl` の絶対パス（`~/.gemini/antigravity/...` または CLI 相当） |
| `artifactDirectoryPath` | string | 成果物・スクリーンショットディレクトリ |

> AGY は stdin にイベント名を含めない（Atamel 記事で確認）。スクリプト側で CLI 引数等によりイベントを識別する必要がある場合がある。

#### PreToolUse

**stdin（例）:**

```json
{
  "toolCall": {
    "name": "run_command",
    "args": {
      "CommandLine": "npm test",
      "Cwd": "/workspace/project",
      "WaitMsBeforeAsync": 5000
    }
  },
  "stepIdx": 19,
  "conversationId": "ec33ebf9-0cba-4100-8142-c61503f6c587",
  "workspacePaths": ["/workspace/project"],
  "transcriptPath": "~/.gemini/antigravity/brain/.../transcript.jsonl",
  "artifactDirectoryPath": "~/.gemini/antigravity/brain/..."
}
```

**stdout:**

| フィールド | 型 | 説明 |
|------------|-----|------|
| `decision` | string | **必須。** `"allow"` / `"deny"` / `"ask"` / `"force_ask"` |
| `reason` | string | 任意。決定理由（エージェント/ユーザー向け） |
| `permissionOverrides` | string[] | 任意。例: `["command(npm test)"]` |

```json
{
  "decision": "ask",
  "reason": "Requires confirmation for test execution.",
  "permissionOverrides": ["command(npm test)"]
}
```

> Issue #399 提案の `allow` / `deny` / `ask` に加え、公式は `"force_ask"`（キャッシュ済み Always Allow を無視して必ず確認）も定義する。

#### PostToolUse

**stdin:** `stepIdx`, `error`（成功時は空文字）, 共通フィールド。実機では `toolCall` が付与される場合あり。  
**stdout:** `{}`（空オブジェクト）。

#### PreInvocation

**stdin:** `invocationNum`（0 始まり）, `initialNumSteps`, 共通フィールド。  
**stdout:** 任意で `injectSteps`（`toolCall` / `userMessage` / `ephemeralMessage` のいずれか）。

#### PostInvocation

**stdin:** PreInvocation と同型。  
**stdout:** `injectSteps`（任意）, `terminationBehavior`（`""` / `"force_continue"` / `"terminate"`）。

#### Stop

**stdin:** `executionNum`, `terminationReason`（例: `model_stop`, `max_steps_exceeded`, `error`）, `error`, `fullyIdle`（必須 boolean）, 共通フィールド。  
**stdout:**

```json
{
  "decision": "continue",
  "reason": "Not done yet"
}
```

- `decision: "continue"` → 停止を阻止してループ再突入。`reason` はシステムメッセージとして注入。
- それ以外 → 停止を許可。

#### Claude Code との I/O 差分（ラッパー必須箇所）

| 項目 | Claude Code | Antigravity | 変換 |
|------|-------------|-------------|------|
| ツール名 | `tool_name` (PascalCase) | `toolCall.name` (snake_case 系) | リマップ |
| ツール引数 | `tool_input` オブジェクト | `toolCall.args`（キーは PascalCase 寄り） | 構造変換 |
| セッション ID | `session_id` | `conversationId` | リネーム |
| cwd | `cwd` | `workspacePaths[0]` 等 | 抽出 |
| PreToolUse 許可 | `hookSpecificOutput.permissionDecision` (`allow`/`deny`) | トップレベル `decision`（+ `ask`/`force_ask`） | アンラップ + 値域拡張 |
| Stop 阻止 | `decision: "block"` | `decision: "continue"` | **値の意味が逆方向に見える**ため要注意 |
| exit code 2 | ブロック | 公式は JSON `decision` 中心。非 0 exit の扱いは実装時に再確認 | ラッパーで JSON へ正規化推奨 |

### 10.6 Claude Code 形式からの変換チェックリスト（#309）

1. **構造:** `hooks.<Event>` → `{ "<plugin-or-hook-name>": { <Event>: ... } }`
2. **非 ToolUse:** matcher グループをフラット handlers に展開する（ラップし直さない）
3. **イベント:** 直接対応 3 種 + 近似 2 種。それ以外は警告除外
4. **ツール名:** matcher とラッパー内の両方で Antigravity 名へ変換
5. **ハンドラ種別:** `command` 以外は除外またはスタブ方針を決める（公式は command のみ）
6. **timeout:** Claude の 600 デフォルトをそのまま持ち込まない（Antigravity 既定 30）
7. **単一ファイル配置:** Personal / Project とも 1 つの `hooks.json`。複数プラグイン時のマージ / 所有権は Codex/Cursor と同様の未解決課題
8. **パス:** CLI では相対パスが起動 cwd 依存で失敗しうるため、配置時に絶対パスへ正規化する方針を検討
9. **旧二次情報の排除:** `allow_tool` / `deny_reason` や `SessionStart` 等の旧イベント一覧は公式と不一致。実装は本セクション（公式）に従う

### 10.7 PLM 実装メモ

| 項目 | 状態 |
|------|------|
| `AntigravityTarget::supported_components` に `Hook` | ✅ |
| EventMap / KeyMap / StructureConverter / ToolMap | ✅ |
| `create_layers` への登録 | ✅ |
| `.agents/hooks.json` / `~/.gemini/config/hooks.json` 配置 | ✅（単一ファイル・上書き/複数 Hook 拒否） |
| stdin/stdout ラッパースクリプト | ❌（command はインライン保持。I/O 差分は利用側で吸収） |

実装: [#309](https://github.com/DIO0550/plugin-manager/issues/309)。スキーマ整理: [#399](https://github.com/DIO0550/plugin-manager/issues/399)。
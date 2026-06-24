# Claude Code → 各ターゲット（Copilot CLI / Codex CLI / Antigravity）Hooks スキーマ対応表

PLM でフック変換ツールを実装する際のリファレンス。
Claude Code を入力元とし、各ターゲット環境（GitHub Copilot CLI / OpenAI Codex CLI / Google Antigravity）への
変換仕様を対比し、変換時の注意点をまとめる。

各セクションでは **公式仕様（最新）** と **PLM 実装状況** を区別して記載する。
実装状況は囲み引用または「実装メモ」ラベルで示す。

> **注記:** Codex / Antigravity の公式ドメイン（developers.openai.com, antigravity.google）は
> bot 対策により直接フェッチが 403 を返すため、検索スニペット・GitHub Issue・公式フォーラム・
> SDK README 等でクロスチェックした内容を記載している。記載は出典 URL を併記し、
> 最新の公式ドキュメントで確認のうえ利用すること。

## 公式ドキュメント

| 環境 | URL |
|------|-----|
| Claude Code Hooks | https://docs.anthropic.com/en/docs/claude-code/hooks |
| Claude Code Plugins | https://docs.anthropic.com/en/docs/claude-code/plugins |
| Copilot CLI Hooks 設定 | https://docs.github.com/en/copilot/reference/hooks-configuration |
| Copilot CLI Hooks ガイド | https://docs.github.com/en/copilot/how-tos/copilot-cli/customize-copilot/use-hooks |
| Copilot CLI Hooks チュートリアル | https://docs.github.com/en/copilot/tutorials/copilot-cli-hooks |
| Codex Hooks | https://developers.openai.com/codex/hooks |
| Codex Config（Advanced） | https://developers.openai.com/codex/config-advanced |
| Antigravity Hooks | https://antigravity.google/docs/hooks |
| Antigravity SDK（Python, hooks/README） | https://github.com/google-antigravity/antigravity-sdk-python |
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

### 構造差分

| 項目 | Claude Code | Copilot CLI |
|------|-------------|-------------|
| トップレベル `version` | なし | `1`（必須） |
| ネスト深度 | event → matcher group → hooks[] | event → hooks[]（フラット） |
| matcher | matcher group の `"matcher"` キー（regex） | なし（スクリプト内で `toolName` を判定） |
| コマンドキー | `"command"` | `"bash"` / `"powershell"` |
| タイムアウト | `"timeout"`（秒、デフォルト 600） | `"timeoutSec"`（秒、デフォルト 30） |
| 作業ディレクトリ | なし（`CLAUDE_PROJECT_DIR` で代替） | `"cwd"` |
| 環境変数 | なし（個別に `CLAUDE_*` が設定される） | `"env"` オブジェクト |
| フック種別 | `command` / `http` / `prompt` / `agent` | `command` / `prompt`（sessionStart のみ） |
| 無効化 | `"disableAllHooks": true` | なし |
| 非同期実行 | `"async": true` | なし |

**変換時の注意:**
- Claude Code の matcher グループ構造を Copilot CLI のフラット構造に展開する必要がある
- Claude Code の `http` / `agent` フックは Copilot CLI に直接変換できない（`command` ラッパーが必要）
- Copilot CLI の `powershell` キーは Claude Code に対応がない

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

### 3 ターゲット横断のイベント対応表

Codex CLI と Antigravity は Claude Code と同じ **PascalCase** のイベント名を採用しているため、
イベント名のケース変換は不要（Copilot CLI のみ camelCase）。
○=サポート、×=未対応、（）内はターゲット固有の差異。

| Claude Code | Copilot CLI | Codex CLI | Antigravity |
|-------------|-------------|-----------|-------------|
| `SessionStart` | `sessionStart` | `SessionStart` | `SessionStart` |
| `SessionEnd` | `sessionEnd` | ×（公式イベント外） | `SessionEnd` |
| `PreToolUse` | `preToolUse` | `PreToolUse` | `PreToolUse` |
| `PostToolUse` | `postToolUse` | `PostToolUse` | `PostToolUse` |
| `UserPromptSubmit` | `userPromptSubmitted` | `UserPromptSubmit` | `UserPromptSubmit` |
| `Stop` | `agentStop` | `Stop` | `Stop` |
| `SubagentStop` | `subagentStop` | `SubagentStop` | `SubagentStop` |
| `SubagentStart` | × | `SubagentStart` | × |
| `PermissionRequest` | （`preToolUse` で代替） | `PermissionRequest` | × |
| `PreCompact` | × | `PreCompact` | `PreCompact` |
| `PostCompact` | × | `PostCompact` | × |
| `Notification` | × | ×（別系統の `notify` 設定あり） | `Notification` |

> **注:** 上表は各環境の **公式仕様** に基づくイベント名の対応であり、PLM の実装可否とは別。
> PLM が現状変換できるイベントは各ターゲットの「実装メモ」を参照（Codex は 6 イベントのみ、
> Antigravity は未対応）。

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

| 種別 | Claude Code | Copilot CLI |
|------|-------------|-------------|
| `command` | 全イベントで使用可。シェルコマンドを実行 | 全イベントで使用可 |
| `http` | HTTP POST でウェブフックを呼び出し。`headers` で `$VAR` 展開可能 | **なし** |
| `prompt` | `$ARGUMENTS` に入力 JSON を展開し LLM に評価させる。`{ok, reason}` で応答 | `sessionStart` のみ。テキストを自動送信 |
| `agent` | サブエージェント（Read/Grep/Glob 使用可、最大50ターン）で調査。`{ok, reason}` で応答 | **なし** |

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

### Copilot CLI → Claude Code

1. `"version"` フィールドを除去
2. イベント名を camelCase → PascalCase に変換
3. フラット配列を matcher グループ構造にラップ（matcher なしの場合は `"matcher": ""` で全マッチ）
4. `"bash"` → `"command"` にキー名変更（`"powershell"` は除外または警告）
5. `"timeoutSec"` → `"timeout"` にキー名変更
6. `"cwd"` / `"env"` は Claude Code に対応がないため除外または警告
7. `errorOccurred` は除外

---

## 10. Codex CLI（OpenAI Codex）

出典: https://developers.openai.com/codex/hooks 、 https://developers.openai.com/codex/config-advanced
（公式ドメインは 403 のため検索スニペット・関連 Issue 等でクロスチェック）

Codex CLI の hooks は Claude Code と **同形のスキーマ**（PascalCase イベント名 + matcher グループ構造）を採用しており、
変換時の差分は Copilot CLI より小さい。

### 10.1 設定ファイル構造（公式仕様）

設定方法は2通りあり、いずれも同一のイベントスキーマを共有する。

**(a) `hooks.json`（JSON）:**

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
            "timeout": 30,
            "statusMessage": "Processing...",
            "command_windows": "<Windows 用コマンド>"
          }
        ]
      }
    ]
  }
}
```

**(b) `config.toml` 内のインライン `[hooks]` テーブル（TOML）:**

`~/.codex/config.toml` などアクティブな config レイヤーに直接記述する形式。JSON 版と同じ構造を TOML で表現する。

**有効化（feature flag）:** `config.toml` で次のフラグが必要。

```toml
[features]
codex_hooks = true
```

（`features.codex_hooks` は新方式への deprecated alias とされる。）

**配置場所:**
- `~/.codex/config.toml`（および `hooks.json`）— ユーザー/アクティブ config レイヤー
- プラグインは plugin manifest または `hooks/hooks.json` でライフサイクル設定をバンドル可能

### 10.2 サポートイベント（公式仕様、10種）

Claude Code と同じ PascalCase イベント名を保持する。`scope` はイベントの適用範囲を示す。

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

### 10.3 ハンドラとフィールド（公式仕様）

- ハンドラは `type: "command"` のみ実行される。`prompt` / `agent` ハンドラは**パースされるがスキップ**される。
- フィールド:

| フィールド | 説明 |
|-----------|------|
| `matcher` | 正規表現。Claude Code と同じく matcher グループ単位 |
| `type` | `"command"`（command のみ実行） |
| `command` | 実行するシェルコマンド |
| `timeout` | タイムアウト（秒） |
| `statusMessage` | 実行中に表示するステータスメッセージ |
| `command_windows` / `commandWindows` | Windows 用コマンド（OS 別の実行コマンド指定） |

> **補足:** Codex には hooks とは別に `notify` 設定（`agent-turn-complete` のみ）が存在し、
> デスクトップ通知/webhook 用途で使われる。これは hooks スキーマとは別物。

### 10.4 Claude Code → Codex 変換のポイント

- イベント名は **PascalCase のまま保持**（ケース変換不要）。matcher グループ構造も保持する。
- トップレベルの `version` / `disableAllHooks` は削除する。
- `timeout` はそのまま保持する。Copilot 形式の `timeoutSec` は `timeout` に正規化（重複時は `timeout` を優先）。
- Copilot 形式の `comment` は `statusMessage` に変換する。
- `async` / `once` / `bash` は削除する（必要に応じて警告）。
- command hook には `"type": "command"` を補う。
- Codex に対応のないイベント（例: `SessionEnd`, `Notification`）は除外する。

> ### 実装メモ（PLM 現状）
>
> 関連コード: `src/hooks/converter/codex.rs`, `src/hooks/event/codex.rs`, `src/hooks/tool/codex.rs`,
> `src/target/env/codex.rs`
>
> - Codex は Hook コンポーネントを **サポート済み**（`supported_components` に `Hook` を含む）。
> - 配置先は `.codex/hooks.json`（personal/project とも）。**1 スコープにつき単一 `hooks.json` のみ**で、
>   **複数 Hook コンポーネントのマージは未実装**（複数配置は conflict エラー）。
> - 対応イベントは **6 種のみ**:
>   `SessionStart`, `PreToolUse`, `PostToolUse`, `UserPromptSubmit`, `Stop`, `PermissionRequest`。
>   → 公式にある **`PreCompact` / `PostCompact` / `SubagentStop` / `SubagentStart` は未対応**（変換時に除外）。
> - StructureConverter: 常に Claude Code 形式とみなし、`version` / `disableAllHooks` を削除。matcher グループは保持。
> - KeyMap: `timeout` 保持、`timeoutSec`→`timeout` 正規化、`comment`→`statusMessage`、
>   `async` / `once` / `bash` は削除し警告、command hook に `"type": "command"` を補う。
> - ScriptGenerator: **スクリプト生成なし**（JSON inline をそのまま保持）。
>   `http` / `agent` / `prompt` ハンドラの扱いは未対応。
> - ToolMap: pass-through（trim のみ）。
> - **`config.toml` 形式の入出力は未対応**（JSON のみ）。
> - **`command_windows` / `commandWindows` は未対応**。
> - **feature flag（`[features] codex_hooks = true`）の案内/自動設定は未対応**。

---

## 11. Google Antigravity

出典: https://antigravity.google/docs/hooks 、
Antigravity SDK（Python, hooks/README）https://github.com/google-antigravity/antigravity-sdk-python 、
フォーラム https://discuss.ai.google.dev/t/hooks-in-antigravity/120458
（公式ドメインは 403 のため SDK README・フォーラム・検索スニペットでクロスチェック）

Antigravity 2.0 は event-driven automation の hooks を主要機能の一つとして導入した。
スキーマは Claude Code に酷似するが、**トップレベルの構造が異なる**点に注意。

### 11.1 設定ファイル構造（公式仕様）

フォーマットは `hooks.json`（JSON）。トップレベルは「**フック名 → イベント設定**」のマップ構造で、
Claude Code（直接 `hooks.<Event>`）とはこの一段が異なる。

```json
{
  "my-linter-hook": {
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

| 項目 | Claude Code | Antigravity |
|------|-------------|-------------|
| トップレベル | `hooks.<Event>` | `<フック名>.<Event>`（フック名のマップが1段入る） |
| イベント名 | PascalCase | PascalCase（同形） |
| matcher グループ | `matcher`(regex) + `hooks[]` | 同形 |
| ハンドラ | `type` / `command` / `timeout` | 同形 |
| 個別無効化 | `disableAllHooks`（全体） | `enabled: false`（個別フック単位） |

**配置場所:**
- プロジェクト（workspace）スコープ: `.agents/` ディレクトリ
- グローバル（personal）スコープ: `~/.gemini/config/hooks.json`

### 11.2 サポートイベント（公式仕様、確認できた範囲）

`PreToolUse`, `PostToolUse`, `Stop`, `SubagentStop`, `SessionStart`, `SessionEnd`,
`UserPromptSubmit`, `PreCompact`, `Notification`

### 11.3 I/O とアーキタイプ（公式仕様）

- **stdin:** JSON を受け取る。主なフィールドは `toolCall.args`, `workspacePaths`,
  `transcriptPath`, `artifactDirectoryPath` 等。
- **stdout:** JSON を返却し、`allow` / `deny` / `ask` で判定を返す。
- `enabled: false` を設定すると個別フックを無効化できる。
- hooks の 3 アーキタイプ（SDK 概念）:

| アーキタイプ | 特性 |
|-------------|------|
| Decide | read-only / blocking（許可・拒否を判定） |
| Inspect | read-only / non-blocking（観測のみ） |
| Transform | modifying / blocking（入力を変更しつつブロック可能） |

> ### 実装メモ（PLM 現状）
>
> 関連コード: `src/target/env/antigravity.rs`, `src/hooks/converter`（`create_layers`）
>
> - Antigravity は Hook コンポーネント **非対応**（`supported_components` は `[Skill]` のみ）。
> - hooks converter も **未実装**。`create_layers` で Codex / Copilot 以外は
>   "Hook conversion is not yet implemented for target" エラーになる。
> - → **Antigravity 向け hooks 変換は完全に未実装**。公式が hooks をサポートし始めたため、今後の対応が必要。

---

## 12. Gemini CLI（参考）

- Gemini CLI 単体の hooks 公式仕様は確認できなかった（Antigravity 経由が実態の可能性）。
- PLM では Gemini CLI は Hook **非対応**（`supported_components` は `[Skill, Instruction]`）。
- → 現状「公式仕様未確認 / 非対応」として扱う。

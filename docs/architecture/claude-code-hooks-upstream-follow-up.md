# Claude Code Hooks 上流仕様追随方針（Issue #462）

> **状態:** 調査完了・実装方針確定
> **対象 Issue:** [#462](https://github.com/DIO0550/plugin-manager/issues/462)
> **上流確認日:** 2026-09-01
> **対象経路:** Claude Code プラグインの `hooks/hooks.json`
> **公式仕様:** [Claude Code Hooks reference](https://code.claude.com/docs/en/hooks)

## 結論

Claude Code が公式に列挙する 33 イベントをすべて `HookEvent` の既知バリアントにする。
ターゲットが実行できるイベントだけを各 `EventBridge` に登録し、既知だが非対応のイベントと、
将来追加された未知イベントを診断上区別する。

`mcp_tool` は共通モデルへ追加する。Codex は同じ `server` / `tool` / `input` 形を公式に
サポートするため、対応イベントでは inline のまま保持する。Copilot CLI、Cursor、Antigravity
では接続済み MCP セッションを同じ意味で呼び出す hook type がないため、command への推測変換は
行わず、定義単位で除外して警告する。

`args`、`if`、`asyncRewake` のように削除すると実行条件やプロセス起動方式が変わるフィールドは、
フィールドだけを黙って落とさない。安全に等価変換できないターゲットでは handler 全体を除外する。
表示だけに影響する `statusMessage` はフィールド単位で除外できる。

Skill / Subagent frontmatter の `hooks` は供給元と有効範囲が異なるため本 Issue へ含めない。
[独立機能 Issue 定義](./skill-subagent-frontmatter-hooks-issue.md)を正とする。

## 1. イベント棚卸し

### 1.1 既存 10 イベント

`HookEvent` が既に知っているイベントの変換方針は次のとおり。`〜` は PLM が既に採用している
近似変換で、発火頻度や入力スキーマが同一ではない。

| Claude Code | Codex | Copilot CLI | Cursor | Antigravity |
|---|---|---|---|---|
| `SessionStart` | `SessionStart` | `sessionStart` | `sessionStart` | 〜 `PreInvocation` |
| `SessionEnd` | `SessionEnd` | `sessionEnd` | `sessionEnd` | 〜 `Stop` |
| `PreToolUse` | `PreToolUse` | `preToolUse` | `preToolUse` | `PreToolUse` |
| `PostToolUse` | `PostToolUse` | `postToolUse` | `postToolUse` | `PostToolUse` |
| `PostToolUseFailure` | × | `postToolUseFailure` | `postToolUseFailure` | × |
| `UserPromptSubmit` | `UserPromptSubmit` | `userPromptSubmitted` | `beforeSubmitPrompt` | 〜 `PreInvocation` |
| `Stop` | `Stop` | `agentStop` | `stop` | `Stop` |
| `SubagentStart` | `SubagentStart` | `subagentStart` | `subagentStart` | × |
| `SubagentStop` | `SubagentStop` | `subagentStop` | `subagentStop` | × |
| `PreCompact` | `PreCompact` | `preCompact` | `preCompact` | × |

### 1.2 追加する 23 イベント

2026-09-01 時点の Claude Code 公式仕様には、現行 `HookEvent` にない 23 イベントがある。
Issue 起票後に `PreModelSwitch` / `PostModelSwitch` も追加されたため、今回の棚卸しへ含める。

| 分類 | Claude Code | Codex | Copilot CLI | Cursor | Antigravity | 方針 |
|---|---|---|---|---|---|---|
| セットアップ | `Setup` | × | × | × | × | 既知・非対応 |
| プロンプト | `UserPromptExpansion` | × | × | × | × | `userPromptTransformed` は発火条件と制御能力が異なるため近似しない |
| 権限 | `PermissionRequest` | `PermissionRequest` | `permissionRequest` | × | × | 2 ターゲットへ直接変換 |
| 権限 | `PermissionDenied` | × | × | × | × | 既知・非対応 |
| ツール | `PostToolBatch` | × | × | × | × | 既知・非対応 |
| 表示 | `Notification` | × | `notification` | × | × | Copilot CLI のみ直接変換 |
| 表示 | `MessageDisplay` | × | × | × | × | Cursor の response/thought hook は発火単位が異なるため近似しない |
| タスク | `TaskCreated` | × | × | × | × | 既知・非対応 |
| タスク | `TaskCompleted` | × | × | × | × | 既知・非対応 |
| ターン | `StopFailure` | × | × | × | × | `errorOccurred` は任意の実行時エラーを含むため近似しない |
| チーム | `TeammateIdle` | × | × | × | × | 既知・非対応 |
| 環境 | `InstructionsLoaded` | × | × | × | × | 既知・非対応 |
| 環境 | `ConfigChange` | × | × | × | × | 既知・非対応 |
| 環境 | `CwdChanged` | × | × | × | × | 既知・非対応 |
| 環境 | `DirectoryAdded` | × | × | × | × | Cursor `workspaceOpen` と lifecycle が異なるため近似しない |
| 環境 | `FileChanged` | × | × | × | × | 既知・非対応 |
| worktree | `WorktreeCreate` | × | × | × | × | 既知・非対応 |
| worktree | `WorktreeRemove` | × | × | × | × | 既知・非対応 |
| 圧縮 | `PostCompact` | `PostCompact` | × | × | × | Codex のみ直接変換 |
| モデル | `PreModelSwitch` | × | × | × | × | 既知・非対応 |
| モデル | `PostModelSwitch` | × | × | × | × | 既知・非対応 |
| MCP | `Elicitation` | × | × | × | × | 既知・非対応 |
| MCP | `ElicitationResult` | × | × | × | × | 既知・非対応 |

### 1.3 モデルと診断

- `HookEvent` へ 33 イベントすべてのバリアントを追加する。`Other(String)` は公式仕様にない
  将来イベントだけに使う。
- `CodexEventMap` の文字列直マッチを廃止し、他ターゲットと同じ `EventBridge` と
  `HookEvent::from_str` を通す。これにより `PermissionRequest` / `PostCompact` も共通モデル上で
  `Other` にならない。
- 既知だがターゲット非対応なら `UnsupportedEvent`、`Other` なら
  `UnknownSourceEvent` を出す。どちらも元のイベント名を保持する。
- 近似マッピングは上表で明示した既存 3 経路だけに限定し、名前や発火タイミングが似ているだけの
  イベントを自動変換しない。

## 2. `mcp_tool` の扱い

共通モデルへ次の定義を追加する。

| フィールド | 必須 | 型 | 方針 |
|---|---|---|---|
| `type` | ✅ | string | `mcp_tool` |
| `server` | ✅ | string | 空文字を拒否 |
| `tool` | ✅ | string | 空文字を拒否 |
| `input` | — | object | 省略時 `{}`。文字列内の `${field.path}` は target runtime に委ねる |
| `timeout` | — | number | 共通フィールドとして検証 |
| `statusMessage` | — | string | 対応ターゲットだけ保持 |

| ターゲット | 変換方針 |
|---|---|
| Codex | 対応イベントでは inline 保持。`SessionEnd` は Codex が `mcp_tool` を受理しないため type/event 非対応警告で除外 |
| Copilot CLI | 除外 + `UnsupportedHookType`。MCP connection / OAuth / tool invocation を shell command へ推測変換しない |
| Cursor | 除外 + `UnsupportedHookType`。`beforeMCPExecution` は Cursor 自身の MCP 呼び出しを監視するイベントであり、MCP tool を実行する hook type ではない |
| Antigravity | 除外 + `UnsupportedHookType`。公式の handler type は `command` のみ |

イベント自体が非対応の場合は `UnsupportedEvent` を 1 件出し、その配下の各 type 警告は重ねない。
対応イベント内で `mcp_tool` だけが非対応の場合に `UnsupportedHookType` を handler ごとに出す。

## 3. 新フィールドの変換方針

凡例: `保持` = 同じ意味で出力、`変換` = target 名・形へ等価変換、`除外` = handler 全体を除外、
`削除` = handler は残しフィールドだけ警告付きで除去。

| Claude Code フィールド | Codex | Copilot CLI | Cursor | Antigravity | 判断理由 |
|---|---|---|---|---|---|
| `args` | 除外 | 除外 | 除外 | 除外 | 全ターゲットに exec-form argv の同等フィールドがない。削除すると shell form へ意味が変わる |
| `shell` | 除外 | `bash` / `powershell` へ変換 | 除外 | 除外 | Copilot CLI だけ OS 別コマンド欄で意図を保持できる |
| `if` | 除外 | 除外 | 除外 | 除外 | permission rule は matcher と同値でない。削除すると本来対象外の tool call に hook が走る |
| `asyncRewake` | 除外 | 除外 | 除外 | 除外 | background 終了時に agent を即時再開する同等機能がない。Codex `async` への縮退も主目的を失う |
| `allowedEnvVars` | HTTP handler ごと除外 | native HTTP handler で保持 | HTTP handler ごと除外 | HTTP handler ごと除外 | Copilot CLI は同名・同目的の whitelist を持つ |
| `statusMessage` | 保持 | 削除 | 削除 | 削除 | 表示専用なので削除しても hook の制御意味は変わらない。Copilot の `comment` への変換は公式根拠がないため廃止 |
| `once` | 削除 | 削除 | 削除 | 削除 | `hooks/hooks.json` では Claude Code 自身が無視する。Skill frontmatter の扱いは別 Issue |
| `async` | 保持 | 除外 | 除外 | 除外 | field-only drop では同期化され、待機と出力タイミングが変わる |
| `timeout` | 保持 | `timeoutSec` へ変換 | 保持 | 保持 | 全ターゲットが秒単位 timeout を持つ |

`args` / `shell` / `if` / `asyncRewake` は hook type ごとの許可フィールドとして検証する。
未知キーを target 出力へそのままコピーする catch-all は使わない。各 type は allowlist で出力を組み立て、
既知の非対応フィールドには理由付き警告、未知フィールドには `UnknownHookField` を出す。

## 4. プレースホルダ

| プレースホルダ | Codex | Copilot CLI | Cursor | Antigravity |
|---|---|---|---|---|
| `${CLAUDE_PROJECT_DIR}` | 公式に互換変数の記載がないため bridge が必要 | wrapper の stdin `cwd` から export | target 固有 wrapper が必要 | target 固有 wrapper が必要 |
| `${CLAUDE_PLUGIN_ROOT}` | plugin runtime が互換変数を設定 | 配置時に実パスへ bridge | target 固有 wrapper が必要 | target 固有 wrapper が必要 |
| `${CLAUDE_PLUGIN_DATA}` | Codex が互換変数を設定するため保持 | 除外 | 除外 | 除外 |

`${CLAUDE_PLUGIN_DATA}` を plugin root や一時ディレクトリへ置き換えない。更新をまたいで残る writable
directory という契約を保てないため、Codex 以外では placeholder を含む handler を除外し、
`UnsupportedPlaceholder` に対象名を含める。将来 PLM が target 共通の永続 data directory を所有した
場合にだけ bridge を追加する。

## 5. frontmatter hooks の境界

Skill / Subagent frontmatter hooks は解析経路、component scope、`once` の意味が `hooks/hooks.json`
と異なる。今回の共通イベント/type/field モデルは再利用できる形にするが、次は本 Issue に含めない。

- `SKILL.md` / agent Markdown の YAML 解析
- component-local な配置と有効期間
- `once` の保持
- hooks.json と frontmatter の順序・重複・競合

詳細と受け入れ条件は
[Skill / Subagent frontmatter hooks 供給経路](./skill-subagent-frontmatter-hooks-issue.md)に集約する。

## 6. 実装時の受け入れ条件

- [ ] 公式 33 イベントが `HookEvent` の専用バリアントへ parse され、未知文字列だけが `Other` になる。
- [ ] Codex / Copilot / Cursor / Antigravity の対応表を table-driven test で固定する。
- [ ] `PermissionRequest` は Codex と Copilot、`Notification` は Copilot、`PostCompact` は Codex へ変換される。
- [ ] `mcp_tool` は Codex 対応イベントで `server` / `tool` / `input` を失わず、他ターゲットでは handler 単位の警告になる。
- [ ] Codex `SessionEnd` + `mcp_tool` が type/event 非対応として除外される。
- [ ] `args` / `if` を持つ command handler が、フィールドだけを落として実行範囲や argv を変えない。
- [ ] Copilot の `statusMessage` が未定義の `comment` へ変換されず、警告付きで削除される。
- [ ] `${CLAUDE_PLUGIN_DATA}` は Codex だけで保持され、他ターゲットで plugin root へ誤変換されない。
- [ ] frontmatter hooks を入力したテストは別 Issue の suite に置き、本 Issue の converter test と混在させない。

## 7. 参照した公式仕様

- [Claude Code Hooks reference](https://code.claude.com/docs/en/hooks)
- [Codex Hooks](https://learn.chatgpt.com/docs/hooks)
- [GitHub Copilot hooks reference](https://docs.github.com/en/copilot/reference/hooks-reference)
- [VS Code Agent hooks](https://code.visualstudio.com/docs/agent-customization/hooks)
- [Cursor Hooks](https://cursor.com/docs/hooks)
- [Google Antigravity Hooks](https://antigravity.google/docs/hooks)

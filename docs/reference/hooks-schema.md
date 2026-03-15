# Claude Code ↔ Copilot CLI Hooks スキーマ対応表

変換ツール実装のためのリファレンスです。Claude Code Hooks と GitHub Copilot CLI / coding agent Hooks は似た構造を持ちますが、イベント名、設定キー、入出力 JSON に差分があります。

## 公式ドキュメント

- [Claude Code Hooks リファレンス](https://code.claude.com/docs/en/hooks)
- [Claude Code Hooks ガイド](https://code.claude.com/docs/en/hooks-guide)
- [Claude Code プラグインリファレンス](https://code.claude.com/docs/en/plugins-reference)
- [Copilot CLI Hooks 設定リファレンス](https://docs.github.com/en/copilot/reference/hooks-configuration)
- [Copilot CLI Hooks の使い方](https://docs.github.com/en/copilot/how-tos/copilot-cli/customize-copilot/use-hooks)
- [Copilot CLI プラグインリファレンス](https://docs.github.com/en/copilot/reference/copilot-cli-reference/cli-plugin-reference)

---

## 1. 設定ファイル構造

### Claude Code

```json
{
  "hooks": {
    "PreToolUse": [
      {
        "type": "command",
        "command": "./scripts/validate.sh",
        "timeout": 15
      }
    ]
  }
}
```

- `version` フィールドは不要
- イベント名は PascalCase
- コマンド指定は `command` / `windows` / `linux` / `osx`

### Copilot CLI / coding agent

```json
{
  "version": 1,
  "hooks": {
    "preToolUse": [
      {
        "type": "command",
        "bash": "./scripts/validate.sh",
        "powershell": "./scripts/validate.ps1",
        "timeoutSec": 15
      }
    ]
  }
}
```

- `version: 1` が必須
- イベント名は camelCase
- コマンド指定は `bash` / `powershell`

---

## 2. トップレベルフィールド対応

| Claude Code | Copilot CLI | 変換ルール |
|-------------|-------------|------------|
| なし | `version: 1` | Claude → Copilot では追加、Copilot → Claude では除去 |
| `hooks` | `hooks` | 同名 |
| `hooks.<PascalCaseEvent>` | `hooks.<camelCaseEvent>` | イベント名を変換 |

---

## 3. イベント名対応

| Claude Code | Copilot CLI | 対応状況 | 備考 |
|-------------|-------------|----------|------|
| `PreToolUse` | `preToolUse` | 直接対応 | 実行前の許可/拒否判定 |
| `PostToolUse` | `postToolUse` | 直接対応 | 実行結果の後処理 |
| `SessionStart` | `sessionStart` | 直接対応 | セッション開始時 |
| `Stop` | `sessionEnd` | 概念対応 | 終了理由フィールドは Copilot 側で明示される |
| `UserPromptSubmit` | `userPromptSubmitted` | 直接対応 | 命名差分のみ |
| `PreCompact` | なし | 非対応 | Claude Code 固有 |
| `SubagentStart` | なし | 非対応 | Claude Code 固有 |
| `SubagentStop` | なし | 非対応 | Claude Code 固有 |
| なし | `errorOccurred` | 非対応 | Copilot 固有 |

> 変換ツールでは、直接対応しないイベントを無理に変換せず、警告付きでスキップする方が安全です。

---

## 4. Hook 定義オブジェクトのフィールド対応

| Claude Code | Copilot CLI | 変換ルール |
|-------------|-------------|------------|
| `type` | `type` | そのまま |
| `command` | `bash` / `powershell` | シェル別に分解する |
| `windows` | `powershell` | Windows 用コマンドを PowerShell 側へ |
| `linux` / `osx` | `bash` | Unix 系コマンドは bash 側へ統合 |
| `timeout` | `timeoutSec` | キー名変更 |
| なし | `cwd` | Copilot 固有。Claude → Copilot 時は必要に応じて追加 |
| なし | `env` | Copilot 固有。変換元に情報がなければ生成しない |
| なし | `comment` | Copilot 固有。説明用の任意フィールド |

### 実装メモ

- Claude Code の `command` が単一文字列しか持たない場合は、Copilot では `bash` に割り当てるのが最小変換です
- Claude Code の OS 別設定を持つ場合は、Windows を `powershell`、それ以外を `bash` に寄せます
- Copilot の `bash` と `powershell` が両方ある構成を Claude に戻す場合、単一の `command` へ完全には畳み込めません

---

## 5. Plugin manifest 上の Hooks 指定

| 項目 | Claude Code | Copilot CLI |
|------|-------------|-------------|
| manifest | `.claude-plugin/plugin.json` | `plugin.json`, `.github/plugin/plugin.json`, `.claude-plugin/plugin.json` |
| hooks 指定 | `hooks` でパス参照 | `hooks` にパスまたは inline object を指定可能 |
| 既定の hooks パス | `hooks/hooks.json` など plugin 側の定義に従う | `hooks.json` または `hooks/hooks.json` |

Copilot CLI は manifest の `hooks` にファイルパスだけでなく inline object も許容します。Claude Code 由来の変換では、まず `hooks.json` を生成してパス参照に統一すると扱いやすくなります。

---

## 6. 入力 JSON の考え方

両者とも hook スクリプトへは stdin で JSON を渡しますが、イベントごとの payload は同一ではありません。変換ツールは「設定ファイルのキー変換」と「実際の hook スクリプトが参照する入出力 JSON」の差分を分けて扱う必要があります。

### Copilot CLI の代表例

#### `preToolUse`

```json
{
  "timestamp": 1704614600000,
  "cwd": "/path/to/project",
  "toolName": "bash",
  "toolArgs": "{\"command\":\"rm -rf dist\"}"
}
```

#### `postToolUse`

```json
{
  "timestamp": 1704614700000,
  "cwd": "/path/to/project",
  "toolName": "bash",
  "toolArgs": "{\"command\":\"npm test\"}",
  "toolResult": {
    "resultType": "success",
    "textResultForLlm": "All tests passed"
  }
}
```

### Claude Code との扱い

- Claude Code でも stdin / stdout の JSON プロトコルを使う
- ただし event 名や応答ラッパーが異なるため、スクリプト本体まで完全互換とはみなさない
- 既存スクリプトを流用する場合は、イベントごとの payload 差分を吸収するラッパースクリプトを挟むのが安全

---

## 7. 出力 JSON 対応

### Claude Code `PreToolUse` 系

```json
{
  "continue": true,
  "hookSpecificOutput": {
    "hookEventName": "PreToolUse",
    "permissionDecision": "allow",
    "permissionDecisionReason": "Validated tool input"
  }
}
```

### Copilot CLI `preToolUse`

```json
{
  "permissionDecision": "deny",
  "permissionDecisionReason": "Destructive operations require approval"
}
```

### 出力変換ルール

| Claude Code | Copilot CLI | 備考 |
|-------------|-------------|------|
| `hookSpecificOutput.permissionDecision` | `permissionDecision` | ラッパーを外す |
| `hookSpecificOutput.permissionDecisionReason` | `permissionDecisionReason` | ラッパーを外す |
| `continue` | なし | Copilot 側には直接対応なし |
| `hookSpecificOutput.hookEventName` | なし | Copilot 側では不要 |

> Copilot CLI の `preToolUse` は `deny` を実際に処理する前提で設計されているため、Claude Code 側の `allow` / `ask` 相当をそのまま移しても同等挙動になるとは限りません。

---

## 8. 推奨変換ルール

### Claude Code → Copilot CLI

1. ルートに `version: 1` を追加
2. イベント名を PascalCase から camelCase へ変換
3. `command` / OS 別キーを `bash` / `powershell` に再配置
4. `timeout` を `timeoutSec` に変換
5. `PreCompact` / `SubagentStart` / `SubagentStop` は警告付きでスキップ
6. Claude 固有の stdout ラッパーを Copilot 形式へ平坦化

### Copilot CLI → Claude Code

1. `version` を除去
2. イベント名を camelCase から PascalCase へ変換
3. `bash` / `powershell` を `command` ベースへ集約
4. `timeoutSec` を `timeout` へ変換
5. `errorOccurred` は警告付きでスキップ
6. `cwd` / `env` / `comment` は Claude に直接載せ替えられないため、必要なら補助スクリプトで吸収

---

## 9. 変換時に注意すべき非互換ポイント

- **イベント集合が一致しない**: Claude 固有イベントと Copilot 固有イベントがある
- **コマンド指定方法が違う**: Claude は OS 別キー、Copilot は shell 別キー
- **出力 JSON の形が違う**: Claude は `hookSpecificOutput` ラッパー付き
- **Copilot は `version` 必須**: 付与漏れで設定が無効になる
- **manifest の柔軟性が違う**: Copilot は inline object を許容する

このため、変換ツールは単純なキー置換ではなく、**イベント互換性チェック** と **入出力ラッパー変換** を必須処理として持つべきです。

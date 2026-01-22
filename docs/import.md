# plm import コマンド仕様

Claude Code Plugin形式のGitHubリポジトリから、特定のコンポーネントを選択してインポートする。

## 基本構文

```bash
plm import <source> [options]
```

## 引数

| 引数 | 説明 |
|------|------|
| `<source>` | GitHubリポジトリ（`owner/repo` または `owner/repo@ref`） |

## オプション

| オプション | 説明 |
|-----------|------|
| `--component <PATH>` | 特定のコンポーネントを指定（複数指定可） |
| `--type <TYPE>` | コンポーネント種別でフィルタ（複数指定可） |
| `--target <TARGET>` | デプロイ先ターゲット（未指定時はTUI選択） |
| `--scope <SCOPE>` | デプロイスコープ（未指定時はTUI選択） |
| `--force` | キャッシュを無視して再ダウンロード |

**注意**: `--component` と `--type` は同時に使用できません。

## コンポーネントパスの形式

`--component` で指定するパスは以下の形式：

```
<plural_kind>/<name>
```

### 有効な kind（複数形、大小文字不問）

| 指定値 | コンポーネント種別 |
|--------|-------------------|
| `skills` | Skill |
| `agents` | Agent |
| `commands` | Command |
| `instructions` | Instruction |
| `hooks` | Hook |

### 例

```bash
# 有効なパス
--component skills/pdf        # OK
--component SKILLS/pdf        # OK（kindは正規化される）
--component skills/PDF        # OK（nameは大小文字を保持）
--component skills/pdf/       # OK（末尾スラッシュは除去）

# 無効なパス
--component skill/pdf         # NG（単数形は不可）
--component skills/           # NG（nameが空）
--component skills/a/b        # NG（ネストパスは不可）
--component skills//pdf       # NG（連続スラッシュ）
```

### name の大小文字

- `name` は**大小文字を区別**する（case-sensitive）
- プラグインに `pdf` というスキルがある場合、`skills/PDF` では一致しない
- 一致しないパスは警告を表示してスキップ

## --type オプション

コンポーネント種別でフィルタリング：

| 値 | 対象 |
|----|------|
| `skill` | スキル |
| `agent` | エージェント |
| `command` | コマンド |
| `instruction` | インストラクション |
| `hook` | フック |

```bash
# スキルのみインポート
plm import owner/repo --type skill

# 複数種別を指定
plm import owner/repo --type skill --type agent
```

## ターゲットとスコープ

| オプション | 有効な値 |
|-----------|---------|
| `--target` | `codex`, `copilot` |
| `--scope` | `personal`, `project` |

未指定の場合は対話的に選択（TUI）。CI環境など非対話環境では必ず指定が必要。

## install との違い

| 観点 | install | import |
|------|---------|--------|
| 対象 | GitHub / Marketplace | Claude Code Plugin形式のみ |
| 選択単位 | プラグイン全体 | コンポーネント単位で選択可能 |
| フィルタ | `--type` のみ | `--type` または `--component` |
| 用途 | プラグイン一括インストール | 他プラグインから部分的に取り込み |

## インポート履歴

インポートしたコンポーネントは `~/.plm/imports.json` に記録される：

```json
{
  "imports": [
    {
      "source_repo": "owner/repo",
      "kind": "skill",
      "name": "pdf",
      "target": "codex",
      "scope": "personal",
      "path": "/home/user/.codex/skills/github/owner--repo/pdf",
      "imported_at": "2024-01-01T00:00:00Z",
      "git_ref": "main",
      "commit_sha": "abc123..."
    }
  ]
}
```

## エラーハンドリング

| 状況 | 動作 |
|------|------|
| `--type` と `--component` 両方指定 | エラー終了 |
| 不正なパス形式 | エラー終了 |
| リポジトリが存在しない | エラー終了 |
| plugin.json がない | エラー終了 |
| フィルタ結果が0件 | エラー終了 |
| 指定コンポーネントの一部が存在しない | 警告表示、他は継続 |

## 使用例

```bash
# 基本的な使い方
plm import DIO0550/sample-plugin

# 特定のスキルだけインポート
plm import DIO0550/sample-plugin --component skills/pdf

# 複数コンポーネントを指定
plm import DIO0550/sample-plugin \
  --component skills/pdf \
  --component agents/review

# エージェントとコマンドのみインポート
plm import DIO0550/sample-plugin --type agent --type command

# ターゲットとスコープを明示
plm import DIO0550/sample-plugin \
  --component skills/pdf \
  --target codex \
  --scope personal

# 特定ブランチからインポート
plm import DIO0550/sample-plugin@develop --type skill

# キャッシュを無視して再ダウンロード
plm import DIO0550/sample-plugin --force
```

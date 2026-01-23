# plm marketplace

マーケットプレイスを管理します。

## サブコマンド

| サブコマンド | 説明 |
|--------------|------|
| `list` | 登録済みマーケットプレイスの一覧表示 |
| `add` | マーケットプレイスを追加 |
| `remove` | マーケットプレイスを削除 |
| `update` | マーケットプレイス情報を更新 |
| `show` | マーケットプレイスの詳細を表示 |

## plm marketplace list

登録済みのマーケットプレイスを表示します。

```bash
$ plm marketplace list
┌──────────────────┬─────────────────────────┬─────────┬──────────────────┐
│ NAME             │ SOURCE                  │ PLUGINS │ LAST UPDATED     │
├──────────────────┼─────────────────────────┼─────────┼──────────────────┤
│ anthropic        │ anthropics/claude-code  │ 5       │ 2024-01-15 10:30 │
│ company-tools    │ company/claude-plugins  │ 3       │ 2024-01-14 15:20 │
│ uncached-mp      │ some/repo               │ N/A     │ Never            │
└──────────────────┴─────────────────────────┴─────────┴──────────────────┘
```

**キャッシュが無い場合**: `PLUGINS` は `N/A`、`LAST UPDATED` は `Never` と表示されます。

**マーケットプレイスが未登録の場合**:

```bash
$ plm marketplace list
No marketplaces registered.
Use 'plm marketplace add <owner/repo>' to add a marketplace.
```

## plm marketplace add

新しいマーケットプレイスを追加します。

### 構文

```bash
plm marketplace add <owner/repo> [--name <name>] [--path <dir>]
```

### オプション

| オプション | 説明 | デフォルト |
|------------|------|------------|
| `--name` | マーケットプレイスの表示名 | リポジトリ名 |
| `--path` | `marketplace.json` が配置されているサブディレクトリ | ルート（`.claude-plugin/`） |

### 名前の制約

- 許容文字: `[a-z0-9._-]`（小文字英数字、ピリオド、ハイフン、アンダースコア）
- 最大長: 64文字
- 大文字は自動的に小文字に正規化
- 先頭・末尾のピリオド/ハイフンは禁止

### 使用例

```bash
# 名前を自動設定
$ plm marketplace add company/claude-plugins
Fetching marketplace.json from company/claude-plugins...
Added marketplace 'claude-plugins' with 5 plugin(s).

# 名前を指定
$ plm marketplace add company/claude-plugins --name company-tools
Fetching marketplace.json from company/claude-plugins...
Added marketplace 'company-tools' with 5 plugin(s).

# サブディレクトリを指定
$ plm marketplace add company/monorepo --path packages/plugins
Fetching marketplace.json from company/monorepo...
Added marketplace 'monorepo' with 3 plugin(s).
```

### エラーケース

```bash
# 名前の重複
$ plm marketplace add company/claude-plugins --name company-tools
Error: Marketplace 'company-tools' already exists. Use --name to specify a different name.

# 無効な名前
$ plm marketplace add company/claude-plugins --name "Invalid Name!"
Error: Invalid character ' ' in name. Only [a-z0-9._-] are allowed.

# marketplace.json が存在しない
$ plm marketplace add company/no-marketplace
Error: Failed to fetch marketplace.json: Not Found
```

## plm marketplace remove

マーケットプレイスを削除します。

### 構文

```bash
plm marketplace remove <name>
```

### 使用例

```bash
$ plm marketplace remove company-tools
Removed marketplace 'company-tools'.
```

**注意**: インストール済みのプラグインは削除されません。

## plm marketplace update

登録済みマーケットプレイスの情報を更新します。

### 構文

```bash
plm marketplace update [name]
```

- `name` 省略時: 全マーケットプレイスを更新
- `name` 指定時: 指定されたマーケットプレイスのみ更新

### 使用例

```bash
# 全マーケットプレイスを更新
$ plm marketplace update
Updating 'anthropic'... 5 plugin(s)
Updating 'company-tools'... 3 plugin(s)

Updated 2 marketplace(s).

# 特定のマーケットプレイスのみ更新
$ plm marketplace update company-tools
Updating 'company-tools'... 3 plugin(s)

Updated 1 marketplace(s).
```

### 一部失敗時

```bash
$ plm marketplace update
Updating 'anthropic'... 5 plugin(s)
Updating 'offline-mp'... FAILED

Updated 1 marketplace(s).

Failed to update 1 marketplace(s):
  offline-mp: Network error: connection refused
```

## plm marketplace show

マーケットプレイスの詳細と利用可能なプラグインを表示します。

### 構文

```bash
plm marketplace show <name>
```

### 使用例

```bash
$ plm marketplace show company-tools
Marketplace: company-tools
Source: company/claude-plugins
Path: (root)
Owner: John Doe <john@example.com>
Last Updated: 2024-01-15 10:30:45 UTC

Plugins (3):
┌───────────┬─────────────────────────┬─────────┐
│ NAME      │ DESCRIPTION             │ VERSION │
├───────────┼─────────────────────────┼─────────┤
│ formatter │ Code formatting tool    │ 1.0.0   │
│ linter    │ Code linting tool       │ 2.0.0   │
│ debugger  │ Debugging utilities     │ 0.5.0   │
└───────────┴─────────────────────────┴─────────┘
```

### キャッシュが無い場合

```bash
$ plm marketplace show uncached-mp
Marketplace: uncached-mp
Source: some/repo
Path: (root)
Status: (not cached)

Run 'plm marketplace update uncached-mp' to fetch plugin information.
```

## 設定ファイル

マーケットプレイスの登録情報は `~/.plm/marketplaces.json` に保存されます。

```json
{
  "marketplaces": [
    {
      "name": "company-tools",
      "source": "company/claude-plugins"
    },
    {
      "name": "monorepo-plugins",
      "source": "company/monorepo",
      "source_path": "packages/plugins"
    }
  ]
}
```

## 関連

- [concepts/marketplace](../concepts/marketplace.md) - マーケットプレイスの仕組み

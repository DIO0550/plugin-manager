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
📦 Registered marketplaces:
   • anthropic (github:anthropics/claude-code)
   • company-tools (github:company/claude-plugins)
```

## plm marketplace add

新しいマーケットプレイスを追加します。

### 構文

```bash
plm marketplace add <owner/repo> [--name <name>]
```

### オプション

| オプション | 説明 | デフォルト |
|------------|------|------------|
| `--name` | マーケットプレイスの表示名 | リポジトリ名 |

### 使用例

```bash
# 名前を自動設定
$ plm marketplace add company/claude-plugins
📥 Fetching marketplace.json...
✅ Added marketplace: claude-plugins
   Available plugins: 5

# 名前を指定
$ plm marketplace add company/claude-plugins --name company-tools
📥 Fetching marketplace.json...
✅ Added marketplace: company-tools
   Available plugins: 5
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
✅ Removed marketplace: company-tools
```

## plm marketplace update

登録済みマーケットプレイスの情報を更新します。

```bash
$ plm marketplace update
📥 Updating marketplaces...
   ✓ anthropic (5 plugins)
   ✓ company-tools (3 plugins)
✅ Updated 2 marketplaces
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
📦 Marketplace: company-tools
   Source: github:company/claude-plugins

   Available plugins:
   • formatter (v1.0.0) - Code formatting tool
   • linter (v2.0.0) - Code linting tool
   • debugger (v0.5.0) - Debugging utilities
```

## 関連

- [concepts/marketplace](../concepts/marketplace.md) - マーケットプレイスの仕組み

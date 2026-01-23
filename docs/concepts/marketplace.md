# マーケットプレイス

PLMのマーケットプレイス機能について説明します。

## 概要

マーケットプレイスは、GitHubリポジトリをプラグインのカタログとして使用する仕組みです。`marketplace.json`ファイルで利用可能なプラグインを定義します。

## ファイル構造

```
~/.plm/
├── marketplaces.json          # マーケットプレイス登録情報
└── cache/
    └── marketplaces/
        ├── company-tools.json   # キャッシュ（プラグイン一覧）
        └── anthropic.json
```

### marketplaces.json

登録されたマーケットプレイスの情報を保存します。

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

| フィールド | 説明 |
|-----------|------|
| `name` | マーケットプレイス名（`[a-z0-9._-]`、最大64文字） |
| `source` | GitHubリポジトリ（`owner/repo` 形式） |
| `source_path` | `marketplace.json` が配置されているサブディレクトリ（省略可） |

### キャッシュファイル

GitHub から取得したプラグイン情報をキャッシュします。

```json
{
  "name": "company-tools",
  "fetched_at": "2024-01-15T10:30:45Z",
  "source": "github:company/claude-plugins",
  "owner": {
    "name": "John Doe",
    "email": "john@example.com"
  },
  "plugins": [
    {
      "name": "formatter",
      "source": "./plugins/formatter",
      "description": "Code formatting tool",
      "version": "1.0.0"
    }
  ]
}
```

## マーケットプレイスの登録

```bash
$ plm marketplace add company/claude-plugins --name company-tools
Fetching marketplace.json from company/claude-plugins...
Added marketplace 'company-tools' with 5 plugin(s).
```

サブディレクトリに `marketplace.json` がある場合:

```bash
$ plm marketplace add company/monorepo --path packages/plugins
```

## プラグインのインストール

### 基本的な使い方

マーケットプレイスからプラグインを検索してインストール:

```bash
plm install formatter
```

登録されたマーケットプレイスを検索し、該当するプラグインを見つけてインストールします。

### マーケットプレイスを明示的に指定

マーケットプレイス名を`@`で指定:

```bash
plm install formatter@company-tools
plm install linter@company-tools
```

## install 連携

`plm install <plugin>` 実行時の動作:

1. **登録済みマーケットプレイス検索**: `marketplaces.json` に登録されたマーケットプレイスのみを検索対象とする
2. **キャッシュ確認**: キャッシュが存在しない場合はエラー + `plm marketplace update` を案内
3. **競合検出**: 同名プラグインが複数マーケットプレイスに存在する場合はエラー + `plugin@marketplace` 形式を案内

### キャッシュが無い場合

```bash
$ plm install formatter
Error: Plugin not found: formatter; some marketplaces have no cache: company-tools.
Run 'plm marketplace update' to fetch plugin information.
```

### 同名プラグインの競合

```bash
$ plm install formatter
Error: Plugin 'formatter' found in multiple marketplaces: company-tools, anthropic.
Use 'formatter@<marketplace>' to specify which one.
```

## 1マーケットプレイス内の複数プラグイン

`marketplace.json`の`plugins`配列に複数のプラグインを定義できます。

### 一覧表示

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

### 個別インストール

各プラグインは独立してインストール可能:

```bash
plm install formatter@company-tools
plm install linter@company-tools
```

## 複数マーケットプレイスでの同名プラグイン

異なるマーケットプレイスに同名のプラグインが存在する場合の競合解決。

### CLIでの競合解決

```bash
$ plm install formatter
Error: Plugin 'formatter' found in multiple marketplaces: company-tools, anthropic.
Use 'formatter@<marketplace>' to specify which one.
```

明示的に指定して解決:

```bash
$ plm install formatter@company-tools
```

### TUIでの競合解決

選択ダイアログを表示:

```
┌─────────────────────────────────────────────────────────────┐
│  Multiple plugins found: formatter                          │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  > [ ] formatter@company-tools                              │
│        v1.0.0 - Code formatting tool                        │
│                                                             │
│    [ ] formatter@anthropic                                  │
│        v2.0.0 - Advanced formatter with AI                  │
│                                                             │
├─────────────────────────────────────────────────────────────┤
│  [Enter] Select   [Esc] Cancel                              │
└─────────────────────────────────────────────────────────────┘
```

## キャッシュディレクトリ構造

マーケットプレイスごとにフォルダ分けされます:

```
~/.plm/cache/plugins/
  company-tools/
    formatter/                  # marketplace 経由
    linter/
  anthropic/
    formatter/                  # 別 marketplace の同名 plugin
    code-review/
  github/
    owner/
      repo/                     # 直接 GitHub インストール
```

## デプロイ先パス

`<marketplace>/<plugin>/<component>`の3階層でデプロイ:

```
~/.codex/skills/
  company-tools/                    # marketplace
    code-formatter/                 # plugin
      formatter-skill/              # skill
        SKILL.md
      linter-skill/
        SKILL.md
  anthropic/
    code-formatter/                 # 同名 plugin でも別ディレクトリ
      ai-formatter-skill/
        SKILL.md
```

### 直接GitHubインストールの場合

marketplace = `github`、plugin = `owner--repo`として展開:

```
~/.codex/skills/
  github/                           # marketplace = "github"
    owner--repo/                    # plugin = "owner/repo" → "owner--repo"
      skill-name/
        SKILL.md
```

## 階層構造のメリット

| メリット | 説明 |
|----------|------|
| 出典の明確化 | ファイルシステム上で marketplace/plugin がわかる |
| 競合回避 | 同名 skill でも異なる plugin なら共存可能 |
| 管理の容易さ | plugin 単位での削除・更新が簡単 |

## 注意事項

Codex/Copilotがネストしたディレクトリを読み込むかは公式ドキュメントで明記されていません。読み込まれない場合はフラット構造（`~/.codex/skills/skill-name/`）にフォールバックする実装が必要になる可能性があります。

## plugin.json / marketplace.json

`plugin.json`と`marketplace.json`は**Claude Codeの公式フォーマット**です。詳細な仕様は[Claude Code Plugins Documentation](https://docs.anthropic.com/en/docs/claude-code/plugins)を参照してください。

### marketplace.json スキーマ

```json
{
  "name": "My Marketplace",
  "owner": {
    "name": "John Doe",
    "email": "john@example.com"
  },
  "plugins": [
    {
      "name": "plugin-a",
      "source": "./plugins/plugin-a",
      "description": "A useful plugin",
      "version": "1.0.0"
    },
    {
      "name": "external-plugin",
      "source": { "source": "github", "repo": "other/repo" },
      "description": "External plugin",
      "version": "2.1.0"
    }
  ]
}
```

| フィールド | 必須 | 説明 |
|-----------|------|------|
| `name` | ✓ | マーケットプレイス表示名 |
| `owner` | | オーナー情報 |
| `plugins` | ✓ | プラグイン定義の配列 |
| `plugins[].name` | ✓ | プラグイン名（マーケットプレイス内でユニーク） |
| `plugins[].source` | ✓ | プラグインソース（相対パスまたは外部リポジトリ） |
| `plugins[].description` | | 説明 |
| `plugins[].version` | | バージョン |

### PLMでの利用

PLMはこれらのフォーマットを読み取り、Codex/Copilot向けにコンポーネントを展開します。

| ファイル | 役割 | PLMでの使用 |
|----------|------|-------------|
| `plugin.json` | プラグインのマニフェスト | skills/agentsを検出し、ターゲットへ展開 |
| `marketplace.json` | マーケットプレイス定義 | プラグイン一覧の取得、インストール元の特定 |

### PLMが抽出するコンポーネント

```
plugin.json で定義されるコンポーネント:
├── skills/     → Codex/Copilotへ展開 ✅
├── agents/     → Codex/Copilotへ展開 ✅
├── commands/   → Claude Code専用（展開対象外）
├── hooks/      → Claude Code専用（展開対象外）
├── mcpServers  → Claude Code専用（展開対象外）
└── lspServers  → Claude Code専用（展開対象外）
```

## 関連

- [commands/marketplace](../commands/marketplace.md) - マーケットプレイス管理コマンド
- [architecture/cache](../architecture/cache.md) - キャッシュ構造
- [Claude Code Plugins](https://docs.anthropic.com/en/docs/claude-code/plugins) - 公式ドキュメント

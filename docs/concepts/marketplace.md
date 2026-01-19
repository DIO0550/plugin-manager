# マーケットプレイス

PLMのマーケットプレイス機能について説明します。

## 概要

マーケットプレイスは、GitHubリポジトリをプラグインのカタログとして使用する仕組みです。`marketplace.json`ファイルで利用可能なプラグインを定義します。

## マーケットプレイスの登録

```bash
$ plm marketplace add company/claude-plugins --name company-tools
📥 Fetching marketplace.json...
✅ Added marketplace: company-tools
   Available plugins: 5
```

## プラグインのインストール

マーケットプレイス名を`@`で指定してインストールします:

```bash
plm install formatter@company-tools
plm install linter@company-tools
```

## 1マーケットプレイス内の複数プラグイン

`marketplace.json`の`plugins`配列に複数のプラグインを定義できます。

### 一覧表示

```bash
$ plm marketplace show company-tools
📦 Marketplace: company-tools
   Source: github:company/claude-plugins

   Available plugins:
   • formatter (v1.0.0) - Code formatting tool
   • linter (v2.0.0) - Code linting tool
   • debugger (v0.5.0) - Debugging utilities
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
Error: Multiple plugins found with name 'formatter':
  - formatter@company-tools (v1.0.0) - Code formatting tool
  - formatter@anthropic (v2.0.0) - Advanced formatter with AI

Please specify: plm install formatter@<marketplace>
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

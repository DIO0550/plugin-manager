# plm import

Claude Code Pluginからコンポーネントをインポートします。

## 基本構文

```bash
plm import <source> [options]
```

## 引数

| 引数 | 説明 | 例 |
|------|------|-----|
| `<source>` | Claude Code Pluginのリポジトリ | `owner/claude-plugin` |

## オプション

| オプション | 説明 | 例 |
|------------|------|-----|
| `--component` | 特定のコンポーネントのみインポート | `--component skills/pdf` |
| `--type` | コンポーネント種別でフィルタ | `--type skill` |

## 使用例

### プラグイン全体をインポート

```bash
$ plm import owner/claude-plugin
📥 Fetching Claude Code plugin...
🔍 Detected components:
   • Skills: pdf, csv-analyzer
   • Agents: data-agent
   • Commands: analyze
📦 Importing to codex, copilot...
✅ Imported 4 components
```

### 特定のコンポーネントをインポート

```bash
$ plm import owner/claude-plugin --component skills/pdf
📥 Fetching skills/pdf from owner/claude-plugin...
📦 Importing to codex, copilot...
✅ Imported skill: pdf
```

### 種別でフィルタしてインポート

```bash
$ plm import owner/claude-plugin --type skill
📥 Fetching Claude Code plugin...
🔍 Importing skills only:
   • pdf
   • csv-analyzer
📦 Importing to codex, copilot...
✅ Imported 2 skills
```

## Claude Code Plugin構造

インポート元のClaude Code Pluginは以下の構造を持ちます:

```
plugin-name/
├── .claude-plugin/
│   └── plugin.json
├── commands/
│   └── command-name.md
├── agents/
│   └── agent-name.md
├── skills/
│   └── skill-name/
│       └── SKILL.md
├── hooks/
│   └── hooks.json
├── .mcp.json
└── .lsp.json
```

## インポート対象

以下のコンポーネントがインポート可能です:

| コンポーネント | インポート先 |
|----------------|--------------|
| Skills | Codex, Copilot |
| Agents | Codex, Copilot |
| Hooks | Copilot |

以下はClaude Code専用のため、インポート対象外です:

- Commands
- MCP Servers (.mcp.json)
- LSP Servers (.lsp.json)

以下はCopilotにのみインポート可能です:

- Hooks（`.github/hooks/` にJSON設定ファイルとして配置）

## 関連

- [concepts/marketplace](../concepts/marketplace.md) - plugin.json/marketplace.jsonについて
- [install](./install.md) - 通常のインストール

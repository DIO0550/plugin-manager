# plm info

プラグインの詳細情報を表示します。

## 基本構文

```bash
plm info <plugin-name>
```

## 引数

| 引数 | 説明 | 例 |
|------|------|-----|
| `<plugin-name>` | 詳細を表示するプラグイン名 | `code-formatter` |

## 使用例

```bash
$ plm info code-formatter

📦 code-formatter @ company-tools
   Version: 2.1.0
   Status: Enabled

   Author: Dev Team <dev@company.com>
   Source: company/claude-plugins@v2.1.0
   Installed: 2025-01-15T10:30:00Z

   Components:
   • Skills: code-formatter
   • Agents: formatter-agent
   • Commands: format

   Deployments:
   • codex (personal): ~/.codex/skills/company-tools/code-formatter/
   • copilot (project): .github/skills/company-tools/code-formatter/
```

## 表示情報

| フィールド | 説明 |
|------------|------|
| Name | プラグイン名 |
| Marketplace | インストール元のマーケットプレイス |
| Version | インストールされているバージョン |
| Status | 有効/無効状態 |
| Author | 作者情報 |
| Source | GitHubリポジトリ参照 |
| Installed | インストール日時 |
| Components | 含まれるコンポーネント一覧 |
| Deployments | 展開先パス |

## 関連

- [list](./list.md) - インストール済み一覧
- [managed](./managed.md) - TUI管理画面での詳細表示

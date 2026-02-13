# Personal/Project スコープ

PLMのインストールスコープについて説明します。

## スコープの種類

| スコープ | 説明 | 用途 |
|----------|------|------|
| **Personal** | ユーザーのホームディレクトリに配置 | 全プロジェクトで使用 |
| **Project** | プロジェクトディレクトリに配置 | 特定プロジェクトのみ |

## 配置場所

### Codex

| 種別 | Personal | Project |
|------|----------|---------|
| Skills | `~/.codex/skills/` | `.codex/skills/` |
| Agents | `~/.codex/agents/` | `.codex/agents/` |
| Instructions | `~/.codex/AGENTS.md` | `AGENTS.md` |

### Copilot

| 種別 | Personal | Project |
|------|----------|---------|
| Skills | - | `.github/skills/` |
| Agents | `~/.copilot/agents/` | `.github/agents/` |
| Commands | - | `.github/prompts/` |
| Instructions | - | `.github/copilot-instructions.md` |

### Antigravity

| 種別 | Personal | Project |
|------|----------|---------|
| Skills | `~/.gemini/antigravity/skills/` | `.agent/skills/` |

### Gemini CLI

| 種別 | Personal | Project |
|------|----------|---------|
| Skills | `~/.gemini/skills/` | `.gemini/skills/` |
| Instructions | `~/.gemini/GEMINI.md` | `GEMINI.md` |

## スコープの選択

### コマンドラインでの指定

```bash
# Personalスコープ
plm install owner/repo --scope personal

# Projectスコープ（デフォルト）
plm install owner/repo --scope project
```

### インタラクティブ選択

`--scope`未指定時は選択UIが表示されます:

```
? Select scope:
> ( ) personal - ~/.codex/, ~/.copilot/
  (x) project  - .codex/, .github/
```

## 使い分けガイド

### Personal スコープを使う場合

- すべてのプロジェクトで共通して使いたいSkill
- 個人的なワークフローやツール
- チームで共有しないコンポーネント

```bash
# 個人的なコードフォーマッターを全プロジェクトで使用
plm install my-formatter --scope personal
```

### Project スコープを使う場合

- プロジェクト固有のコーディング規約
- チームで共有するコンポーネント
- リポジトリにコミットして共有したい場合

```bash
# チームで共有するリンターをプロジェクトにインストール
plm install team/linter --scope project
```

## Copilotの特殊事情

Copilotはグローバルファイル（`~/.copilot/`等）を直接読み込みません。Personalスコープでインストールした場合、VSCode設定への参照追加が必要です:

```json
// settings.json (User)
{
  "github.copilot.chat.codeGeneration.instructions": [
    {
      "file": "~/.copilot/agents/my-agent.agent.md"
    }
  ],
  "github.copilot.chat.codeGeneration.useInstructionFiles": true
}
```

PLMはPersonalスコープでのCopilotインストール時に、この設定を自動で追加します。

## デフォルト設定

`~/.plm/config.toml`でデフォルトスコープを設定できます:

```toml
[general]
default_scope = "personal"  # または "project"
```

## 関連

- [concepts/targets](./targets.md) - ターゲット環境の詳細
- [reference/config](../reference/config.md) - 設定ファイル

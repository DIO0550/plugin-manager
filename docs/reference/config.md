# 設定ファイル

PLMの設定ファイル（`~/.plm/config.toml`）について説明します。

## 設定ファイルの場所

```
~/.plm/config.toml
```

## 基本構造

```toml
[general]
default_scope = "personal"  # personal | project

[targets]
enabled = ["codex", "copilot"]

[targets.codex]
skills_personal = "~/.codex/skills"
skills_project = ".codex/skills"
agents_personal = "~/.codex/agents"
agents_project = ".codex/agents"
instructions_personal = "~/.codex/AGENTS.md"
instructions_project = "AGENTS.md"

[targets.copilot]
skills_project = ".github/skills"
agents_personal = "~/.copilot/agents"
agents_project = ".github/agents"
prompts_project = ".github/prompts"
instructions_project = ".github/copilot-instructions.md"

[marketplaces]

[marketplaces.anthropic]
source = "github:anthropics/claude-code"
subdir = "plugins"

[marketplaces.company-tools]
source = "github:company/claude-plugins"
```

## セクション詳細

### [general]

一般設定。

| キー | 型 | デフォルト | 説明 |
|------|-----|------------|------|
| `default_scope` | string | `"project"` | デフォルトのインストールスコープ |

```toml
[general]
default_scope = "personal"  # または "project"
```

### [targets]

有効なターゲット環境の設定。

| キー | 型 | 説明 |
|------|-----|------|
| `enabled` | string[] | 有効なターゲットのリスト |

```toml
[targets]
enabled = ["codex", "copilot"]
```

### [targets.codex]

Codexターゲットのパス設定。

| キー | 型 | デフォルト |
|------|-----|------------|
| `skills_personal` | string | `"~/.codex/skills"` |
| `skills_project` | string | `".codex/skills"` |
| `agents_personal` | string | `"~/.codex/agents"` |
| `agents_project` | string | `".codex/agents"` |
| `instructions_personal` | string | `"~/.codex/AGENTS.md"` |
| `instructions_project` | string | `"AGENTS.md"` |

```toml
[targets.codex]
skills_personal = "~/.codex/skills"
skills_project = ".codex/skills"
agents_personal = "~/.codex/agents"
agents_project = ".codex/agents"
instructions_personal = "~/.codex/AGENTS.md"
instructions_project = "AGENTS.md"
```

### [targets.copilot]

Copilotターゲットのパス設定。

| キー | 型 | デフォルト |
|------|-----|------------|
| `skills_project` | string | `".github/skills"` |
| `agents_personal` | string | `"~/.copilot/agents"` |
| `agents_project` | string | `".github/agents"` |
| `prompts_project` | string | `".github/prompts"` |
| `instructions_project` | string | `".github/copilot-instructions.md"` |

```toml
[targets.copilot]
skills_project = ".github/skills"
agents_personal = "~/.copilot/agents"
agents_project = ".github/agents"
prompts_project = ".github/prompts"
instructions_project = ".github/copilot-instructions.md"
```

> 注: CopilotはPersonalスコープでのSkillsをサポートしていません。

### [marketplaces]

登録済みマーケットプレイスの設定。

| キー | 型 | 説明 |
|------|-----|------|
| `source` | string | GitHubリポジトリ（`github:owner/repo`形式） |
| `subdir` | string | マーケットプレイスのサブディレクトリ（オプション） |

```toml
[marketplaces.anthropic]
source = "github:anthropics/claude-code"
subdir = "plugins"

[marketplaces.company-tools]
source = "github:company/claude-plugins"
```

## 設定の初期化

初回実行時にデフォルト設定が作成されます。

```bash
# 設定ファイルを手動で作成
mkdir -p ~/.plm
cat > ~/.plm/config.toml << 'EOF'
[general]
default_scope = "project"

[targets]
enabled = ["codex", "copilot"]
EOF
```

## 環境変数

| 変数 | 説明 |
|------|------|
| `PLM_HOME` | PLMのホームディレクトリ（デフォルト: `~/.plm`） |
| `PLM_CONFIG` | 設定ファイルのパス（デフォルト: `$PLM_HOME/config.toml`） |

## 設定の優先順位

1. コマンドライン引数（`--scope`, `--target`等）
2. 環境変数
3. 設定ファイル
4. デフォルト値

## 関連ファイル

| ファイル | 説明 |
|----------|------|
| `~/.plm/config.toml` | 設定ファイル |
| `~/.plm/plugins.json` | プラグインキャッシュ |
| `~/.plm/cache/marketplaces/` | マーケットプレイスキャッシュ |
| `~/.plm/cache/plugins/` | プラグインファイルキャッシュ |

## 関連

- [concepts/targets](../concepts/targets.md) - ターゲット環境
- [concepts/scopes](../concepts/scopes.md) - Personal/Projectスコープ
- [architecture/cache](../architecture/cache.md) - キャッシュアーキテクチャ

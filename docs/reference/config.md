# 設定ファイル

> **⚠️ 未実装（将来仕様）**: このページに記載されている `~/.plm/config.toml` および `PLM_CONFIG` 環境変数は**現時点で未実装**です。読み込む処理は存在せず、ファイルを作成しても効果はありません。現在実際に使用される設定ファイルは「[現在の実装状態](#現在の実装状態)」を参照してください。

PLMの設定ファイル（`~/.plm/config.toml`）の**将来仕様**について説明します。

## 現在の実装状態

現時点で PLM が実際に読み書きする設定・状態ファイルは以下の JSON ファイルです：

| ファイル | 説明 |
|----------|------|
| `~/.plm/targets.json` | 有効なターゲット環境（`plm target add/remove` で管理） |
| `~/.plm/marketplaces.json` | 登録済みマーケットプレイス（`plm marketplace add/remove` で管理） |
| `~/.plm/imports.json` | インポート履歴 |
| `~/.plm/cache/marketplaces/<name>.json` | マーケットプレイスキャッシュ |
| `~/.plm/cache/plugins/<marketplace>/<plugin>/` | プラグインファイルキャッシュ（各プラグイン直下に `.plm-meta.json`） |

- ターゲットの配置先パスは各ターゲット実装内のハードコード定数であり、現状は変更できません。
- デフォルトスコープの設定はできません。`--scope` 未指定時は対話（TUI）選択になります。
- `PLM_HOME` はキャッシュルートの解決で参照されますが、全パスで一貫して尊重されるとは限りません。

---

以下は将来実装予定の仕様です。

## 設定ファイルの場所

```
~/.plm/config.toml
```

## 基本構造

```toml
[general]
default_scope = "personal"  # personal | project

[targets]
enabled = ["antigravity", "codex", "copilot", "gemini"]

[targets.antigravity]
skills_personal = "~/.gemini/antigravity/skills"
skills_project = ".agent/skills"

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
commands_project = ".github/prompts"
instructions_project = ".github/copilot-instructions.md"

[targets.gemini]
skills_personal = "~/.gemini/skills"
skills_project = ".gemini/skills"
instructions_personal = "~/.gemini/GEMINI.md"
instructions_project = "GEMINI.md"

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
enabled = ["antigravity", "codex", "copilot", "gemini"]
```

### [targets.antigravity]

Antigravityターゲットのパス設定。

| キー | 型 | デフォルト |
|------|-----|------------|
| `skills_personal` | string | `"~/.gemini/antigravity/skills"` |
| `skills_project` | string | `".agent/skills"` |

```toml
[targets.antigravity]
skills_personal = "~/.gemini/antigravity/skills"
skills_project = ".agent/skills"
```

> 注: AntigravityはSkillsのみサポートしています。

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
| `commands_project` | string | `".github/prompts"` |
| `instructions_project` | string | `".github/copilot-instructions.md"` |

```toml
[targets.copilot]
skills_project = ".github/skills"
agents_personal = "~/.copilot/agents"
agents_project = ".github/agents"
commands_project = ".github/prompts"
instructions_project = ".github/copilot-instructions.md"
```

> 注: CopilotはPersonalスコープでのSkillsをサポートしていません。

### [targets.gemini]

Gemini CLIターゲットのパス設定。

| キー | 型 | デフォルト |
|------|-----|------------|
| `skills_personal` | string | `"~/.gemini/skills"` |
| `skills_project` | string | `".gemini/skills"` |
| `instructions_personal` | string | `"~/.gemini/GEMINI.md"` |
| `instructions_project` | string | `"GEMINI.md"` |

```toml
[targets.gemini]
skills_personal = "~/.gemini/skills"
skills_project = ".gemini/skills"
instructions_personal = "~/.gemini/GEMINI.md"
instructions_project = "GEMINI.md"
```

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

（将来仕様）初回実行時にデフォルト設定が作成される予定です。

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

| 変数 | 説明 | 実装状況 |
|------|------|----------|
| `PLM_HOME` | PLMのホームディレクトリ（デフォルト: `~/.plm`） | 一部実装（キャッシュルート解決） |
| `PLM_CONFIG` | 設定ファイルのパス（デフォルト: `$PLM_HOME/config.toml`） | 未実装 |

## 設定の優先順位

1. コマンドライン引数（`--scope`, `--target`等）
2. 環境変数
3. 設定ファイル
4. デフォルト値

## 関連ファイル

| ファイル | 説明 | 実装状況 |
|----------|------|----------|
| `~/.plm/config.toml` | 設定ファイル | 未実装 |
| `~/.plm/targets.json` | 有効ターゲット設定 | 実装済み |
| `~/.plm/marketplaces.json` | マーケットプレイス登録設定 | 実装済み |
| `~/.plm/imports.json` | インポート履歴 | 実装済み |
| `~/.plm/cache/marketplaces/` | マーケットプレイスキャッシュ | 実装済み |
| `~/.plm/cache/plugins/` | プラグインファイルキャッシュ | 実装済み |

## 関連

- [concepts/targets](../concepts/targets.md) - ターゲット環境
- [concepts/scopes](../concepts/scopes.md) - Personal/Projectスコープ
- [architecture/cache](../architecture/cache.md) - キャッシュアーキテクチャ

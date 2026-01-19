# コンポーネント種別

PLMが管理するコンポーネントの種類について説明します。

## コンポーネント一覧

| 種別 | 説明 | ファイル形式 |
|------|------|-------------|
| **Skills** | 専門的な知識・ワークフロー | `SKILL.md` (YAML frontmatter) |
| **Agents** | カスタムエージェント定義 | `*.agent.md` |
| **Prompts** | 再利用可能なプロンプト | `*.prompt.md` |
| **Instructions** | コーディング規約・カスタム指示 | `AGENTS.md` / `copilot-instructions.md` |

## Skills

専門的な知識やワークフローを定義するコンポーネントです。

### 特徴

- YAML frontmatterでメタデータを定義
- 専門的なタスクを実行するための詳細な指示を含む
- Codex、Copilot両方でサポート

### ファイル形式

```markdown
---
name: skill-name
description: スキルの説明（500文字以内）
metadata:
  short-description: 短い説明
---

# Skill Name

スキルの詳細な指示...
```

### 配置場所

| ターゲット | Personal | Project |
|------------|----------|---------|
| Codex | `~/.codex/skills/<marketplace>/<plugin>/<skill>/` | `.codex/skills/<marketplace>/<plugin>/<skill>/` |
| Copilot | - | `.github/skills/<marketplace>/<plugin>/<skill>/` |

## Agents

カスタムエージェントを定義するコンポーネントです。

### 特徴

- 特定のタスクに特化したエージェントを定義
- 使用可能なツールを指定可能
- Copilotで公式サポート、Codexは将来対応見込み

### ファイル形式

```markdown
---
name: agent-name
description: エージェントの説明
tools: ['search', 'fetch', 'edit']
---

# Agent Instructions

エージェントの指示...
```

### 配置場所

| ターゲット | Personal | Project |
|------------|----------|---------|
| Codex | `~/.codex/agents/<marketplace>/<plugin>/` | `.codex/agents/<marketplace>/<plugin>/` |
| Copilot | `~/.copilot/agents/<marketplace>/<plugin>/` | `.github/agents/<marketplace>/<plugin>/` |

## Prompts

再利用可能なプロンプトを定義するコンポーネントです。

### 特徴

- よく使うプロンプトをテンプレート化
- Copilotでのみサポート
- 手動で呼び出して使用

### ファイル形式

```markdown
---
name: prompt-name
description: プロンプトの説明
---

# Prompt

プロンプトの内容...
```

### 配置場所

| ターゲット | Personal | Project |
|------------|----------|---------|
| Codex | - | - |
| Copilot | - | `.github/prompts/<marketplace>/<plugin>/` |

## Instructions

プロジェクト固有のコーディング規約やカスタム指示を定義するコンポーネントです。

### 特徴

- AGENTS.md形式のオープン標準（Linux Foundation管轄）
- プロジェクト全体に適用される指示
- Codex、Copilot両方でサポート

### ファイル形式

```markdown
# Project Guidelines

プロジェクト固有のコーディング規約やワークフロー...
```

### 配置場所

| ターゲット | Personal | Project |
|------------|----------|---------|
| Codex | `~/.codex/AGENTS.md` | `AGENTS.md` |
| Copilot | - | `.github/copilot-instructions.md`, `AGENTS.md` |

## ターゲット別サポート状況

| コンポーネント | Codex | Copilot |
|----------------|-------|---------|
| Skills | ✅ | ✅ |
| Agents | ✅* | ✅ |
| Prompts | ❌ | ✅ |
| Instructions | ✅ | ✅ |

> *Codexは現時点で`.agent.md`を公式サポートしていませんが、将来対応を見越して配置します。

## 共通規格

| 規格 | 説明 | 参照 |
|------|------|------|
| **AGENTS.md** | カスタム指示ファイル（Linux Foundation管轄のオープン標準） | https://agents.md |
| **SKILL.md** | スキル定義（Anthropicがオープン標準として公開、OpenAI/Microsoftが採用） | - |

## 関連

- [concepts/targets](./targets.md) - ターゲット環境の詳細
- [commands/init](../commands/init.md) - コンポーネントテンプレート作成

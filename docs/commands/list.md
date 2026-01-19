# plm list

インストール済みコンポーネントの一覧を表示します。

## 基本構文

```bash
plm list [options]
```

## オプション

| オプション | 説明 | 例 |
|------------|------|-----|
| `--target` | 特定のターゲット環境でフィルタ | `--target codex` |
| `--type` | コンポーネント種別でフィルタ | `--type skill` |

## 使用例

### 全てのコンポーネントを表示

```bash
$ plm list
┌────────────────────────────┬─────────┬────────┬───────────────┬─────────────┐
│ Name                       │ Version │ Type   │ Targets       │ Marketplace │
├────────────────────────────┼─────────┼────────┼───────────────┼─────────────┤
│ html-educational-material  │ 1.0.0   │ skill  │ codex,copilot │ -           │
│ code-formatter             │ 2.1.0   │ plugin │ codex,copilot │ company     │
│ code-reviewer              │ 0.1.0   │ agent  │ copilot       │ -           │
└────────────────────────────┴─────────┴────────┴───────────────┴─────────────┘
```

### ターゲット別フィルタ

```bash
# Codexにインストールされたコンポーネントのみ
plm list --target codex

# Copilotにインストールされたコンポーネントのみ
plm list --target copilot
```

### 種別フィルタ

```bash
# Skillsのみ表示
plm list --type skill

# Agentsのみ表示
plm list --type agent

# Promptsのみ表示
plm list --type prompt
```

### フィルタの組み合わせ

```bash
# CodexのSkillsのみ表示
plm list --target codex --type skill
```

## 出力フィールド

| フィールド | 説明 |
|------------|------|
| Name | コンポーネント/プラグイン名 |
| Version | インストールされているバージョン |
| Type | 種別（skill, agent, prompt, plugin） |
| Targets | インストール先のターゲット環境 |
| Marketplace | インストール元のマーケットプレイス（直接インストールの場合は`-`） |

## 関連

- [info](./info.md) - プラグインの詳細情報
- [managed](./managed.md) - TUI管理画面での一覧表示

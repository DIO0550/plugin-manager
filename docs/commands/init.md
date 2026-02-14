# plm init

コンポーネントのテンプレートを作成します。

## 基本構文

```bash
plm init <name> --type <type>
```

## 引数

| 引数 | 説明 | 例 |
|------|------|-----|
| `<name>` | コンポーネント名 | `my-skill`, `my-agent` |

## オプション

| オプション | 説明 | 必須 |
|------------|------|------|
| `--type` | コンポーネント種別 | ✅ |

## コンポーネント種別

| 種別 | 説明 |
|------|------|
| `skill` | SKILL.md テンプレートを作成 |
| `agent` | *.agent.md テンプレートを作成 |
| `command` | *.prompt.md テンプレートを作成 |

## 使用例

### Skillの作成

```bash
$ plm init my-skill --type skill
📁 Created my-skill/
   └── SKILL.md
```

生成されるファイル:

```markdown
---
name: my-skill
description: スキルの説明
metadata:
  short-description: 短い説明
---

# my-skill

スキルの詳細な指示をここに記述...
```

### Agentの作成

```bash
$ plm init my-agent --type agent
📁 Created my-agent.agent.md
```

生成されるファイル:

```markdown
---
name: my-agent
description: エージェントの説明
tools: ['search', 'fetch', 'edit']
---

# my-agent

エージェントの指示をここに記述...
```

### Commandの作成

```bash
$ plm init my-command --type command
📁 Created my-command.prompt.md
```

生成されるファイル:

```markdown
---
name: my-command
description: コマンドの説明
---

# my-command

コマンドの内容をここに記述...
```

## 関連

- [concepts/components](../concepts/components.md) - コンポーネント種別
- [pack](./pack.md) - コンポーネントのパッケージ化

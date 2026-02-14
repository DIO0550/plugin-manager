# plm target

ターゲット環境（Codex, Copilot等）を管理します。

## サブコマンド

| サブコマンド | 説明 |
|--------------|------|
| `list` | 登録済みターゲットの一覧表示 |
| `add` | ターゲットを追加 |
| `remove` | ターゲットを削除 |

## plm target list

登録済みのターゲット環境を表示します。

```bash
$ plm target list
📍 Active targets:
   • antigravity (skills)
   • codex       (skills, agents, instructions)
   • copilot     (skills, agents, commands, instructions)
   • gemini      (skills, instructions)
```

## plm target add

新しいターゲット環境を追加します。

### 構文

```bash
plm target add <target-name>
```

### 使用例

```bash
$ plm target add codex
✅ Added target: codex
   Supports: skills, agents, instructions

$ plm target add copilot
✅ Added target: copilot
   Supports: skills, agents, commands, instructions

$ plm target add antigravity
✅ Added target: antigravity
   Supports: skills

$ plm target add gemini
✅ Added target: gemini
   Supports: skills, instructions
```

### 利用可能なターゲット

| ターゲット | サポートするコンポーネント |
|------------|----------------------------|
| `antigravity` | Skills |
| `codex` | Skills, Agents, Instructions |
| `copilot` | Skills, Agents, Commands, Instructions |
| `gemini` | Skills, Instructions |

## plm target remove

ターゲット環境を削除します。

### 構文

```bash
plm target remove <target-name>
```

### 使用例

```bash
$ plm target remove copilot
✅ Removed target: copilot
```

## 関連

- [concepts/targets](../concepts/targets.md) - ターゲット環境の詳細
- [reference/config](../reference/config.md) - ターゲット設定

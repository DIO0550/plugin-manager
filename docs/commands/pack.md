# plm pack

コンポーネントを配布用にパッケージ化します。

## 基本構文

```bash
plm pack <path>
```

## 引数

| 引数 | 説明 | 例 |
|------|------|-----|
| `<path>` | パッケージ化するコンポーネントのパス | `./my-skill`, `./my-plugin` |

## 使用例

### Skillのパッケージ化

```bash
$ plm pack ./my-skill
📦 Packaging my-skill...
✅ Created my-skill.zip
   Contents:
   └── SKILL.md
```

### プラグインのパッケージ化

```bash
$ plm pack ./my-plugin
📦 Packaging my-plugin...
✅ Created my-plugin.zip
   Contents:
   ├── .claude-plugin/
   │   └── plugin.json
   ├── skills/
   │   └── my-skill/
   │       └── SKILL.md
   └── agents/
       └── my-agent.agent.md
```

## 出力

- ZIPファイルが現在のディレクトリに作成されます
- ファイル名は `<name>.zip` 形式

## バリデーション

パッケージ化時に以下のバリデーションが実行されます:

- 必須ファイルの存在確認
- YAML frontmatterの構文チェック
- plugin.jsonの構文チェック（プラグインの場合）

## 関連

- [init](./init.md) - コンポーネントテンプレートの作成
- [concepts/marketplace](../concepts/marketplace.md) - plugin.json/marketplace.jsonについて

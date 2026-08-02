# plm pack

コンポーネントまたはプラグインを配布用にパッケージ化します。

## 基本構文

```bash
plm pack <path>
```

## 引数

| 引数 | 説明 | 例 |
|------|------|-----|
| `<path>` | パッケージ化するディレクトリ | `./my-skill`, `./my-plugin` |

## 対象判定

| 条件 | 扱い | ZIP 名 |
|------|------|--------|
| `.claude-plugin/plugin.json` または `plugin.json` がある | プラグイン | `plugin.json` の `name` |
| 直下に `SKILL.md` がある | Skill 単体 | ディレクトリ名 |
| それ以外 | エラー | — |

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

- ZIP ファイルが現在のディレクトリに作成される
- ファイル名は `<name>.zip`
- 既存の同名 ZIP がある場合はエラー（上書きしない）

## バリデーション

パッケージ化時に以下を検証する:

| 対象 | 内容 |
|------|------|
| Skill | `SKILL.md` の YAML frontmatter 構文、`name` / `description` 必須 |
| プラグイン | `plugin.json` 構文、`name` / `version` 必須。配下 skill があれば frontmatter も検証 |

## 除外

次は ZIP に含めない:

- `.git/` 配下
- `.plm-meta.json`
- シンボリックリンク

## 関連

- [init](./init.md) - コンポーネントテンプレートの作成
- [concepts/marketplace](../concepts/marketplace.md) - plugin.json/marketplace.jsonについて

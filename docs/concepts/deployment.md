# 自動展開の仕組み

PLMがプラグインのコンポーネントを各ターゲット環境に自動展開する仕組みについて説明します。

## 展開の流れ

```
1. plm install owner/repo@v1.0.0
2. GitRepo::parse("owner/repo@v1.0.0")
3. repo.github_zipball_url("v1.0.0") でダウンロード
4. ~/.plm/cache/plugins/<marketplace>/<name>/ に展開
5. plugin.json パース
6. デプロイ先の競合チェック
7. ターゲットへ自動展開
8. CachedPlugin作成（source = repo.raw, marketplace = marketplace名）
9. plugins.json に保存
```

## 階層型デプロイ（marketplace/plugin/component）

マーケットプレイス経由でインストールした場合、デプロイ先は `<marketplace>/<plugin>/<component>` の3階層:

### Skills

```
プラグイン内:
├── skills/
│   └── formatter-skill/
│       └── SKILL.md

展開先:
Codex:   ~/.codex/skills/company-tools/code-formatter/formatter-skill/SKILL.md
Copilot: .github/skills/company-tools/code-formatter/formatter-skill/SKILL.md
```

### Agents

```
プラグイン内:
├── agents/
│   └── formatter-agent.md

展開先:
Codex:   ~/.codex/agents/company-tools/code-formatter/formatter-agent.agent.md
Copilot: .github/agents/company-tools/code-formatter/formatter-agent.agent.md
```

> Codexは現時点で`.agent.md`を公式サポートしていませんが、将来対応を見越して配置します。

### Prompts

```
プラグイン内:
├── prompts/
│   └── format-prompt.prompt.md

展開先:
Copilot: .github/prompts/company-tools/code-formatter/format-prompt.prompt.md
Codex:   展開対象外（未サポート）
```

### 展開対象外

以下のコンポーネントはClaude Code専用のため、展開対象外です:

- `commands/` - スラッシュコマンド
- `hooks/` - イベントハンドラ
- `.mcp.json` - MCPサーバー設定
- `.lsp.json` - LSPサーバー設定

## 直接GitHubインストールの場合

マーケットプレイス経由でない場合は `github/<owner--repo>/<component>` に展開:

```
Codex:   ~/.codex/skills/github/owner--repo/skill-name/SKILL.md
Copilot: .github/skills/github/owner--repo/skill-name/SKILL.md
```

## デプロイ例

### マーケットプレイス経由

```
~/.codex/skills/
  company-tools/                    # marketplace
    code-formatter/                 # plugin
      formatter-skill/              # skill
        SKILL.md
      linter-skill/
        SKILL.md
  anthropic/
    code-formatter/                 # 同名 plugin でも別ディレクトリ
      ai-formatter-skill/
        SKILL.md
```

### 直接GitHubインストール

```
~/.codex/skills/
  github/                           # marketplace = "github"
    owner--repo/                    # plugin = "owner/repo" → "owner--repo"
      skill-name/
        SKILL.md
```

## 階層構造のメリット

| メリット | 説明 |
|----------|------|
| **出典の明確化** | ファイルシステム上で marketplace/plugin がわかる |
| **競合回避** | 同名 skill でも異なる plugin なら共存可能 |
| **管理の容易さ** | plugin 単位での削除・更新が簡単 |

## 注意事項

Codex/Copilotがネストしたディレクトリを読み込むかは公式ドキュメントで明記されていません。読み込まれない場合はフラット構造（`~/.codex/skills/skill-name/`）にフォールバックする実装が必要になる可能性があります。

## 環境別の展開マッピング

### Codex

| コンポーネント | Personal | Project |
|----------------|----------|---------|
| Skills | `~/.codex/skills/<marketplace>/<plugin>/<skill>/` | `.codex/skills/<marketplace>/<plugin>/<skill>/` |
| Agents | `~/.codex/agents/<marketplace>/<plugin>/` | `.codex/agents/<marketplace>/<plugin>/` |
| Instructions | `~/.codex/AGENTS.md` | `AGENTS.md` |

### Copilot

| コンポーネント | Personal | Project |
|----------------|----------|---------|
| Skills | - | `.github/skills/<marketplace>/<plugin>/<skill>/` |
| Agents | `~/.copilot/agents/<marketplace>/<plugin>/` | `.github/agents/<marketplace>/<plugin>/` |
| Prompts | - | `.github/prompts/<marketplace>/<plugin>/` |
| Instructions | - | `.github/copilot-instructions.md` |

## 関連

- [concepts/marketplace](./marketplace.md) - マーケットプレイスの仕組み
- [concepts/scopes](./scopes.md) - Personal/Projectスコープ
- [architecture/cache](../architecture/cache.md) - キャッシュアーキテクチャ

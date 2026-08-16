# plm install

GitHubまたはマーケットプレイスからプラグインをインストールします。

## 基本構文

```bash
plm install <source> [options]
```

## 引数

| 引数 | 説明 | 例 |
|------|------|-----|
| `<source>` | インストール元 | `owner/repo`, `owner/repo@v1.0.0`, `plugin@marketplace` |

## オプション

| オプション | 説明 | デフォルト |
|------------|------|------------|
| `--target` | ターゲット環境を指定 | 全ての有効なターゲット |
| `--scope` | スコープを指定（personal/project） | `project` |
| `--type` | コンポーネント種別でフィルタ（skill, agent, command, instruction） | 全種別 |
| `--force` | キャッシュ済みでも再ダウンロード | - |

## 使用例

### GitHubから直接インストール

```bash
# 最新版をインストール
plm install owner/repo

# 特定のバージョン/タグをインストール
plm install owner/repo@v1.0.0

# 特定のブランチをインストール
plm install owner/repo@feature-branch
```

### マーケットプレイス経由

```bash
# マーケットプレイスからインストール
plm install formatter@company-tools

# 別のマーケットプレイスから同名プラグインをインストール
plm install formatter@anthropic
```

### ターゲット・スコープ指定

```bash
# Codexのみにインストール
plm install owner/repo --target codex

# Copilotのみにインストール
plm install owner/repo --target copilot

# Personalスコープにインストール
plm install owner/repo --scope personal
```

## インタラクティブ選択

`--target`未指定時、有効なターゲットから選択UIを表示:

```
$ plm install formatter@my-market

? Select target(s) to deploy: (use space to select, enter to confirm)
> [x] codex   - Skills, Agents, Instructions
  [x] copilot - Skills, Agents, Prompts, Instructions

? Select scope:
> ( ) personal - ~/.codex/, ~/.copilot/
  (x) project  - .codex/, .github/

📥 Installing formatter to codex, copilot (project scope)...
```

## 同名プラグインの競合

複数のマーケットプレイスに同名のプラグインがある場合:

```bash
$ plm install formatter
Error: Multiple plugins found with name 'formatter':
  - formatter@company-tools (v1.0.0) - Code formatting tool
  - formatter@anthropic (v2.0.0) - Advanced formatter with AI

Please specify: plm install formatter@<marketplace>
```

## 動作詳細

1. ソースをパースしてGitHubリポジトリを特定
2. `GitRepo::parse()`でリポジトリ情報を解析
3. `repo.github_zipball_url()`でZIPをダウンロード
4. `~/.plm/cache/plugins/<marketplace>/<name>/`に展開
5. `plugin.json`をパースしてコンポーネントを検出
6. デプロイ先の競合チェック
7. ターゲット環境へ自動展開
8. `CachedPlugin`を作成し`plugins.json`に保存

## 部分成功

プラグイン内の一部コンポーネントだけが変換・配置に失敗した場合、成功したコンポーネントはディスク上に残ります。プラグイン全体を失敗として巻き戻しません。

- CLI は成功行と失敗行を出し、対象ターゲットに混在があるとき `{target} - PARTIAL` と集計します
- TUI は `⚠ {plugin}: N/M components placed, K failed` のあと、失敗コンポーネントを箇条書きします
- 1 件以上置けたターゲットは `.plm-meta.json` の `statusByTarget` を `enabled` にし、Plugin リソースも配置します
- 全コンポーネントが失敗したターゲットは meta を昇格せず、Plugin リソースも置きません

失敗したコンポーネントはソースを直したうえで `plm install --force` で再配置します。

## デプロイ先

インストールされたコンポーネントは以下のパスに展開されます:

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

- [concepts/marketplace](../concepts/marketplace.md) - マーケットプレイスの仕組み
- [concepts/deployment](../concepts/deployment.md) - 自動展開の詳細
- [concepts/scopes](../concepts/scopes.md) - Personal/Projectスコープ

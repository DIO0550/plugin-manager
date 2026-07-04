# キャッシュアーキテクチャ

PLMのキャッシュ設計について説明します。

## 全体構成

```
┌─────────────────────────────────────────────────────────────────┐
│                        plm managed (TUI)                        │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                        PluginCache                              │
│                    (~/.plm/plugins.json)                        │
│  ┌────────────────────────────────────────────────────────────┐ │
│  │ CachedPlugin                                               │ │
│  │  - name: String                                            │ │
│  │  - marketplace: Option<String>                             │ │
│  │  - source: String (GitRepo.raw)  ──┐                       │ │
│  │  - version: String                 │                       │ │
│  │  - status: Enabled/Disabled        │                       │ │
│  │  - installed_sha: String           │                       │ │
│  │  - components: [...]               │                       │ │
│  │  - deployments: {...}              │                       │ │
│  └────────────────────────────────────│───────────────────────┘ │
└───────────────────────────────────────│─────────────────────────┘
                                        │
                                        ▼ GitRepo::parse()
                              ┌─────────────────────┐
                              │      GitRepo        │
                              │  - owner            │
                              │  - repo             │
                              │  - git_ref          │
                              │  - raw              │
                              │                     │
                              │  github_web_url()   │──→ ブラウザで開く
                              │  github_*_url()     │──→ API呼び出し
                              └─────────────────────┘
                                        │
                                        ▼
                              ┌─────────────────────┐
                              │    GitHub API       │
                              │  - 更新チェック     │
                              │  - ダウンロード     │
                              └─────────────────────┘
```

## キャッシュディレクトリ構造

### 全体構造

```
~/.plm/
├── targets.json                    # 有効ターゲット設定
├── marketplaces.json               # マーケットプレイス登録設定
├── imports.json                    # インポート履歴
└── cache/
    ├── marketplaces/               # マーケットプレイスキャッシュ
    │   ├── anthropic.json
    │   └── company-tools.json
    └── plugins/                    # プラグインファイルキャッシュ
        ├── .backup/                # 更新時のバックアップ（作業用）
        ├── .temp/                  # 更新時のステージング（作業用）
        ├── company-tools/
        │   ├── formatter/          # 各プラグイン直下に .plm-meta.json
        │   └── linter/
        ├── anthropic/
        │   ├── formatter/
        │   └── code-review/
        └── github/
            └── owner--repo/        # 直接GitHubインストールは owner--repo 形式
```

> 注: `~/.plm/config.toml` は未実装の将来仕様です（[reference/config](../reference/config.md) 参照）。

### プラグインキャッシュ（階層型）

マーケットプレイスごとにフォルダ分け:

```
~/.plm/cache/plugins/
  company-tools/                    # マーケットプレイス
    formatter/                      # プラグイン
    linter/
  anthropic/
    formatter/                      # 別マーケットプレイスの同名プラグイン
    code-review/
  github/                           # 直接GitHubインストール
    owner--repo/                    # {owner}--{repo} 形式のID
```

## プラグインメタデータ（.plm-meta.json）

プラグインの状態は中央集約ファイルではなく、**各プラグインのキャッシュディレクトリ直下の `.plm-meta.json`** に分散して保持される（`src/plugin/meta/meta.rs`）。

```json
{
  "installedAt": "2025-01-15T10:30:00Z",
  "updatedAt": "2025-02-01T09:00:00Z",
  "statusByTarget": {
    "codex": "enabled",
    "copilot": "disabled"
  },
  "gitRef": "main",
  "commitSha": "abc123def456...",
  "sourceRepo": "company/claude-plugins",
  "marketplace": "company-tools",
  "managedFiles": {
    "codex": ["/home/user/.codex/hooks.json"]
  }
}
```

### フィールド説明

| フィールド | 説明 |
|------------|------|
| `installedAt` / `updatedAt` | インストール・更新日時（RFC3339） |
| `statusByTarget` | ターゲット別の `enabled` / `disabled` 状態 |
| `gitRef` | Git参照（ブランチ・タグ） |
| `commitSha` | インストール時のコミットSHA（更新判定に使用） |
| `sourceRepo` | ソースリポジトリ（`owner/repo` 形式） |
| `marketplace` | マーケットプレイス名（直接インストールは `github`） |
| `managedFiles` | 共有配置先ファイル（`.codex/hooks.json` 等）の所有権追跡 |

プラグイン名・バージョン・説明・コンポーネント一覧などは `.plm-meta.json` には持たず、キャッシュ内の `plugin.json`（上流成果物、変更しない）とディレクトリ走査から都度取得する。

## マーケットプレイスキャッシュ

`~/.plm/cache/marketplaces/<name>.json`:

```json
{
  "name": "company-tools",
  "fetched_at": "2025-01-15T10:00:00Z",
  "source": "github:company/claude-plugins",
  "owner": {
    "name": "Company Dev Team",
    "email": "dev@company.com"
  },
  "plugins": [
    {
      "name": "code-formatter",
      "source": "./plugins/code-formatter",
      "description": "Automatic code formatting",
      "version": "2.1.0"
    }
  ]
}
```

## キャッシュの役割

| 役割 | 説明 |
|------|------|
| **オフライン表示** | TUI起動時にネットワーク不要で一覧表示 |
| **状態管理** | Enabled/Disabled、バージョン情報 |
| **更新検知** | `commitSha`とリモート最新SHAを比較 |
| **永続化** | `sourceRepo` + `gitRef`からソース参照を復元可能 |
| **marketplace追跡** | どのmarketplaceからインストールしたかを記録 |

## キャッシュ操作

- **ルート解決**: キャッシュルートは `$PLM_HOME/.plm/cache/plugins/`（`PLM_HOME` 未設定時は `$HOME`）。
- **書き込み**: `.plm-meta.json` は一時ファイル + rename によるアトミック書き込み。破損時は警告を出して `None` 扱いし、次回インストール時に再生成する。
- **更新**: `backup → download → アトミック差し替え → 再デプロイ → メタ更新` の順で行い、失敗時は `.backup/` から復元する。`--all` は全プラグインを prepare → commit の2段階で処理するバッチアトミック方式（詳細は `src/plugin/lifecycle/update.rs`）。

## 関連

- [overview](./overview.md) - アーキテクチャ概要
- [core-design](./core-design.md) - コア設計
- [reference/config](../reference/config.md) - 設定ファイル

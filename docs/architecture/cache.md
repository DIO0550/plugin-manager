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
├── config.toml                     # 設定ファイル
├── plugins.json                    # プラグインキャッシュ
└── cache/
    ├── marketplaces/               # マーケットプレイスキャッシュ
    │   ├── anthropic.json
    │   └── company-tools.json
    └── plugins/                    # プラグインファイルキャッシュ
        ├── company-tools/
        │   ├── formatter/
        │   └── linter/
        ├── anthropic/
        │   ├── formatter/
        │   └── code-review/
        └── github/
            └── owner/
                └── repo/
```

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
    owner/
      repo/
```

## プラグインキャッシュ（plugins.json）

```json
{
  "version": 1,
  "plugins": [
    {
      "name": "code-formatter",
      "source": "company/claude-plugins@v2.1.0",
      "version": "2.1.0",
      "status": "enabled",
      "marketplace": "company-tools",
      "installed_at": "2025-01-15T10:30:00Z",
      "installed_sha": "abc123def456",
      "author": {
        "name": "Dev Team",
        "email": "dev@company.com"
      },
      "components": {
        "skills": ["code-formatter"],
        "agents": ["formatter-agent"],
        "commands": ["format"],
        "hooks": []
      },
      "deployments": {
        "codex": {
          "scope": "personal",
          "enabled": true,
          "paths": ["~/.codex/skills/company-tools/code-formatter"]
        },
        "copilot": {
          "scope": "project",
          "enabled": true,
          "paths": [".github/skills/company-tools/code-formatter"]
        }
      }
    }
  ]
}
```

### フィールド説明

| フィールド | 説明 |
|------------|------|
| `name` | プラグイン名 |
| `source` | GitRepo.raw形式のソース参照 |
| `version` | インストールされているバージョン |
| `status` | `enabled` または `disabled` |
| `marketplace` | マーケットプレイス名（直接インストールの場合は`null`） |
| `installed_at` | インストール日時 |
| `installed_sha` | インストール時のコミットSHA |
| `author` | 作者情報 |
| `components` | 含まれるコンポーネント |
| `deployments` | ターゲットごとの展開情報 |

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
| **更新検知** | `installed_sha`と最新を比較 |
| **永続化** | `source`(raw)からいつでも`GitRepo`を復元可能 |
| **marketplace追跡** | どのmarketplaceからインストールしたかを記録 |

## キャッシュ操作

### 読み込み

```rust
impl PluginCache {
    pub fn load() -> Result<Self> {
        let path = dirs::home_dir()?.join(".plm/plugins.json");
        let content = fs::read_to_string(path)?;
        serde_json::from_str(&content)
    }
}
```

### 保存

```rust
impl PluginCache {
    pub fn save(&self) -> Result<()> {
        let path = dirs::home_dir()?.join(".plm/plugins.json");
        let content = serde_json::to_string_pretty(self)?;
        fs::write(path, content)
    }
}
```

### GitRepo復元

```rust
impl CachedPlugin {
    pub fn git_repo(&self) -> Result<GitRepo> {
        GitRepo::parse(&self.source)
    }
}
```

## 関連

- [overview](./overview.md) - アーキテクチャ概要
- [core-design](./core-design.md) - コア設計
- [reference/config](../reference/config.md) - 設定ファイル

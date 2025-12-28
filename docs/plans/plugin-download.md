# プラグインダウンロード機能 実装計画 v2

## 概要

`plm install` コマンドの内部処理として、GitHubリポジトリまたはMarketplace経由でプラグインをローカルにダウンロードする機能を実装する。

## 実装状況

| ステップ | 状態 | 備考 |
|---------|------|------|
| Step 1: エラー型定義 | 完了 | `src/error.rs` |
| Step 2: GitHubクライアント | 完了 | `src/github/` - `gh` CLI認証対応、コミットSHA取得追加済み |
| Step 3: プラグインマニフェスト | 完了 | `src/plugin/manifest.rs` |
| Step 4: プラグインキャッシュ | 完了 | `src/plugin/cache.rs` - zip展開対応 |
| Step 5: Marketplaceレジストリ | 完了 | `src/marketplace/` |
| Step 6: installコマンド統合 | 完了 | `src/commands/install.rs` - ダウンロード処理実装 |

---

## 実装方針

### ダウンロード元
1. **直接GitHub指定**: `owner/repo[@ref]` 形式
2. **Marketplace経由**: `plugin-name@marketplace-name` 形式

### キャッシュ構成
```
~/.plm/
├── config.toml                           # 設定ファイル
├── plugins.json                          # インストール済みプラグイン管理
└── cache/
    ├── plugins/<plugin-name>/            # ダウンロードしたプラグイン
    │   ├── .claude-plugin/
    │   │   └── plugin.json
    │   ├── skills/
    │   ├── agents/
    │   └── ...
    └── marketplaces/<name>.json          # マーケットプレイスキャッシュ
```

### GitHub認証戦略

プライベートリポジトリへのアクセスには認証が必要。以下の優先順位でトークンを取得する：

```rust
fn get_github_token() -> Option<String> {
    // 1. 環境変数を優先（CI/CD用）
    if let Ok(token) = std::env::var("GITHUB_TOKEN") {
        return Some(token);
    }

    // 2. gh CLI から取得（ローカル開発用）
    if let Ok(output) = Command::new("gh").args(["auth", "token"]).output() {
        if output.status.success() {
            return Some(String::from_utf8_lossy(&output.stdout).trim().to_string());
        }
    }

    None  // パブリックリポジトリのみアクセス可能
}
```

| 優先度 | 方法 | ユースケース |
|-------|------|-------------|
| 1 | `GITHUB_TOKEN` 環境変数 | CI/CD、明示的な設定 |
| 2 | `gh auth token` | ローカル開発（gh CLI認証済み） |
| 3 | なし | パブリックリポジトリのみ |

### バージョン管理戦略

Claude Codeのプラグインは**リリースタグではなくブランチのHEAD**を追跡する：

- インストール時: デフォルトブランチの最新コミットをダウンロード
- 更新検知: コミットSHAの比較で判定
- plugin.jsonの`version`フィールドは表示用

**plugins.json に保存する情報:**
```json
{
  "plugins": [
    {
      "name": "formatter",
      "version": "1.0.0",
      "source": "github:owner/repo",
      "git_ref": "main",
      "commit_sha": "abc123...",
      "installed_at": "2025-01-15T10:30:00Z"
    }
  ]
}
```

**更新チェックの流れ:**
1. GitHub APIでブランチのHEADコミットSHAを取得
2. 保存された`commit_sha`と比較
3. 異なれば更新可能と判定

---

## モジュール構成

```
src/
├── main.rs                   (UPDATE) モジュール追加
├── error.rs                  (DONE)   統一エラー型
├── github/                   (DONE)   gh CLI認証対応、コミットSHA取得追加済み
│   ├── mod.rs
│   └── fetcher.rs                     GitHub APIクライアント
├── plugin/                   (NEW)
│   ├── mod.rs
│   ├── manifest.rs                    plugin.json パーサー
│   └── cache.rs                       キャッシュ管理・zip展開
├── marketplace/              (NEW)
│   ├── mod.rs
│   ├── registry.rs                    Marketplace登録管理
│   └── fetcher.rs                     marketplace.json取得
└── commands/
    └── install.rs            (UPDATE) ダウンロード処理統合
```

---

## Step 2: GitHubクライアント（更新）

**ファイル:** `src/github/fetcher.rs`

### 追加が必要な機能

#### 1. 認証トークン取得の改善

```rust
fn get_github_token() -> Option<String> {
    // 1. 環境変数を優先（CI/CD用）
    if let Ok(token) = std::env::var("GITHUB_TOKEN") {
        return Some(token);
    }

    // 2. gh CLI から取得（ローカル開発用）
    if let Ok(output) = std::process::Command::new("gh")
        .args(["auth", "token"])
        .output()
    {
        if output.status.success() {
            let token = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !token.is_empty() {
                return Some(token);
            }
        }
    }

    None
}
```

#### 2. コミットSHA取得API

```rust
impl GitHubClient {
    /// ブランチの最新コミットSHAを取得
    pub async fn get_branch_sha(&self, repo: &RepoRef, branch: &str) -> Result<String> {
        let url = format!(
            "{}/repos/{}/{}/commits/{}",
            self.base_url, repo.owner, repo.repo, branch
        );
        // ...
    }

    /// ダウンロード時にコミットSHAも返す
    pub async fn download_archive_with_sha(&self, repo: &RepoRef) -> Result<(Vec<u8>, String)> {
        let branch = self.get_default_branch(repo).await?;
        let sha = self.get_branch_sha(repo, &branch).await?;
        let archive = self.download_archive(repo).await?;
        Ok((archive, sha))
    }
}
```

### 現在の実装との差分

| 機能 | 現状 | 追加 |
|------|------|------|
| 認証 | `GITHUB_TOKEN`のみ | `gh auth token`フォールバック |
| コミットSHA | なし | `get_branch_sha()` |
| ダウンロード結果 | `Vec<u8>` | `(Vec<u8>, String)` でSHAも返す |

---

## Step 3: プラグインマニフェスト

**ファイル:** `src/plugin/mod.rs`, `src/plugin/manifest.rs`

### 目的
- `.claude-plugin/plugin.json` のパース
- プラグインメタデータの構造化

### データ構造

```rust
/// plugin.json のスキーマ（docs/plm-plan-v2.md より）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    pub name: String,
    pub version: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub author: Option<Author>,
    #[serde(default)]
    pub homepage: Option<String>,
    #[serde(default)]
    pub repository: Option<String>,
    #[serde(default)]
    pub license: Option<String>,
    #[serde(default)]
    pub keywords: Option<Vec<String>>,

    // コンポーネントパス（プラグインルートからの相対パス）
    #[serde(default)]
    pub commands: Option<String>,
    #[serde(default)]
    pub agents: Option<String>,
    #[serde(default)]
    pub skills: Option<String>,
    #[serde(default)]
    pub hooks: Option<String>,
    #[serde(default, rename = "mcpServers")]
    pub mcp_servers: Option<String>,
    #[serde(default, rename = "lspServers")]
    pub lsp_servers: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Author {
    pub name: String,
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub url: Option<String>,
}
```

### API

```rust
impl PluginManifest {
    /// JSONからパース
    pub fn parse(content: &str) -> Result<Self>;

    /// ファイルから読み込み
    pub fn load(path: &Path) -> Result<Self>;

    /// 含まれるコンポーネントタイプを列挙
    pub fn component_types(&self) -> Vec<ComponentType>;
}
```

### 考慮事項
- `plugin.json` が存在しない場合のフォールバック処理
- 必須フィールド（name, version）のバリデーション

---

## Step 4: プラグインキャッシュ

**ファイル:** `src/plugin/cache.rs`

### 目的
- ダウンロードしたzipアーカイブの展開
- キャッシュディレクトリの管理

### データ構造

```rust
pub struct PluginCache {
    /// キャッシュルート: ~/.plm/cache/plugins/
    cache_dir: PathBuf,
}

/// キャッシュされたプラグイン情報
pub struct CachedPlugin {
    pub name: String,
    pub path: PathBuf,
    pub manifest: PluginManifest,
}
```

### API

```rust
impl PluginCache {
    /// キャッシュマネージャを初期化（ディレクトリ作成含む）
    pub fn new() -> Result<Self>;

    /// プラグインのキャッシュパスを取得
    pub fn plugin_path(&self, name: &str) -> PathBuf;

    /// キャッシュ済みかチェック
    pub fn is_cached(&self, name: &str) -> bool;

    /// zipアーカイブを展開してキャッシュに保存
    /// 戻り値: 展開先ディレクトリ
    pub fn store_from_archive(&self, name: &str, archive: &[u8]) -> Result<PathBuf>;

    /// キャッシュからマニフェストを読み込み
    pub fn load_manifest(&self, name: &str) -> Result<PluginManifest>;

    /// キャッシュから削除
    pub fn remove(&self, name: &str) -> Result<()>;

    /// 全キャッシュをクリア
    pub fn clear(&self) -> Result<()>;
}
```

### 考慮事項
- **zip展開時のディレクトリ構造**: GitHubのzipballは `{repo}-{ref}/` というプレフィックスが付くため、それを除去する必要あり
- **上書き動作**: 既存キャッシュがある場合は削除してから展開
- **ディレクトリ作成**: 親ディレクトリが存在しない場合は作成

### zip展開の詳細

```rust
/// GitHubのzipballは以下の構造:
/// repo-main/
///   ├── .claude-plugin/
///   │   └── plugin.json
///   └── skills/
///
/// これを ~/.plm/cache/plugins/<name>/ に展開:
/// ~/.plm/cache/plugins/<name>/
///   ├── .claude-plugin/
///   │   └── plugin.json
///   └── skills/
```

---

## Step 5: Marketplaceレジストリ

**ファイル:** `src/marketplace/mod.rs`, `src/marketplace/registry.rs`, `src/marketplace/fetcher.rs`

### 目的
- マーケットプレイスの登録・管理
- `marketplace.json` の取得・パース

### データ構造

```rust
/// marketplace.json のスキーマ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketplaceManifest {
    pub name: String,
    #[serde(default)]
    pub owner: Option<MarketplaceOwner>,
    pub plugins: Vec<MarketplacePluginEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketplaceOwner {
    pub name: String,
    #[serde(default)]
    pub email: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketplacePluginEntry {
    pub name: String,
    pub source: PluginSource,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub version: Option<String>,
}

/// プラグインソース
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PluginSource {
    /// 相対パス: "./plugins/plugin-a"
    Local(String),
    /// 外部GitHub: { "source": "github", "repo": "org/repo" }
    External { source: String, repo: String },
}

/// キャッシュされたマーケットプレイス情報
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketplaceCache {
    pub name: String,
    pub fetched_at: DateTime<Utc>,
    pub source: String,
    #[serde(default)]
    pub owner: Option<MarketplaceOwner>,
    pub plugins: Vec<MarketplacePluginEntry>,
}
```

### API

```rust
/// マーケットプレイスレジストリ
pub struct MarketplaceRegistry {
    cache_dir: PathBuf,  // ~/.plm/cache/marketplaces/
}

impl MarketplaceRegistry {
    pub fn new() -> Result<Self>;

    /// キャッシュを取得
    pub fn get(&self, name: &str) -> Result<Option<MarketplaceCache>>;

    /// キャッシュを保存
    pub fn store(&self, cache: &MarketplaceCache) -> Result<()>;

    /// キャッシュを削除
    pub fn remove(&self, name: &str) -> Result<()>;

    /// 全マーケットプレイス一覧
    pub fn list(&self) -> Result<Vec<String>>;

    /// 全マーケットプレイスからプラグインを検索
    pub fn find_plugin(&self, plugin_name: &str) -> Result<Option<(String, MarketplacePluginEntry)>>;
}

/// マーケットプレイス取得
pub struct MarketplaceFetcher {
    github: GitHubClient,
}

impl MarketplaceFetcher {
    /// GitHubリポジトリから marketplace.json を取得
    pub async fn fetch(&self, repo: &RepoRef, subdir: Option<&str>) -> Result<MarketplaceManifest>;
}
```

### 考慮事項
- `marketplace.json` の場所: `.claude-plugin/marketplace.json`
- サブディレクトリ指定対応（`subdir` オプション）
- 外部プラグイン参照の解決

---

## Step 6: installコマンド統合

**ファイル:** `src/commands/install.rs`

### 目的
- 各モジュールを統合してダウンロード処理を実行

### ダウンロードフロー

```
1. 引数パース
   - "owner/repo[@ref]" → GitHub直接
   - "plugin@marketplace" → Marketplace経由
   - "plugin" → 全Marketplaceから検索

2. プラグイン情報取得
   - GitHub直接: リポジトリ情報を使用
   - Marketplace: キャッシュからプラグインエントリを取得

3. ダウンロード
   - GitHubClient.download_archive() でzipを取得

4. キャッシュに保存
   - PluginCache.store_from_archive() で展開・保存

5. マニフェスト読み込み
   - PluginCache.load_manifest() でplugin.jsonをパース

6. 結果を返す
   - CachedPlugin を返して後続処理（デプロイ）に渡す
```

### API

```rust
/// インストール元の指定
pub enum InstallSource {
    /// GitHub直接: owner/repo[@ref]
    GitHub(RepoRef),
    /// Marketplace経由: plugin@marketplace
    Marketplace { plugin: String, marketplace: String },
    /// Marketplace検索: plugin
    Search(String),
}

impl InstallSource {
    /// 引数文字列からパース
    pub fn parse(input: &str) -> Result<Self>;
}

/// ダウンロード実行
pub async fn download_plugin(source: &InstallSource) -> Result<CachedPlugin>;
```

### 考慮事項
- **名前の決定**:
  - GitHub直接の場合はリポジトリ名
  - Marketplace経由の場合はplugin.jsonのname
- **再ダウンロード**: `--force` オプションでキャッシュを無視
- **エラーハンドリング**:
  - リポジトリが見つからない
  - plugin.jsonが存在しない
  - ネットワークエラー（リトライ）

---

## 依存関係

### Cargo.toml（既存）
- `reqwest` 0.12 - HTTPクライアント
- `indicatif` 0.17 - プログレスバー
- `zip` 2 - アーカイブ展開
- `thiserror` 2 - エラー定義
- `chrono` 0.4 - タイムスタンプ
- `serde` / `serde_json` - シリアライズ

### 追加が必要な場合
- `dirs` - ホームディレクトリ取得（`~/.plm/`）

---

## テスト計画

### ユニットテスト
- `RepoRef::parse()` - リポジトリ参照パース（完了）
- `PluginManifest::parse()` - plugin.jsonパース
- `InstallSource::parse()` - インストール元パース

### 結合テスト
- パブリックリポジトリからのダウンロード
- プライベートリポジトリからのダウンロード（GITHUB_TOKEN）
- zip展開とキャッシュ保存

---

## 次のアクション

1. **Step 3**: `src/plugin/manifest.rs` - PluginManifest実装
2. **Step 4**: `src/plugin/cache.rs` - PluginCache実装（zip展開含む）
3. **Step 5**: `src/marketplace/` - Marketplace機能（後回し可能）
4. **Step 6**: `src/commands/install.rs` - 統合

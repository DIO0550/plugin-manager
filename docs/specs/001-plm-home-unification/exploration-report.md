# Codebase Exploration Report: PLM_HOME / HOME 解決の一元化

**探索目的**: `PLM_HOME` 設定時にプラグインキャッシュとレジストリ／設定 JSON が別ルートに分裂する不整合を解消し、PLM 状態（`~/.plm` 相当）のパス解決を単一ポリシー（`plm_root` / `PlmPaths`）に集約する。

---

## 0. エグゼクティブサマリー

**重要な発見（Top 5）**:

1. **バグの分布が明確**: 5 種のファイルが `~/.plm` 配下のパスを自前で構築しているが、そのうち **3 つ（`MarketplaceConfig`・`TargetRegistry`・`ImportRegistry`）が `PLM_HOME` を無視**して生の `std::env::var("HOME")` を使っており、`PackageCache` / `MarketplaceRegistry` の 2 つだけが正しく `EnvVar::get("PLM_HOME")` → `HOME` フォールバックを実装している。
2. **正解パターンはコードベース内に既存**: `PackageCache::new()` と `MarketplaceRegistry::new()` が実装している `EnvVar::get("PLM_HOME").or_else(|| EnvVar::get("HOME"))` が「案 A 相当」の正しいパターン。`plm_root()` 関数はこのロジックの抽出で済む。
3. **テスト注入 API が全 5 つに揃っている**: `with_cache_dir()` / `with_path()` / `load_from()` 系のテスト専用コンストラクタが全レジストリに整備済み。`PlmPaths` にも同様の `with_root(PathBuf)` を追加するだけでテストが書ける。
4. **`MarketplaceConfig` のエラー型が不統一**: 他の 4 つは `crate::error::Result<T>` を返すが `MarketplaceConfig` だけ `Result<Self, String>` を返す。一元化の際の **必須でない** 改善候補だが、ユーザー体験に影響。
5. **Personal スコープ用の `home_dir()` / `resolve_home_dir()` は意図的スコープ外**: `src/target/core/paths.rs` と `src/plugin/cache/cleanup.rs` の両関数は `HOME` のみを使う（Target の Personal 配置用）。本機能では **変更しない**。

**推奨される次のステップ**:
- `src/env.rs` に `plm_root()` 関数（または `PlmPaths` 構造体の `new()` / `with_root()`）を追加し、3 つのバグ箇所を置き換える（TDD: Red → Green）
- `MarketplaceConfig::load()` の `String` エラーは `plm_root()` 導入と同時に `PlmError` 寄せ可能だが、ヒアリングでは必須でないとされているため判断が必要

---

## 1. アーキテクチャ概要

### 1.1 ディレクトリ構造

```
src/
├── main.rs             # エントリポイント（tokio::main）
├── cli.rs              # clap CLI 定義（16コマンド）
├── commands.rs         # コマンドディスパッチ
├── env.rs              # EnvVar ユーティリティ（PLM_HOME 解決の起点）
├── env_test.rs
├── config.rs           # HTTP設定・AuthProvider（PLM_HOME 無関係）
├── path_ext.rs         # PathExt トレイト（Path 拡張）
├── install.rs          # インストール処理（PackageCache を使用）
├── application.rs      # アプリケーション層
├── plugin/
│   ├── cache/
│   │   ├── cache.rs    # PackageCache（PLM_HOME → HOME 済み ✓）
│   │   ├── cache_test.rs
│   │   └── cleanup.rs  # resolve_home_dir（HOME のみ・スコープ外）
│   └── ...
├── marketplace/
│   ├── registry.rs     # MarketplaceRegistry（PLM_HOME → HOME 済み ✓）
│   ├── registry_test.rs
│   └── config.rs       # MarketplaceConfig（HOME のみ ← バグ）
├── import/
│   ├── registry.rs     # ImportRegistry（HOME のみ ← バグ）
│   └── registry_test.rs
├── target/
│   └── core/
│       ├── registry.rs # TargetRegistry（HOME のみ ← バグ）
│       ├── registry_test.rs
│       ├── paths.rs    # home_dir()（HOME のみ・スコープ外）
│       └── paths_test.rs
└── commands/
    └── manage/
        └── marketplace.rs  # MarketplaceConfig + MarketplaceRegistry 両方を呼ぶ
```

**構造の特徴**:
- Feature ベース（domain/application/infrastructure 分離なし）
- テストファイルは `foo_test.rs` として分離（`#[cfg(test)] #[path = "foo_test.rs"]` パターン）

### 1.2 主要ファイル

| ファイルパス | 役割 | 重要度 |
|---|---|---|
| `src/env.rs` | `EnvVar::get()` 提供・`plm_root()` の追加先 | 高 |
| `src/plugin/cache/cache.rs` | `PackageCache` — 正しいパターンの参考実装 | 高 |
| `src/marketplace/registry.rs` | `MarketplaceRegistry` — 正しいパターンの参考実装 | 高 |
| `src/marketplace/config.rs` | `MarketplaceConfig` — `HOME` のみ（バグ修正対象） | 高 |
| `src/import/registry.rs` | `ImportRegistry` — `HOME` のみ（バグ修正対象） | 高 |
| `src/target/core/registry.rs` | `TargetRegistry` — `HOME` のみ（バグ修正対象） | 高 |
| `src/target/core/paths.rs` | `home_dir()` — スコープ外（変更しない） | 中 |
| `src/plugin/cache/cleanup.rs` | `resolve_home_dir()` — スコープ外（変更しない） | 中 |
| `src/commands/manage/marketplace.rs` | 両 registry の呼び出し元 | 中 |

### 1.3 レイヤー構成

```
CLI (clap)
   ↓
commands/ (コマンドハンドラ)
   ↓
registries / config  ← ここが修正の核
   ├── PackageCache         ($PLM_HOME → $HOME → .plm/cache/plugins/)   ✓
   ├── MarketplaceRegistry  ($PLM_HOME → $HOME → .plm/cache/marketplaces/) ✓
   ├── MarketplaceConfig    ($HOME のみ → .plm/marketplaces.json)  ← バグ
   ├── TargetRegistry       ($HOME のみ → .plm/targets.json)       ← バグ
   └── ImportRegistry       ($HOME のみ → .plm/imports.json)       ← バグ
```

### 1.4 依存関係

```
src/env.rs  ←──── PackageCache::new()
                └── MarketplaceRegistry::new()
                (MarketplaceConfig / TargetRegistry / ImportRegistry は未使用 ← バグ)

std::env::var("HOME")  ←── MarketplaceConfig::load()
                         ├── TargetRegistry::new()
                         └── ImportRegistry::new()
```

**循環依存**: なし  
**主要な外部依存**: `clap 4`, `tokio 1`, `reqwest 0.12`, `serde 1`, `serde_json 1`, `tempfile 3`（テスト用）, `assert_cmd 2`（統合テスト）

---

## 2. 関連コード分析

### 2.1 変更対象に関連する既存コード

| ファイルパス | 関連内容 | 関連度 |
|---|---|---|
| `src/env.rs` | `EnvVar::get()` — `plm_root()` の追加先 | 高 |
| `src/plugin/cache/cache.rs` | `PackageCache::new()` — 正しいパターンの参考実装 | 高 |
| `src/marketplace/registry.rs` | `MarketplaceRegistry::new()` — 正しいパターンの参考実装 | 高 |
| `src/marketplace/config.rs` | `MarketplaceConfig::load()` — `HOME` のみバグ | 高 |
| `src/import/registry.rs` | `ImportRegistry::new()` — `HOME` のみバグ | 高 |
| `src/target/core/registry.rs` | `TargetRegistry::new()` — `HOME` のみバグ | 高 |
| `src/target/core/paths.rs` | `home_dir()` — Personal 配置用、スコープ外 | 低 |
| `src/plugin/cache/cleanup.rs` | `resolve_home_dir()` — Personal cleanup 用、スコープ外 | 低 |

### 2.2 再利用可能なパターン

#### パターン: PLM_HOME → HOME フォールバック（PackageCache / MarketplaceRegistry 実装済み）

**場所**: `src/plugin/cache/cache.rs`, `src/marketplace/registry.rs`  
**概要**: `EnvVar::get("PLM_HOME").or_else(|| EnvVar::get("HOME"))` で PLM_HOME 優先の HOME フォールバックを実現。空文字列を自動フィルタ。  
**再利用方法**: このロジックを `plm_root()` として `src/env.rs` に抽出し、全 5 レジストリから呼ぶ。

```rust
// src/plugin/cache/cache.rs （PackageCache::new — 正しいパターン）
pub fn new() -> Result<Self> {
    let fs = RealFs;
    let home = crate::env::EnvVar::get("PLM_HOME")
        .or_else(|| crate::env::EnvVar::get("HOME"))
        .ok_or_else(|| {
            PlmError::Cache(
                "PLM_HOME and HOME environment variables not set or empty".to_string(),
            )
        })?;
    let cache_dir = PathBuf::from(home)
        .join(".plm")
        .join("cache")
        .join("plugins");
    fs.create_dir_all(&cache_dir)?;
    Ok(Self { cache_dir })
}
```

#### パターン: テスト用コンストラクタ（カスタムパス注入）

**場所**: 全 5 レジストリ  
**概要**: 本番コンストラクタ（`new()` / `load()`）とは別に、テスト用のパス注入 API が揃っている。  
**再利用方法**: `PlmPaths::with_root(root: PathBuf)` を同様のパターンで追加。

```rust
// PackageCache — with_cache_dir パターン
pub fn with_cache_dir(cache_dir: PathBuf) -> Result<Self> {
    let fs = RealFs;
    fs.create_dir_all(&cache_dir)?;
    Ok(Self { cache_dir })
}

// MarketplaceRegistry — with_cache_dir パターン（同上）

// TargetRegistry — with_path パターン
pub fn with_path(path: PathBuf) -> Self {
    Self { config_path: path, state: State::Idle, config: None }
}

// ImportRegistry — with_path パターン（同上）

// MarketplaceConfig — load_from パターン
pub fn load_from(path: PathBuf) -> Result<Self, String> { ... }
```

#### パターン: 状態マシン（TargetRegistry / ImportRegistry）

**場所**: `src/target/core/registry.rs`, `src/import/registry.rs`  
**概要**: Idle → Loaded → Modified → Idle の状態遷移で、不整合な操作を防ぎつつ NamedTempFile による耐障害性のある保存を実現。  
**再利用方法**: 新モジュールでも踏襲可能。`PlmPaths` 自体はパス計算のみで状態は持たないため直接は不要。

```rust
// src/target/core/registry.rs の状態マシン例
fn save(&mut self) -> Result<()> {
    // ...
    let parent = self.config_path.parent().unwrap_or(Path::new("."));
    let mut temp_file = NamedTempFile::new_in(parent)
        .map_err(|e| PlmError::TargetRegistry(format!("Failed to create temp file: {}", e)))?;
    let content = serde_json::to_string_pretty(config)?;
    temp_file.write_all(content.as_bytes())?;
    temp_file.persist(&self.config_path)
        .map_err(|e| PlmError::TargetRegistry(format!("Failed to persist config: {}", e)))?;
    self.state = State::Idle;
    Ok(())
}
```

### 2.3 類似実装の参考例

#### 参考: `EnvVar::get()` 空文字フィルタ

**実装ファイル**: `src/env.rs`  
**類似点**: PLM_HOME が設定されているが空文字の場合も `None` として扱い、HOME フォールバックに進む。  
**参考になる点**: `plm_root()` でも同じく空文字を `None` 相当として扱うべき。

```rust
// src/env.rs
pub fn get(key: &str) -> Option<String> {
    std::env::var(key).ok().filter(|s| !s.is_empty())
}
```

#### 参考: `resolve_home_dir()` のエッジケース処理（スコープ外だが参考）

**実装ファイル**: `src/plugin/cache/cleanup.rs`  
**類似点**: 空白のみ / 相対パス / 非 UTF-8 / 空文字を `None` として扱い、Personal scope の誤操作を防ぐ。  
**参考になる点**: `plm_root()` でも相対パス拒否ロジックを入れることを hearing-notes が推奨。

```rust
// src/plugin/cache/cleanup.rs
fn resolve_home_dir() -> Option<PathBuf> {
    let home = std::env::var_os("HOME")?;
    let home = match home.to_str() {
        Some(s) => {
            let trimmed = s.trim();
            if trimmed.is_empty() { return None; }
            PathBuf::from(trimmed)
        }
        None => { /* OsString フォールバック */ }
    };
    home.is_absolute().then_some(home)
}
```

### 2.4 命名規則・コーディングスタイル

- **ファイル命名**: `snake_case`（`registry.rs`, `cache.rs`）、テストは `*_test.rs` に分離
- **変数命名**: `snake_case`（`cache_dir`, `config_path`）、型は `CamelCase`（`PlmPaths`, `PackageCache`）
- **インデント**: 4 スペース（rustfmt 準拠）
- **エラー型**: `crate::error::Result<T>` が基本。`MarketplaceConfig` だけ `Result<Self, String>` という例外がある
- **`*Outcome` 命名**: `AddOutcome`, `RemoveOutcome` — 成果レポート型に `Outcome` 接尾辞（CLAUDE.md 規約）

### 2.6 再探索: [NEW] 項目の類似実装検証

**目的**: 実装計画の [NEW] 項目（`plm_root()` / `PlmPaths struct` / パスアクセサ群）について、類似の既存実装が初回探索で見落とされていないかを確認する。

**追加検索キーワード**: `struct.*Paths`, `CachePaths`, `AppPaths`, `HomePaths`, `RootPath`, `fn plm_root`, `fn get_home`, `fn resolve.*home`, `fn get_root`, `fn app_dir`, `fn state_dir`, `PlmPaths`, `with_root`, `plm_dir`, `join(".plm")`（全ファイル横断）、`utils/`, `helpers/`, `shared/` ディレクトリ探索

#### [NEW-1] `plm_root()` 関数 — **類似実装なし**

コードベース全体で `plm_root` に相当する関数は存在しない。最も近い既存実装:

- `home_dir()` (`src/target/core/paths.rs`) — `HOME` のみ・`PathBuf` 返却（`Result` でない）・Personal 配置専用
- `resolve_home_dir()` (`src/plugin/cache/cleanup.rs`) — `HOME` のみ・`Option<PathBuf>` 返却・cleanup 専用

どちらも `PLM_HOME` を扱わず、用途・シグネチャ・エラー処理が計画の `plm_root() -> Result<PathBuf>` と異なる。**再利用不可**。

```rust
// src/target/core/paths.rs — 「HOMEのみ・フォールバックに"~"を返す」という仕様が plm_root と異なる
pub(crate) fn home_dir() -> PathBuf {
    std::env::var("HOME")
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("~"))  // ← Result でなくフォールバック
}
```

#### [NEW-2] `PlmPaths struct + new/with_root` — **類似実装なし**

コードベース全体でパス集約 struct は以下の 2 つのみ:

- `ScopedPath` (`src/component/model/scoped_path.rs`) — プロジェクトルート配下のトラバーサル防止用型。PLM 状態パスとは無関係。
- `PluginSourcePath` (`src/marketplace/path/plugin_source_path.rs`) — マーケットプレイスのプラグインパス文字列ラッパー。PLM 状態パスとは無関係。

`utils/`, `helpers/`, `shared/`, `lib/` ディレクトリは存在しない。**再利用できる構造体はゼロ**。

#### [NEW-3] パスアクセサ群（`plm_dir/targets_json/marketplaces_json/imports_json/plugins_cache_dir/marketplaces_cache_dir`）— **類似実装なし**

`.join(".plm")` の全出現箇所を横断検索した結果、5 箇所のみ（全て既知のバグ・正解箇所）:

| ファイル | 構築するパス | 状態 |
|---|---|---|
| `src/target/core/registry.rs` | `{HOME}/.plm/targets.json` | バグ（HOME のみ） |
| `src/import/registry.rs` | `{HOME}/.plm/imports.json` | バグ（HOME のみ） |
| `src/marketplace/config.rs` | `{HOME}/.plm/marketplaces.json` | バグ（HOME のみ） |
| `src/plugin/cache/cache.rs` | `{PLM_HOME\|HOME}/.plm/cache/plugins` | 正解パターン |
| `src/marketplace/registry.rs` | `{PLM_HOME\|HOME}/.plm/cache/marketplaces` | 正解パターン |

各箇所が個別にパス文字列を組み立てており、**集約アクセサは存在しない**。計画通り `PlmPaths` への集約が必要。

#### 計画の「PackageCache のパターンを抽出」方針の評価

初回探索で特定した `PackageCache::new()` / `MarketplaceRegistry::new()` の `EnvVar::get("PLM_HOME").or_else(|| EnvVar::get("HOME"))` パターンが、**コードベース内で唯一の正解実装**であることを再確認。

計画が「このロジックを `plm_root()` として `src/env.rs` に抽出する」としている方針は **十分かつ正確**。他に抽出・再利用すべき隠れた実装は存在しない。

**結論: 3つの [NEW] 項目すべてについて類似の既存実装は存在しない。計画の新規作成は妥当。**

---

### 2.5 既存構造の問題点・技術的負債

| 問題のあるパターン / 場所 | 問題の内容 | 新実装での扱い |
|---|---|---|
| `MarketplaceConfig::load()` / `src/marketplace/config.rs:32` | `std::env::var("HOME")` 直接使用・PLM_HOME を無視・エラー型が `String`（不統一） | 避ける。`plm_root()` を使い、`PlmError` 寄せも検討 |
| `TargetRegistry::new()` / `src/target/core/registry.rs:104` | `std::env::var("HOME")` 直接使用・PLM_HOME を無視 | 避ける。`plm_root()` を使う |
| `ImportRegistry::new()` / `src/import/registry.rs:93` | `std::env::var("HOME")` 直接使用・PLM_HOME を無視 | 避ける。`plm_root()` を使う |
| 各 `::new()` 内でのパス文字列リテラル重複 | `".plm"`, `"cache"`, `"plugins"` 等がコードに散在 | 避ける。`PlmPaths` に集約する |

**「避ける」とした場合の代替方針**: 新実装では `EnvVar::get("PLM_HOME").or_else(|| EnvVar::get("HOME"))` ロジックを `plm_root()` として `src/env.rs` に抽出し、全 5 つの `::new()` / `::load()` から呼ぶ。パス文字列は `PlmPaths` に集約。

---

## 3. 技術的制約・リスク

### 3.1 既存の制約

**型システム・リンター**:
- Rust 2021 edition。`cargo clippy -- -D warnings` で警告をエラー扱い（CI 設定）
- `pub(crate)` / `pub` の可視性を適切に設定する必要がある（`TargetsConfig` は `pub(crate)`）
- `EnvVar` は `pub struct` として公開されているため `plm_root()` も同モジュールに追加可能

**ビルド設定**:
- ビルドツール: `cargo build` / `cargo build --release`
- テスト: `cargo test`（CI では `cargo build` → `cargo test` の順序）

### 3.2 互換性の問題

| ライブラリ | バージョン | リスク |
|---|---|---|
| `tempfile` | 3 | テスト用 TempDir が全テストで使用中。`PlmPaths::with_root()` でも同様に使用可能 |
| `serde_json` | 1 | JSON シリアライズ。既存パターンと同一 |
| `thiserror` | 2 | `PlmError` 定義に使用。`MarketplaceConfig` のエラー型変更時に関係 |

**`MarketplaceConfig` エラー型リスク**: `load()` の戻り値型が `Result<Self, String>` であり、呼び出し元 `commands/manage/marketplace.rs` が `.map_err(|e| e.to_string())` / `?` で処理している。`PlmError` に変更する場合は呼び出し元の変更も必要。

### 3.3 パフォーマンスボトルネック

- 環境変数読み取りのみ（起動時 1 回）。パフォーマンス問題なし（hearing-notes で「対象外」）

### 3.4 セキュリティ考慮点

- `PLM_HOME` に相対パスが渡された場合の挙動: hearing-notes では「拒否してエラー」を推奨
- `resolve_home_dir()` が `is_absolute()` チェックを行っているパターンが参考になる
- `PlmPaths` に相対パス拒否チェックを入れる必要あり

---

## 4. 変更影響範囲

### 4.1 波及ファイル

**直接影響**（修正が必須）:

| ファイルパス | 理由 | 影響の種類 |
|---|---|---|
| `src/env.rs` | `plm_root()` / `PlmPaths` の追加先 | 追加 |
| `src/marketplace/config.rs` | `MarketplaceConfig::load()` の HOME 解決をバグ修正 | 修正 |
| `src/import/registry.rs` | `ImportRegistry::new()` の HOME 解決をバグ修正 | 修正 |
| `src/target/core/registry.rs` | `TargetRegistry::new()` の HOME 解決をバグ修正 | 修正 |
| `src/plugin/cache/cache.rs` | `PackageCache::new()` — `plm_root()` 抽出後に呼び出し側を変更 | 修正（ロジック統一） |
| `src/marketplace/registry.rs` | `MarketplaceRegistry::new()` — 同上 | 修正（ロジック統一） |

**間接影響**（確認が必要）:

| ファイルパス | 理由 | 確認内容 |
|---|---|---|
| `src/commands/manage/marketplace.rs` | `MarketplaceConfig::load()` のエラー型が変わる場合 | `String` → `PlmError` 対応 |
| `src/commands/manage/target.rs` | `TargetRegistry::new()` 呼び出し元 | エラーハンドリング影響なし（`Result<>` 戻り値は同じ）のはずだが要確認 |
| `src/source.rs`, `src/source/marketplace_source.rs` | `MarketplaceRegistry::new()` / `PackageCache::new()` 呼び出し元 | パス解決の変更が正しく反映されるか |
| `src/install.rs` | `PackageCache::new()` 呼び出し元 | 同上 |
| `src/tui/manager/core/data.rs` | 各 registry の `new()` 呼び出し元 | 同上 |

### 4.2 テスト範囲

**既存テストファイル**:

| テストファイルパス | テスト対象 | 修正の必要性 |
|---|---|---|
| `src/env_test.rs` | `EnvVar::get()` | 中（`plm_root()` の新テスト追加） |
| `src/plugin/cache/cache_test.rs` | `PackageCache` | 低（`with_cache_dir()` で分離済み、`new()` テストは追加） |
| `src/marketplace/registry_test.rs` | `MarketplaceRegistry` | 低（同上） |
| `src/target/core/registry_test.rs` | `TargetRegistry` | 低（`with_path()` で分離済み） |
| `src/import/registry_test.rs` | `ImportRegistry` | 低（同上） |
| `src/commands/lifecycle/disable_test.rs` | `plm disable`（統合テスト） | 中（PLM_HOME を環境変数で設定するテストを追加） |
| `src/commands/lifecycle/enable_test.rs` | `plm enable`（統合テスト） | 中（同上） |

**新規テストの必要性**:
- [x] ユニットテスト: `plm_root()` — PLM_HOME 設定あり / なし / 空 / 相対パス / 両方未設定
- [x] ユニットテスト: `PlmPaths` のパス計算（`targets_json` / `marketplaces_json` / `imports_json` / `plugins_cache_dir` / `marketplaces_cache_dir`）
- [x] ユニットテスト: `PLM_HOME` 設定下でも Personal 配置（`home_dir()`）が `HOME` を使う回帰テスト
- [ ] 統合テスト: `PLM_HOME` 設定時に全 PLM 状態が同一ルートに収まることを検証（`plm install` ← `plm list` 等）

### 4.3 破壊的変更の可能性

| API / 関数 | 変更内容 | 影響範囲 |
|---|---|---|
| `MarketplaceConfig::load()` | `HOME` → `PLM_HOME` 優先（動作変更） | `marketplace` コマンド使用者（PLM_HOME 設定時のみ） |
| `TargetRegistry::new()` | 同上 | `target` コマンド使用者（同上） |
| `ImportRegistry::new()` | 同上 | `import` コマンド使用者（同上） |
| `MarketplaceConfig` エラー型 | `String` → `PlmError`（任意） | `commands/manage/marketplace.rs` の `.map_err(|e| e.to_string())` |

### 4.4 移行計画の必要性

- 段階的リリース: 不要（`PLM_HOME` 未設定のデフォルト動作は変わらない）
- ロールバック計画: Git リバートで十分

---

## 5. テストインフラストラクチャ

### 5.1 テスト環境

- **テストフレームワーク**: `cargo test`（標準）
- **テストランナーコマンド**: `cargo test` / `cargo test <test_name>`
- **アサーションライブラリ**: 標準 `assert!`, `assert_eq!`（ユニット）; `assert_cmd` + `predicates`（統合テスト）
- **モックライブラリ**: 手書きモック（`MockHostClient` 等）。`vi.mock` 相当はなし。テスト注入は `with_path()` / `with_cache_dir()` / `load_from()` パターン

### 5.2 テストファイル構成

- **配置パターン**: `foo.rs` のテストは `foo_test.rs` に分離（`#[cfg(test)] #[path = "foo_test.rs"] mod tests;`）
- **命名規則**: `*_test.rs`（ユニット）、`tests/` ディレクトリ（統合テストは `assert_cmd` 使用）
- **テストヘルパー**: TempDir ファクトリ関数（`fn create_test_registry() -> (TargetRegistry, TempDir)` 等）

### 5.3 既存テストパターン

| テストファイル | テスト対象 | パターン |
|---|---|---|
| `src/env_test.rs` | `EnvVar::get()` | Unit — `std::env::set_var` / `remove_var` で環境変数を制御 |
| `src/target/core/registry_test.rs` | `TargetRegistry` | Unit — `TempDir` + `with_path()` で分離 |
| `src/import/registry_test.rs` | `ImportRegistry` | Unit — `TempDir` + `with_path()` で分離 |
| `src/plugin/cache/cache_test.rs` | `PackageCache` | Unit — `TempDir` + `with_cache_dir()` で分離 |
| `src/commands/lifecycle/disable_test.rs` | `plm disable` | Integration — `assert_cmd::Command` + `.env("HOME", ...)` / `.env_remove("PLM_HOME")` |
| `src/commands/lifecycle/enable_test.rs` | `plm enable` | Integration — 同上 |

**統合テストで確立された環境変数パターン**（再利用必須）:

```rust
// src/commands/lifecycle/disable_test.rs（統合テスト）
fn test_disable_cache_not_found_shows_error_once() {
    let home = TempDir::new().unwrap();
    plm()
        .env_remove("PLM_HOME")       // PLM_HOME を確実にクリア
        .env("HOME", home.path())     // HOME を TempDir に向ける
        .args(["disable", "nonexistent-plugin"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found in cache").count(1));
}
```

`PLM_HOME` のテストでは逆パターン（`.env_remove("HOME").env("PLM_HOME", temp_dir)` または両方設定して PLM_HOME が優先されることを検証）が新規テストの雛形になる。

**ユニットテストで確立された環境変数パターン**:

```rust
// src/env_test.rs — 環境変数操作パターン
#[test]
fn test_get_existing_var() {
    std::env::set_var("TEST_ENV_VAR", "test_value");
    assert_eq!(EnvVar::get("TEST_ENV_VAR"), Some("test_value".to_string()));
    std::env::remove_var("TEST_ENV_VAR");
}
```

> 注意: ユニットテストでの `std::env::set_var` は並列テスト実行で競合するリスクがある。`plm_root()` のユニットテストは `cargo test -- --test-threads=1` か、環境変数操作を避けて `with_root()` 経由のテストを優先すること。

### 5.4 カバレッジ・CI

- **カバレッジツール**: 設定ファイル未確認（`.nycrc` / `lcov.info` なし）
- **CI テストジョブ**: `.github/workflows/ci.yml` の `test` ジョブが `cargo build && cargo test` を実行。`check` / `fmt` / `clippy` / `audit` の 5 ジョブが並列実行

---

## 6. 追加調査が必要な項目

- [ ] `src/commands/manage/target.rs` の `TargetRegistry::new()` 呼び出し箇所（エラーハンドリングの詳細確認）
- [ ] `src/tui/manager/core/data.rs` での各 registry 使用パターン（TUI からの呼び出しが `PLM_HOME` 設定下で正しく動くか）
- [ ] 統合テスト（`tests/` ディレクトリ）の有無 — `assert_cmd` は `src/commands/lifecycle/` 内のインラインテストでのみ確認。`tests/` ディレクトリは未探索
- [ ] `MarketplaceConfig` エラー型を `PlmError` に変える場合の呼び出し元の全数調査（`commands/manage/marketplace.rs` 以外の呼び出し箇所）
- [ ] `docs/reference/config.md` / `docs/architecture/cache.md` の存在確認（ドキュメント更新対象）

---

## 7. ユーザー判断が必要な論点

> このセクションは探索中に浮かんだ、ユーザー判断が必要そうな論点のメモ。
> Step 3.5 で requirements.md の「未解決の確認事項」へ転記して解消する。論点が無ければ「該当なし」と記載する。

hearing-notes / 事前レビュー（Issue #344）で以下は確定済みのため、requirements の未解決確認事項には載せない:

1. **`PLM_HOME` セマンティクス**: 案 A（HOME 代替）。`plm_root = PLM_HOME ?? HOME`、`plm_dir = {plm_root}/.plm`
2. **対象境界**: `paths.rs` / `cleanup.rs` は対象外（ユーザー HOME）
3. **`MarketplaceConfig` エラー型**: 本 Issue では `String` のまま維持（必須ではない。呼び出し元変更を避ける）
4. **相対パス**: `plm_root` は相対パスを拒否してエラー

---

## 8. 探索メトリクス（自己検証用）

| 指標 | 基準 | 初回実績 | 再探索後 |
|---|---|---|---|
| Read したファイル数 | 10 以上 | 16 | 21（+5: `env.rs`, `env_test.rs`, `target/env.rs`, `path_ext.rs`, `application.rs`, `cleanup.rs`全文） |
| Grep 検索キーワード数 | 5 以上 | 8 | 21（+13: `struct.*Paths`, `CachePaths`, `fn plm_root`, `fn get_home`, `fn get_root`, `fn app_dir`, `fn state_dir`, `PlmPaths`, `with_root`, `plm_dir`, `join(".plm")`, `utils/`, `helpers/` 等） |
| コードスニペット数 | 5 以上 | 7 | 9（+2: `home_dir()` 全文、再探索確認コード） |
| 逆引き検索実施 | 必須 | 実施済み | 実施済み（全 join(".plm") 箇所横断確認） |
| [NEW] 類似実装発見数 | — | — | 0（3項目すべて類似なし） |

**再探索追加キーワード**: `struct.*Paths`, `CachePaths`, `AppPaths`, `HomePaths`, `RootPath`, `fn plm_root`, `fn get_home`, `fn resolve.*home`, `fn get_root`, `fn app_dir`, `fn state_dir`, `PlmPaths`, `with_root`, `plm_dir`, `join(".plm")`, `utils/` ディレクトリ, `helpers/` ディレクトリ, `shared/` ディレクトリ

**探索キーワード一覧（全）**: `PLM_HOME`, `plm_root`, `PlmPaths`, `plm_dir`, `EnvVar`, `home_dir`, `resolve_home_dir`, `std::env::var("HOME")`, `PackageCache::new`, `MarketplaceRegistry::new`, `ImportRegistry::new`, `TargetRegistry::new`, `MarketplaceConfig::load`, `\.plm`, `crate::env::EnvVar`, `TODO|FIXME|HACK`, `struct.*Paths`, `CachePaths`, `fn get_home`, `fn get_root`, `join(".plm")`（全体横断）

**Read したファイル一覧**:
- `src/env.rs`
- `src/env_test.rs`
- `src/plugin/cache/cache.rs`
- `src/marketplace/registry.rs`
- `src/marketplace/config.rs`
- `src/import/registry.rs`
- `src/target/core/registry.rs`
- `src/target/core/registry_test.rs`
- `src/target/core/paths.rs`
- `src/target/core/paths_test.rs`
- `src/plugin/cache/cleanup.rs`
- `src/config.rs`
- `src/application.rs`
- `src/path_ext.rs`
- `src/plugin/cache/cache_test.rs`（先頭 60 行）
- `src/marketplace/registry_test.rs`（先頭 60 行）
- `src/import/registry_test.rs`
- `src/commands/lifecycle/disable_test.rs`
- `src/commands/lifecycle/enable_test.rs`
- `src/commands/manage/marketplace.rs`（部分）
- `src/install.rs`（先頭 60 行）
- `.github/workflows/ci.yml`
- `Cargo.toml`（先頭 60 行）

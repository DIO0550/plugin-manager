# PLM_HOME / HOME 解決の一元化

**関連Issue**: #344（関連: #333 config.toml）

`PLM_HOME` 設定時にプラグインキャッシュとレジストリ／設定 JSON が別ルートに分裂する不整合を解消するため、PLM 状態パスの解決を `plm_root()` / `PlmPaths` に集約する。バグ箇所（`MarketplaceConfig` / `TargetRegistry` / `ImportRegistry`）が `std::env::var("HOME")` を直接使って `PLM_HOME` を無視している問題を修正し、全 5 レジストリが同一の解決ポリシーを用いるよう統一する。

## ユーザーレビューが必要な点

> **NOTE**
> - 破壊的変更は **PLM_HOME を設定しているユーザーのみ** に影響します（未設定時の実効パスは不変）。
> - `MarketplaceConfig` の戻り値型 `Result<_, String>` は本 Issue で維持（C-4 制約）。内部で `PlmPaths::new()` を呼ぶが `map_err(|e| e.to_string())` で変換します。
> - `src/target/core/paths.rs` と `src/plugin/cache/cleanup.rs` は **変更しません**（Personal 配置用 HOME は意図的スコープ外）。

---

## システム図

### 状態マシン / フロー図

```
                    plm_root() 呼び出し
                           │
                           ▼
              ┌────────────────────────┐
              │ EnvVar::get("PLM_HOME")│
              │ （空文字フィルタ済み）  │
              └────────────────────────┘
                           │
               ┌───────────┴───────────┐
               ▼                       ▼
           Some(val)                 None
               │                       │
         trim → 空チェック             │
               │                       │
         ┌─────┴─────┐                 │
         ▼           ▼                 │
      非空文字     空白のみ            │
         │           │                 │
         │           └─────────────────┤
         │                             ▼
         │              ┌────────────────────────┐
         │              │  EnvVar::get("HOME")   │
         │              │ （空文字フィルタ済み）  │
         │              └────────────────────────┘
         │                             │
         │               ┌─────────────┴─────────────┐
         │               ▼                             ▼
         │           Some(val)                       None
         │               │                             │
         │         trim → 空チェック                   │
         │               │                             │
         │         ┌─────┴─────┐                      │
         │         ▼           ▼                       │
         │      非空文字     空白のみ                  │
         │         │           └────────────────────── ┤
         │         │                                   ▼
         │         │                   PlmError（両方無効: 明確なエラー）
         │         │
         ├─────────┘
         ▼
  PathBuf::from(value)
         │
   is_absolute() ?
         │
    ┌────┴──────────────┐
    ▼                   ▼
  true                false
    │                   │
    ▼                   ▼
plm_root 確定    PlmError（相対パス拒否: "PLM_HOME/HOME must be absolute path"）
    │
    ▼
PlmPaths { root: PathBuf }
    │
    ├─ plm_dir()              → {root}/.plm
    ├─ targets_json()         → {root}/.plm/targets.json
    ├─ marketplaces_json()    → {root}/.plm/marketplaces.json
    ├─ imports_json()         → {root}/.plm/imports.json
    ├─ plugins_cache_dir()    → {root}/.plm/cache/plugins
    └─ marketplaces_cache_dir()→ {root}/.plm/cache/marketplaces
```

### データフロー

```
CLIコマンド実行（plm install / plm target add / plm import ...）
         │
         ▼
各 Registry / Config の new() / load()
         │
         ▼
    PlmPaths::new()                    ← 全 5 レジストリの統一入口
         │
         ▼
    plm_root()  [src/env.rs]
         │
    ┌────┴──────────────────────────────┐
    ▼                                   ▼
EnvVar::get("PLM_HOME")          EnvVar::get("HOME")
（trim + 空白チェック）          （フォールバック）
    │                                   │
    └─────────────┬─────────────────────┘
                  ▼
         is_absolute() チェック
                  │
         ┌────────┴────────┐
         ▼                 ▼
       OK                Err
         │                 │
         ▼                 ▼
  PlmPaths { root }   エラー返却
         │
    ┌────┼────────────────────────────────────────┐
    │    │                                        │
    ▼    ▼                                        ▼
PackageCache           MarketplaceRegistry    MarketplaceConfig
plugins_cache_dir()    marketplaces_cache_dir() marketplaces_json()
    │                         │                        │
    └─────────────────────────┴────────────────────────┘
                  ▼
    全 PLM 状態が {PLM_HOME ?? HOME}/.plm/ 配下に収まる
                  │
    ┌─────────────┴──────────────────┐
    ▼                                ▼
TargetRegistry                  ImportRegistry
targets_json()                  imports_json()
```

---

## 変更案

### カテゴリ 1: `plm_root()` / `PlmPaths` の追加（新規 API）

#### [MODIFY] `src/env.rs`

`plm_root()` 関数と `PlmPaths` 構造体を追加する。`EnvVar::get()` と密接に連携するため同ファイルに配置（C-3 制約準拠）。

- `plm_root()`: `PLM_HOME`（有効時）→ `HOME` の順で解決、相対パス拒否
- `PlmPaths::new()`: 本番コンストラクタ（`plm_root()` を内部で呼ぶ）
- `PlmPaths::with_root(root: PathBuf)`: テスト用コンストラクタ
- 各パスアクセサ: `plm_dir` / `targets_json` / `marketplaces_json` / `imports_json` / `plugins_cache_dir` / `marketplaces_cache_dir`

```rust
// src/env.rs（追加分）

use std::path::PathBuf;
use crate::error::{PlmError, Result};

/// PLM の状態ルートディレクトリを返す（案 A: HOME 代替セマンティクス）
///
/// 解決順: `PLM_HOME`（有効時 = 非空・非空白・絶対パス）→ `HOME`
/// - 空・空白のみは無効扱い（`EnvVar::get` 互換 + trim）
/// - 相対パスはエラーとして返す
/// - 両方無効なら明確なエラー
pub(crate) fn plm_root() -> Result<PathBuf> {
    let raw = EnvVar::get("PLM_HOME")
        .filter(|s| !s.trim().is_empty())
        .or_else(|| EnvVar::get("HOME").filter(|s| !s.trim().is_empty()))
        .ok_or_else(|| {
            // 実在バリアントのみ使用（`PlmError::Config` は存在しない）。
            // 共有リゾルバのため `General` を返し、呼び出し側でドメイン別バリアントへマップ可能。
            PlmError::General(
                "PLM_HOME and HOME environment variables are not set or empty".to_string(),
            )
        })?;

    let path = PathBuf::from(raw.trim());
    if !path.is_absolute() {
        return Err(PlmError::General(format!(
            "PLM_HOME/HOME must be an absolute path, got: {}",
            path.display()
        )));
    }
    Ok(path)
}

/// PLM 状態ファイル群のパスを集約する値オブジェクト
///
/// `new()` で本番環境変数から構築、`with_root()` でテスト用パスを注入する。
/// バイナリクレートのため可視性は `pub(crate)`（過剰公開を避ける）。
pub(crate) struct PlmPaths {
    root: PathBuf,
}

impl PlmPaths {
    /// 環境変数（PLM_HOME → HOME）からパスを解決して構築する
    pub(crate) fn new() -> Result<Self> {
        Ok(Self { root: plm_root()? })
    }

    /// カスタムルートを指定して構築する（テスト用）
    ///
    /// # Arguments
    /// * `root` - PLM の状態ルートディレクトリ（`~` 相当）
    pub(crate) fn with_root(root: PathBuf) -> Self {
        Self { root }
    }

    /// PLM の状態ディレクトリ: `{root}/.plm`
    pub(crate) fn plm_dir(&self) -> PathBuf {
        self.root.join(".plm")
    }

    /// ターゲットレジストリのパス: `{plm_dir}/targets.json`
    pub(crate) fn targets_json(&self) -> PathBuf {
        self.plm_dir().join("targets.json")
    }

    /// マーケットプレイス設定のパス: `{plm_dir}/marketplaces.json`
    pub(crate) fn marketplaces_json(&self) -> PathBuf {
        self.plm_dir().join("marketplaces.json")
    }

    /// インポートレジストリのパス: `{plm_dir}/imports.json`
    pub(crate) fn imports_json(&self) -> PathBuf {
        self.plm_dir().join("imports.json")
    }

    /// プラグインキャッシュディレクトリ: `{plm_dir}/cache/plugins`
    pub(crate) fn plugins_cache_dir(&self) -> PathBuf {
        self.plm_dir().join("cache").join("plugins")
    }

    /// マーケットプレイスキャッシュディレクトリ: `{plm_dir}/cache/marketplaces`
    pub(crate) fn marketplaces_cache_dir(&self) -> PathBuf {
        self.plm_dir().join("cache").join("marketplaces")
    }
}
```

---

### カテゴリ 2: バグ修正（`std::env::var("HOME")` 直接使用の置き換え）

#### [MODIFY] `src/marketplace/config.rs`

`MarketplaceConfig::load()` の HOME 直接参照を `PlmPaths` 経由に変更する。エラー型は `Result<_, String>` のまま維持（C-4）。

- `std::env::var("HOME")` → `crate::env::PlmPaths::new().map_err(|e| e.to_string())`

```rust
// before:
pub fn load() -> Result<Self, String> {
    let home = std::env::var("HOME").map_err(|_| "HOME environment variable not set")?;
    let path = PathBuf::from(home).join(".plm").join("marketplaces.json");
    Self::load_from(path)
}
```

```rust
// after:
pub fn load() -> Result<Self, String> {
    let paths = crate::env::PlmPaths::new().map_err(|e| e.to_string())?;
    Self::load_from(paths.marketplaces_json())
}
```

#### [MODIFY] `src/target/core/registry.rs`

`TargetRegistry::new()` の HOME 直接参照を `PlmPaths` 経由に変更する。

- `std::env::var("HOME")` → `crate::env::PlmPaths::new()?`

```rust
// before:
pub fn new() -> Result<Self> {
    let home = std::env::var("HOME").map_err(|_| {
        PlmError::TargetRegistry("HOME environment variable not set".to_string())
    })?;
    let config_path = PathBuf::from(home).join(".plm").join("targets.json");
    Ok(Self {
        config_path,
        state: State::Idle,
        config: None,
    })
}
```

```rust
// after:
pub fn new() -> Result<Self> {
    // ドメイン文脈を保持するため General → TargetRegistry にマップ
    let paths = crate::env::PlmPaths::new()
        .map_err(|e| PlmError::TargetRegistry(e.to_string()))?;
    Ok(Self {
        config_path: paths.targets_json(),
        state: State::Idle,
        config: None,
    })
}
```

#### [MODIFY] `src/import/registry.rs`

`ImportRegistry::new()` の HOME 直接参照を `PlmPaths` 経由に変更する。

- `std::env::var("HOME")` → `crate::env::PlmPaths::new()` + `ImportRegistry` への map_err

```rust
// before:
pub fn new() -> Result<Self> {
    let home = std::env::var("HOME").map_err(|_| {
        PlmError::ImportRegistry("HOME environment variable not set".to_string())
    })?;
    let config_path = PathBuf::from(home).join(".plm").join("imports.json");
    Ok(Self {
        config_path,
        state: State::Idle,
        config: None,
    })
}
```

```rust
// after:
pub fn new() -> Result<Self> {
    let paths = crate::env::PlmPaths::new()
        .map_err(|e| PlmError::ImportRegistry(e.to_string()))?;
    Ok(Self {
        config_path: paths.imports_json(),
        state: State::Idle,
        config: None,
    })
}
```

---

### カテゴリ 3: 既存正解パターンの統一（ロジック抽出）

#### [MODIFY] `src/plugin/cache/cache.rs`

`PackageCache::new()` の `EnvVar::get("PLM_HOME").or_else(...)` を `PlmPaths::new()` に統一する。

```rust
// before:
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

```rust
// after:
pub fn new() -> Result<Self> {
    let fs = RealFs;
    // 従来どおり Cache バリアントでドメイン文脈を保持（General からの map）
    let paths = crate::env::PlmPaths::new()
        .map_err(|e| PlmError::Cache(e.to_string()))?;
    let cache_dir = paths.plugins_cache_dir();
    fs.create_dir_all(&cache_dir)?;
    Ok(Self { cache_dir })
}
```

#### [MODIFY] `src/marketplace/registry.rs`

`MarketplaceRegistry::new()` の `EnvVar::get("PLM_HOME").or_else(...)` を `PlmPaths::new()` に統一する。

```rust
// before:
pub fn new() -> Result<Self> {
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
        .join("marketplaces");

    fs::create_dir_all(&cache_dir)?;

    Ok(Self { cache_dir })
}
```

```rust
// after:
pub fn new() -> Result<Self> {
    let paths = crate::env::PlmPaths::new()
        .map_err(|e| PlmError::Cache(e.to_string()))?;
    let cache_dir = paths.marketplaces_cache_dir();
    fs::create_dir_all(&cache_dir)?;
    Ok(Self { cache_dir })
}
```

---

### カテゴリ 4: テストファイルの追加・拡張

#### [MODIFY] `src/env_test.rs`

`plm_root()` と `PlmPaths` のユニットテストを追加する。既存の `EnvVar::get()` テストは維持。

- **注意**: 環境変数操作（`std::env::set_var`）は並列テストで競合するリスクがある。`PlmPaths::with_root()` によるパス計算テストを優先し、`plm_root()` のユニットテストは `#[serial_test]` または `-- --test-threads=1` で保護する（NFR-4）。
- 環境変数操作テストは必要最小限（正常系 1 ケース・フォールバック 1 ケース・エラー系）にとどめ、パス計算の大半は `with_root()` 経由で実施。

```rust
// src/env_test.rs（追加分の骨格）

use super::*;
use std::path::PathBuf;
use tempfile::TempDir;

// ── PlmPaths: with_root() によるパス計算テスト（環境変数操作なし・並列安全）──

#[test]
fn test_plm_paths_plm_dir() { /* {root}/.plm */ }

#[test]
fn test_plm_paths_targets_json() { /* {root}/.plm/targets.json */ }

#[test]
fn test_plm_paths_marketplaces_json() { /* {root}/.plm/marketplaces.json */ }

#[test]
fn test_plm_paths_imports_json() { /* {root}/.plm/imports.json */ }

#[test]
fn test_plm_paths_plugins_cache_dir() { /* {root}/.plm/cache/plugins */ }

#[test]
fn test_plm_paths_marketplaces_cache_dir() { /* {root}/.plm/cache/marketplaces */ }

// ── plm_root(): 環境変数制御テスト（test-threads=1 で実行）──

#[test]
fn test_plm_root_uses_plm_home_when_set() { /* PLM_HOME 有効 → PLM_HOME を返す */ }

#[test]
fn test_plm_root_falls_back_to_home() { /* PLM_HOME なし → HOME */ }

#[test]
fn test_plm_root_ignores_empty_plm_home() { /* PLM_HOME="" → HOME フォールバック */ }

#[test]
fn test_plm_root_ignores_whitespace_only_plm_home() { /* PLM_HOME="  " → HOME フォールバック */ }

#[test]
fn test_plm_root_plm_home_takes_priority_over_home() { /* 両方設定 → PLM_HOME 優先 */ }

#[test]
fn test_plm_root_rejects_relative_plm_home() { /* PLM_HOME="relative/path" → Err */ }

#[test]
fn test_plm_root_rejects_relative_home() { /* HOME="relative/path", PLM_HOME なし → Err */ }

#[test]
fn test_plm_root_errors_when_both_unset() { /* 両方未設定 → Err */ }
```

---

## 検証計画

### テスト戦略

**機能タイプ**: Pure Logic（主）/ Configuration（副）

**テスト方針**: TDD（Red-Green-Refactor サイクル）

**根拠**: `plm_root()` はバリデーション・フォールバック・優先順位解決を含む純粋ロジック（Pure Logic）であり、hearing-notes.md にも「TDD」が明記されている。test-design-patterns.md の決定フローで Pure Logic → TDD 推奨に該当。

---

### 自動テスト

#### `src/env_test.rs`

**役割**: `plm_root()` の環境変数解決ロジック（優先順位・フォールバック・バリデーション）および `PlmPaths` のパス計算結果を検証する。

| カテゴリ | テストケース | ユースケース / 想定シナリオ | 期待結果 |
|----------|------------|--------------------------|---------|
| 正常系 | PLM_HOME 有効時の plm_root() | PLM_HOME に絶対パスを設定して呼び出す | PLM_HOME の PathBuf を返す |
| 正常系 | PLM_HOME 未設定時の HOME フォールバック | PLM_HOME を unset、HOME に絶対パスを設定 | HOME の PathBuf を返す |
| 正常系 | PlmPaths::plm_dir() の計算 | `with_root("/tmp/foo")` でパス確認 | `/tmp/foo/.plm` |
| 正常系 | PlmPaths::targets_json() の計算 | `with_root("/tmp/foo")` でパス確認 | `/tmp/foo/.plm/targets.json` |
| 正常系 | PlmPaths::marketplaces_json() の計算 | `with_root("/tmp/foo")` でパス確認 | `/tmp/foo/.plm/marketplaces.json` |
| 正常系 | PlmPaths::imports_json() の計算 | `with_root("/tmp/foo")` でパス確認 | `/tmp/foo/.plm/imports.json` |
| 正常系 | PlmPaths::plugins_cache_dir() の計算 | `with_root("/tmp/foo")` でパス確認 | `/tmp/foo/.plm/cache/plugins` |
| 正常系 | PlmPaths::marketplaces_cache_dir() の計算 | `with_root("/tmp/foo")` でパス確認 | `/tmp/foo/.plm/cache/marketplaces` |
| 正常系 | PLM_HOME と HOME 両方設定時の優先順位 | 両方に有効な絶対パスを設定 | PLM_HOME が優先される |
| 境界値 | PLM_HOME が空文字 | `PLM_HOME=""` 設定で呼び出す | HOME フォールバック |
| 境界値 | PLM_HOME が空白のみ | `PLM_HOME="   "` 設定で呼び出す | HOME フォールバック |
| 異常系 | PLM_HOME が相対パス | `PLM_HOME="relative/path"` 設定で呼び出す | `Err`（相対パス拒否メッセージ） |
| 異常系 | HOME が相対パス（PLM_HOME なし） | `HOME="relative"` かつ PLM_HOME なし | `Err`（相対パス拒否） |
| 異常系 | 両方未設定 | PLM_HOME・HOME ともに unset | `Err`（両方未設定エラー） |
| 異常系 | 両方が空白のみ | PLM_HOME・HOME ともに空白 | `Err` |
| エッジケース（回帰） | PLM_HOME 設定下での paths.rs home_dir() | PLM_HOME ≠ HOME の状態でも `home_dir()` は HOME を返す | HOME の値を返す（スコープ外不変の回帰確認） |

**テスト実行コマンド**

```bash
# パス計算テスト（並列実行可能）
cargo test plm_paths

# 環境変数制御テスト（並列競合を避けるため threads=1）
cargo test plm_root -- --test-threads=1

# 全ユニットテスト
cargo test
```

#### `src/target/core/registry_test.rs`

**役割**: `TargetRegistry::new()` が `PlmPaths` 経由のパスを使うことを検証する（修正確認）。

| カテゴリ | テストケース | ユースケース / 想定シナリオ | 期待結果 |
|----------|------------|--------------------------|---------|
| 正常系 | `new()` がデフォルトパスを構築 | HOME を TempDir に向けて `new()` を呼ぶ | `{HOME}/.plm/targets.json` が config_path になる |
| 正常系 | `PLM_HOME` 設定下の `new()` | PLM_HOME を TempDir、HOME を別パスに設定 | `{PLM_HOME}/.plm/targets.json` が config_path になる |
| 異常系 | 両方未設定で `new()` | PLM_HOME・HOME ともに unset | `Err(PlmError::TargetRegistry(...))` |

#### `src/import/registry_test.rs`

**役割**: `ImportRegistry::new()` が `PlmPaths` 経由のパスを使うことを検証する（修正確認）。

| カテゴリ | テストケース | ユースケース / 想定シナリオ | 期待結果 |
|----------|------------|--------------------------|---------|
| 正常系 | `new()` がデフォルトパスを構築 | HOME を TempDir に向けて `new()` を呼ぶ | `{HOME}/.plm/imports.json` が config_path になる |
| 正常系 | `PLM_HOME` 設定下の `new()` | PLM_HOME を TempDir、HOME を別パスに設定 | `{PLM_HOME}/.plm/imports.json` が config_path になる |
| 異常系 | 両方未設定で `new()` | PLM_HOME・HOME ともに unset | `Err(PlmError::ImportRegistry(...))` |

#### `src/marketplace/config_test.rs`

**役割**: `MarketplaceConfig::load()` が `PlmPaths` 経由のパスを使うこと、および `String` エラー型が維持されることを検証する（FR-3 / C-4）。

| カテゴリ | テストケース | ユースケース / 想定シナリオ | 期待結果 |
|----------|------------|--------------------------|---------|
| 正常系 | `PLM_HOME` 設定下の `load()` | PLM_HOME を TempDir、HOME を別パスに設定 | `{PLM_HOME}/.plm/marketplaces.json` を読む（無ければ空） |
| 正常系 | HOME のみの `load()` | PLM_HOME unset、HOME=TempDir | `{HOME}/.plm/marketplaces.json` |
| 異常系 | 両方未設定の `load()` | PLM_HOME・HOME ともに unset | `Err(String)`（`map_err` 後） |

#### `src/plugin/cache/cache_test.rs`

**役割**: `PackageCache::new()` が `PlmPaths` 経由でキャッシュパスを構築することを検証する（`PlmPaths` への統一確認）。

| カテゴリ | テストケース | ユースケース / 想定シナリオ | 期待結果 |
|----------|------------|--------------------------|---------|
| 正常系 | `PLM_HOME` 設定下の `new()` | PLM_HOME を TempDir に向けて `new()` を呼ぶ | `{PLM_HOME}/.plm/cache/plugins` が cache_dir になる |
| 異常系 | 両方未設定で `new()` | PLM_HOME・HOME ともに unset | `Err` |

#### `src/marketplace/registry_test.rs`

**役割**: `MarketplaceRegistry::new()` が `PlmPaths` 経由でキャッシュパスを構築することを検証する（`PlmPaths` への統一確認）。

| カテゴリ | テストケース | ユースケース / 想定シナリオ | 期待結果 |
|----------|------------|--------------------------|---------|
| 正常系 | `PLM_HOME` 設定下の `new()` | PLM_HOME を TempDir に向けて `new()` を呼ぶ | `{PLM_HOME}/.plm/cache/marketplaces` が cache_dir になる |
| 異常系 | 両方未設定で `new()` | PLM_HOME・HOME ともに unset | `Err` |

#### `src/env_test.rs`（横断整合性・UC-1）

**役割**: 同一 `PLM_HOME` 下で 5 経路のパスが同一 `{root}/.plm` 配下に揃うことを 1 テストで検証する（hearing-notes の統合検証要件）。

| カテゴリ | テストケース | ユースケース / 想定シナリオ | 期待結果 |
|----------|------------|--------------------------|---------|
| 正常系（整合性） | 5 パスが同一 root 配下 | `PlmPaths::with_root(root)` で 5 アクセサを取得 | すべて `{root}/.plm/...` でプレフィックス一致 |
| べき等性 | `plm_root()` 連続呼び出し | 環境固定で 2 回呼ぶ | 同一 PathBuf |

---

### 手動検証

1. `PLM_HOME=/tmp/plm-test plm target list` を実行し、`/tmp/plm-test/.plm/targets.json` が生成されることを確認（`~/.plm/targets.json` に書かれないこと）
2. `PLM_HOME=/tmp/plm-test plm marketplace add` を実行し、`/tmp/plm-test/.plm/marketplaces.json` に記録されることを確認
3. `PLM_HOME` 未設定で `plm target list` を実行し、`$HOME/.plm/targets.json` を使うことを確認（後方互換）
4. `PLM_HOME` 設定下で `plm install <plugin>` を実行し、Personal 配置先が `$HOME/.codex/` 等になることを確認（`$PLM_HOME/.codex` にならないこと）

---

### ドキュメント更新（FR-7）

#### [MODIFY] `docs/reference/config.md`

- 環境変数表の「デフォルト: `~/.plm`」を「`PLM_HOME` は `$HOME` の代替。未設定時の実効パスは `$HOME/.plm`」に修正
- 「全パスで一貫して尊重されるとは限りません」を、本修正後は一貫する旨に更新

#### [MODIFY] `docs/architecture/cache.md`

- ルート解決の記述を案 A（HOME 代替）と揃える

---

## 技術的制約・リスク（exploration-report より反映）

| リスク | 詳細 | 対応策 |
|--------|------|--------|
| 環境変数並列競合 | `std::env::set_var` を使うユニットテストは並列実行で競合する | `PlmPaths::with_root()` テストを優先; 環境変数操作テストは `--test-threads=1` |
| `MarketplaceConfig` エラー型 | `Result<_, String>` のまま維持（C-4）。`PlmPaths::new()` の `PlmError` を `.map_err(|e| e.to_string())` で変換 | 呼び出し元 `commands/manage/marketplace.rs` の変更不要 |
| `clippy` 警告 | `cargo clippy -- -D warnings` が CI で有効 | `PlmPaths` に `Default` trait が必要な場合は `new()` → `Default::default()` を検討 |
| エラーバリアント | `PlmError::Config` は存在しない。共有リゾルバは `General` を返す | 全呼び出し側で `map_err` しドメインバリアントへ（TargetRegistry / ImportRegistry / Cache） |
| 可視性 | バイナリクレートのため過剰公開不要 | `plm_root` / `PlmPaths` とも `pub(crate)` |

---

## Definition of Done

以下をすべて満たした時点で本機能の実装完了とする。

- [ ] すべてのタスク（tasks.md）が ■ になっている
- [ ] `PlmPaths::new()` が `plm_root()` を通じて `PLM_HOME`（有効時）→ `HOME` の順でパスを解決する（UC-1 / FR-1）
- [ ] `PLM_HOME` 未設定時の実効パスが従来の `$HOME/.plm` と同一（UC-2 / NFR-3・後方互換）
- [ ] `targets.json` / `marketplaces.json` / `imports.json` / `cache/plugins` / `cache/marketplaces` が すべて `{plm_root}/.plm/...` に解決される（UC-1 / FR-3）
- [ ] `std::env::var("HOME")` の直接使用が `src/marketplace/config.rs` / `src/target/core/registry.rs` / `src/import/registry.rs` から除去されている（FR-3）
- [ ] `src/target/core/paths.rs` / `src/plugin/cache/cleanup.rs` が変更されていない（C-2）
- [ ] 相対パスの `PLM_HOME` / `HOME` に対してエラーを返す（FR-5）
- [ ] `PlmPaths::with_root()` でテスト用ルートを注入できる（FR-8 相当）
- [ ] `cargo test` が全テストグリーン
- [ ] `cargo clippy -- -D warnings` が警告ゼロ
- [ ] 手動検証 4 項目が完了している
- [ ] 既存の `with_cache_dir` / `with_path` / `load_from` API が破壊されていない（FR-8）
- [ ] `docs/reference/config.md` / `docs/architecture/cache.md` の `PLM_HOME` 説明が案 A に更新されている（FR-7）
- [ ] `plm_root()` は実在する `PlmError` バリアント（`General`）のみを返す

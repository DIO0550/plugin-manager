# Task: PLM_HOME / HOME 解決の一元化

## Research & Planning

- □ `src/env_test.rs` の既存テストパターンを確認し、並列競合リスクを再評価する
- □ テストTODOリストの作成（implementation-plan.md の検証計画と照合）
  - 正常系: PLM_HOME 有効 → PLM_HOME を返す
  - 正常系: PLM_HOME 未設定 → HOME フォールバック
  - 正常系: PLM_HOME 空 / 空白のみ → HOME フォールバック
  - 正常系: 両方設定 → PLM_HOME 優先
  - 正常系: `PlmPaths` の全パスアクセサが正しいパスを返す（6 ケース）
  - 境界値: PLM_HOME 空文字 / 空白のみの各ケース
  - 異常系: PLM_HOME 相対パス → エラー
  - 異常系: HOME 相対パス（PLM_HOME なし）→ エラー
  - 異常系: 両方未設定 → エラー
  - 回帰: PLM_HOME 設定下でも `home_dir()`（paths.rs）は HOME を返す
- □ 各バグ修正対象レジストリの `new()` / `load()` メソッドの現在のエラーメッセージ文言を確認する

## Implementation (TDD サイクル)

### PlmPaths::with_root() によるパス計算（環境変数操作なし・並列安全）

- □ RED: `PlmPaths::with_root(root)` が存在しないためコンパイルエラーになることを確認
- □ RED: `plm_dir()` = `{root}/.plm` を期待するテストを `src/env_test.rs` に書く
- □ RED: `targets_json()` = `{root}/.plm/targets.json` を期待するテストを書く
- □ RED: `marketplaces_json()` = `{root}/.plm/marketplaces.json` を期待するテストを書く
- □ RED: `imports_json()` = `{root}/.plm/imports.json` を期待するテストを書く
- □ RED: `plugins_cache_dir()` = `{root}/.plm/cache/plugins` を期待するテストを書く
- □ RED: `marketplaces_cache_dir()` = `{root}/.plm/cache/marketplaces` を期待するテストを書く
- □ GREEN: `PlmPaths` 構造体と `with_root()` / 全アクセサを `src/env.rs` に追加してテストを通す
- □ REFACTOR: アクセサの実装を整理（`plm_dir()` を基点に統一されているか確認）

### plm_root() 正常系・フォールバック

- □ RED: `PLM_HOME` 有効時に `plm_root()` が PLM_HOME の PathBuf を返すテストを書く
- □ RED: `PLM_HOME` 未設定時に HOME を返すテストを書く
- □ GREEN: `plm_root()` 関数を `src/env.rs` に追加（`EnvVar::get("PLM_HOME").filter(trim 非空).or_else(HOME フォールバック)`）してテストを通す
- □ REFACTOR: `PlmPaths::new()` から `plm_root()` を呼ぶよう接続する

### plm_root() フォールバック境界値

- □ RED: `PLM_HOME=""` の場合に HOME フォールバックになるテストを書く
- □ RED: `PLM_HOME="   "` （空白のみ）の場合に HOME フォールバックになるテストを書く
- □ GREEN: `trim().is_empty()` フィルタを実装してテストを通す（既存の `EnvVar::get` の空文字フィルタを拡張）
- □ REFACTOR: 不要

### plm_root() 優先順位

- □ RED: `PLM_HOME` と `HOME` が両方設定されている場合に `PLM_HOME` が優先されるテストを書く
- □ GREEN: テストを通す（`or_else()` チェーンの順序を確認）
- □ REFACTOR: 不要（正しい順序は GREEN で確立済み）

### plm_root() 異常系・相対パス拒否

- □ RED: `PLM_HOME` に相対パスを渡すと `Err` を返すテストを書く
- □ RED: `HOME` に相対パス（`PLM_HOME` なし）を渡すと `Err` を返すテストを書く
- □ RED: `PLM_HOME`・`HOME` 両方未設定で `Err` を返すテストを書く
- □ GREEN: `is_absolute()` チェックを追加してエラーを返すよう実装し、両方未設定のエラーも実装する
- □ REFACTOR: エラーメッセージの文言を統一する（`PlmError::General` — `Config` バリアントは存在しない）

### バグ修正: MarketplaceConfig（HOME 直接参照を PlmPaths に置き換え）

- □ RED: `PLM_HOME` を設定した状態で `MarketplaceConfig::load()` を呼ぶと `PLM_HOME/.plm/marketplaces.json` パスが使われることを期待するテストを `src/marketplace/config_test.rs` に書く
- □ RED: 両方未設定で `load()` が `Err(String)` を返すテストを書く
- □ GREEN: `src/marketplace/config.rs` の `load()` を `PlmPaths::new().map_err(|e| e.to_string())?` に置き換えてテストを通す
- □ REFACTOR: 不要

### バグ修正: TargetRegistry（HOME 直接参照を PlmPaths に置き換え）

- □ RED: `PLM_HOME` を設定した状態で `TargetRegistry::new()` が `PLM_HOME/.plm/targets.json` を `config_path` にするテストを書く
- □ RED: 両方未設定で `new()` が `Err(TargetRegistry)` を返すテストを書く
- □ GREEN: `src/target/core/registry.rs` の `new()` を `PlmPaths::new().map_err(|e| PlmError::TargetRegistry(e.to_string()))?` に置き換えてテストを通す
- □ REFACTOR: 不要

### バグ修正: ImportRegistry（HOME 直接参照を PlmPaths に置き換え）

- □ RED: `PLM_HOME` を設定した状態で `ImportRegistry::new()` が `PLM_HOME/.plm/imports.json` を `config_path` にするテストを書く
- □ RED: 両方未設定で `new()` が `Err(ImportRegistry)` を返すテストを書く
- □ GREEN: `src/import/registry.rs` の `new()` を `PlmPaths::new().map_err(|e| PlmError::ImportRegistry(e.to_string()))?` に置き換えてテストを通す
- □ REFACTOR: 不要

### 統一: PackageCache（既存正解パターンを PlmPaths に移行）

- □ RED: `PLM_HOME` を設定して `PackageCache::new()` を呼ぶと `PLM_HOME/.plm/cache/plugins` が `cache_dir` になることを確認するテストを書く（既存テストの補完）
- □ RED: 両方未設定で `new()` が `Err` を返すテストを書く
- □ GREEN: `src/plugin/cache/cache.rs` の `new()` を `PlmPaths::new()?` に統一してテストを通す
- □ REFACTOR: 不要

### 統一: MarketplaceRegistry（既存正解パターンを PlmPaths に移行）

- □ RED: `PLM_HOME` を設定して `MarketplaceRegistry::new()` を呼ぶと `PLM_HOME/.plm/cache/marketplaces` が `cache_dir` になることを確認するテストを書く（既存テストの補完）
- □ RED: 両方未設定で `new()` が `Err` を返すテストを書く
- □ GREEN: `src/marketplace/registry.rs` の `new()` を `PlmPaths::new()?` に統一してテストを通す
- □ REFACTOR: 不要

### 横断整合性（UC-1）

- □ RED: `PlmPaths::with_root` で 5 アクセサが同一 `{root}/.plm` プレフィックスを持つテストを書く
- □ RED: `plm_root()` のべき等性テストを書く
- □ GREEN: パス計算実装で通す
- □ REFACTOR: 不要

### 回帰テスト: Personal 配置は PLM_HOME に引きずられない

- □ RED: `PLM_HOME` に TempDir を設定した状態で `home_dir()` (paths.rs) が `HOME` を返すことを確認するテストを書く（`src/target/core/paths_test.rs` を確認・追加）
- □ GREEN: `paths.rs` は変更不要なのでテストがそのままパスすることを確認する（スコープ外の不変確認）
- □ REFACTOR: 不要

### ドキュメント（FR-7）

- □ `docs/reference/config.md` の PLM_HOME 説明を案 A（HOME 代替）に更新する
- □ `docs/architecture/cache.md` のルート解決記述を案 A に揃える

## Verification

- □ `cargo test -- --test-threads=1` を実行して全テストがパスすることを確認する
- □ `cargo clippy -- -D warnings` を実行して警告ゼロを確認する
- □ `cargo fmt` を実行してフォーマットを整える
- □ `src/target/core/paths.rs` / `src/plugin/cache/cleanup.rs` が変更されていないことを確認する（git diff で確認）
- □ `std::env::var("HOME")` の直接使用が `src/marketplace/config.rs` / `src/target/core/registry.rs` / `src/import/registry.rs` から除去されていることを確認する（`rg 'std::env::var\("HOME"\)'` で検索）
- □ `plm_root` が `PlmError::Config` を使っていないことを確認する（実在バリアントのみ）
- □ 手動検証: `PLM_HOME=/tmp/plm-test plm target list` を実行し `/tmp/plm-test/.plm/targets.json` が生成されることを確認する
- □ 手動検証: `PLM_HOME` 未設定で `plm target list` が `$HOME/.plm/targets.json` を使うことを確認する（後方互換）
- □ 手動検証: `PLM_HOME` 設定下で Personal 配置先が `$HOME/.codex/` 等のままであることを確認する（`$PLM_HOME/.codex` にならない）

# Requirements: PLM_HOME / HOME 解決の一元化

> research（exploration-report）と plan（implementation-plan）の間で、
> 「誰が・何のために・どう使うか」と要件・制約を確定する中間ドキュメント。
> ユースケースを起点に、技術計画に進む前の前提を固める。

## ユースケース

### UC-1: PLM_HOME で状態ルートを差し替える

- **アクター**: PLM 利用者（開発者・CI）
- **状況/前提**: `PLM_HOME` を一時ディレクトリや専用ルートに設定して `plm` を実行する
- **達成したいこと**: キャッシュ・レジストリ・設定 JSON がすべて同一ルート配下に収まり、ホストの `~/.plm` と混ざらない
- **成功条件**: `targets.json` / `marketplaces.json` / `imports.json` / `cache/plugins` / `cache/marketplaces` がすべて `{PLM_HOME}/.plm/...` に解決される

### UC-2: PLM_HOME 未設定時は従来どおり HOME 配下

- **アクター**: 通常利用者
- **状況/前提**: `PLM_HOME` 未設定（または空）
- **達成したいこと**: 既存どおり `$HOME/.plm/...` に状態が置かれる
- **成功条件**: デフォルトパスが現状ドキュメントの「実効パス `$HOME/.plm`」と一致し、破壊的変更がない

### UC-3: Personal 配置は PLM_HOME に引きずられない

- **アクター**: Personal スコープで install / enable する利用者
- **状況/前提**: `PLM_HOME` と `HOME` が異なる値
- **達成したいこと**: Codex / Cursor 等の Personal 配置先は引き続き `$HOME` 配下（例: `~/.codex`）
- **成功条件**: `home_dir()` / Personal `placement_location` が `$HOME` を参照し、`$PLM_HOME/.codex` 等に書かない

### UC-4: 実装者・テストが単一のパス API を使う

- **アクター**: メンテナ / 将来の #333 実装
- **状況/前提**: 新しい PLM 状態ファイルやキャッシュパスを追加する
- **達成したいこと**: `PlmPaths`（または `plm_dir()`）経由だけでパスを組み立て、`HOME` / `PLM_HOME` の直読みを増やさない
- **成功条件**: デフォルト構築経路に `std::env::var("HOME")` の散在がなく、テストは `with_root` / 既存 `with_path` で差し替え可能

## 要件・制約

### 機能要件

- FR-1: `plm_root()` は `PLM_HOME`（有効時）→ `HOME` の順で親ディレクトリを返す（案 A: HOME 代替）
- FR-2: `plm_dir()` は `{plm_root}/.plm` を返す
- FR-3: 次のデフォルト構築がすべて `plm_root` / `PlmPaths` 経由になること
  - `PackageCache::new` → `{plm_dir}/cache/plugins`
  - `MarketplaceRegistry::new` → `{plm_dir}/cache/marketplaces`
  - `MarketplaceConfig::load` → `{plm_dir}/marketplaces.json`
  - `TargetRegistry::new` → `{plm_dir}/targets.json`
  - `ImportRegistry::new` → `{plm_dir}/imports.json`
- FR-4: 未設定・空・空白のみの `PLM_HOME` は無効扱いとし `HOME` へフォールバック（現行 `EnvVar::get` 互換を拡張して trim）
- FR-5: 相対パスの `PLM_HOME` / `HOME` は拒否してエラー
- FR-6: 両方無効なら明確なエラー（既存キャッシュと同系統のメッセージ）
- FR-7: `docs/reference/config.md` / `docs/architecture/cache.md` の `PLM_HOME` 説明を案 A に合わせて更新する
- FR-8: 既存の `with_cache_dir` / `with_path` / `load_from` は維持する

### 非機能要件

- NFR-1: 新規クレート依存を追加しない
- NFR-2: TDD（Red → Green → Refactor）で進める
- NFR-3: `PLM_HOME` 未設定時の実効パスは現状と同一（後方互換）
- NFR-4: 環境変数操作を伴うユニットテストは並列競合に注意（`with_root` 優先、または必要なら threads=1）

### 制約・設計方針

- C-1: `PLM_HOME` セマンティクスは **案 A（HOME 代替）**。CARGO_HOME 型（案 B）は採用しない
- C-2: `src/target/core/paths.rs` と `src/plugin/cache/cleanup.rs` は **変更しない**（ユーザー HOME）
- C-3: `plm_root` / `PlmPaths` の配置は `src/env.rs`（または同 Feature の隣接モジュール）。`path_ext` には置かない
- C-4: `MarketplaceConfig` の戻り値型 `Result<_, String>` は本 Issue では変更しない
- C-5: Feature ベースのモジュール構成・`*_test.rs` 分離を踏襲
- C-6: #333（config.toml）は本 Issue のスコープ外。ただし `PlmPaths` に将来の `config_toml()` を載せられる形にしてよい

## 未解決の確認事項

なし

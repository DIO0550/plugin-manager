# Task: clippy.toml 導入と cognitive-complexity 閾値によるコード品質改善

## Research & Planning

- [■] `clippy.toml` を作成して `cargo clippy` を実行し、正確な違反リストを取得する
- [■] 既存 `allow(clippy::...)` を `rg 'allow\(clippy::' src/` で棚卸しし、抑制中の箇所も必須対応対象として追加する
- [■] 違反をカテゴリ別に分類する（cognitive-complexity / too-many-arguments / too-many-lines / type-complexity）
- [■] 各違反関数のリファクタリング方針を確定する（スキャン結果 + 棚卸し結果に基づく微調整）

## Implementation

### Phase 0: clippy.toml 作成

- [■] [NEW] `clippy.toml` をプロジェクトルートに作成（4つの閾値設定）
- [■] `cargo clippy` を実行し、正確な違反リストを記録する

### Phase 1: cognitive-complexity 違反の修正

> スキャン結果: cognitive-complexity 違反なし。Phase 1 はスキップ。

#### src/error.rs

- [■] 違反なし — スキップ

#### src/hooks/converter.rs

- [■] 違反なし — スキップ

#### src/component/deployment.rs

- [■] 違反なし — スキップ

#### src/plugin/update.rs

- [■] 違反なし — スキップ

#### src/install.rs

- [■] 違反なし — スキップ

#### src/commands/import.rs

- [■] 違反なし — スキップ

#### src/commands/marketplace.rs

- [■] 違反なし — スキップ

#### src/commands/list.rs

- [■] 違反なし — スキップ

### Phase 2: too-many-arguments 違反の修正

> `too-many-arguments-threshold = 5` は「> 5 引数」で違反。5引数の関数は違反しない。

#### src/hooks/converter.rs

- [■] [MODIFY] `ConversionCollector` 構造体を定義（warnings + wrapper_scripts を束ねる）
- [■] [MODIFY] `convert_prompt_agent_hook`（6引数）の引数を `ConversionCollector` に変更する
  - `cargo test` で converter 関連テストが通ることを確認
- [■] `convert_hook_definition`（5引数）→ 違反なし、スキップ
- [■] `convert_command_hook`（5引数）→ 違反なし、スキップ

#### src/plugin/update.rs

- [■] [MODIFY] `UpdateCtx` 構造体を定義する
- [■] [MODIFY] `do_update`（8引数）の引数を `UpdateCtx` に変更し、既存の `allow(clippy::too_many_arguments)` を削除する
  - `cargo test` で update 関連テストが通ることを確認

#### src/tui/manager/screens/marketplaces/view.rs

- [■] [MODIFY] `ViewCtx` / `FilterCtx` / `BrowseData` 構造体を定義する
- [■] [MODIFY] `view_market_list`（7引数）の引数を `ViewCtx` に変更し、既存の `allow` を削除する
- [■] [MODIFY] `view_market_detail`（7引数）の引数を `ViewCtx` に変更し、既存の `allow` を削除する
- [■] [MODIFY] `view_plugin_browse`（8引数）の引数を `BrowseData` + `FilterCtx` に変更し、既存の `allow` を削除する
- [■] [MODIFY] `view_plugin_list`（6引数）の引数を `FilterCtx` に変更する

#### src/tui/manager/screens/installed/view.rs

- [■] [MODIFY] `ViewCtx` 構造体を定義する
- [■] [MODIFY] `view_plugin_list`（7引数）の引数を `ViewCtx` に変更し、既存の `allow` を削除する
- [■] [MODIFY] `view_plugin_detail`（6引数）の引数を `ViewCtx` に変更する
- [■] [MODIFY] `view_component_types`（6引数）の引数を `ViewCtx` に変更する
- [■] [MODIFY] `view_component_list`（7引数）の引数を `ViewCtx` に変更する

#### src/sync.rs

- [■] [MODIFY] `SyncPlan` 構造体を定義（3つの Vec を束ねる）
- [■] [MODIFY] `execute_sync`（6引数）の引数を `SyncPlan` に変更する

#### src/tui/manager/screens/marketplaces/actions.rs

- [■] [MODIFY] `InstallCtx` 構造体を定義する
- [■] [MODIFY] `install_single_plugin`（7引数）の引数を `InstallCtx` に変更する

#### src/tui/manager/screens/marketplaces/update.rs

- [■] [MODIFY] `AddEntry` 構造体を定義する
- [■] [MODIFY] `execute_add_with`（6引数）の引数を `AddEntry` に変更する
  - テストコード（update_test.rs）も合わせて更新

#### src/application/plugin_operations.rs（候補）

- [■] `disable_plugin` / `enable_plugin`（5引数）→ 違反なし、スキップ

### Phase 3: too-many-lines 残存違反の修正

- [■] スキャン結果: too-many-lines 違反なし。Phase 3 はスキップ。

### Phase 4: type-complexity 違反の修正

- [■] `src/application/plugin_intent.rs` に `CreateOperationResult` 型エイリアスを導入し、違反を解消
- [■] `src/host.rs` に `BoxFuture` 型エイリアスを導入し、既存の `allow(clippy::type_complexity)` を削除

### Phase 5: 既存 allow アノテーションの最終クリーンアップ

- [■] `src/plugin/update.rs` の `allow(clippy::too_many_arguments)` が削除されていることを確認する
- [■] `src/tui/manager/screens/marketplaces/view.rs` の `allow(clippy::too_many_arguments)` (3箇所) が削除されていることを確認する
- [■] `src/tui/manager/screens/installed/view.rs` の `allow(clippy::too_many_arguments)` が削除されていることを確認する
- [■] `src/host.rs` の `allow(clippy::type_complexity)` が削除されていることを確認する
- [■] プロダクションコード全体で新たな `allow` 抑制が追加されていないことを `grep` で確認する

## Verification

- [■] `cargo clippy -- -D warnings` で warning/error が0件であることを確認する
- [■] `cargo test` で全テストが通ることを確認する（23件の既存失敗は環境要因で変更と無関係）
- [■] `cargo fmt --check` で差分がないことを確認する
- [■] `cargo check` がエラーなしで通ることを確認する
- [■] `clippy.toml` に4つの閾値が正しく設定されていることを確認する
- [■] プロダクションコードに `allow(clippy::...)` 抑制が残っていないことを確認する
- [ ] CI パイプラインが green であることを確認する

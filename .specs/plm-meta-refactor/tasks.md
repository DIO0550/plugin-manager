# installedAt 分離タスク

## Research & Planning

- [x] 現状の `installedAt` 実装箇所を特定
- [x] 代替案（案A/B/C）のメリット・デメリットを比較
- [x] Codexによる設計レビューを実施
- [x] 実装計画（implementation-plan.md）を作成
- [x] Codexによる実装計画レビューを実施
- [x] 指摘事項を反映して計画を改善

## Implementation

### Phase 1: 新規モジュール作成

- [x] `src/plugin/meta.rs` を新規作成
  - [x] `PluginMeta` 構造体を定義（`installed_at: Option<String>`）
  - [x] `write_meta()` 関数を実装（アトミック書き込み: tmp→rename）
  - [x] `load_meta()` 関数を実装（破損時は警告ログ + None）
  - [x] `resolve_installed_at()` 関数を実装（フォールバック付き）
  - [x] 単体テストを追加

### Phase 2: 書き込み処理の移行

- [x] `src/plugin.rs` に `pub mod meta;` を追加
- [x] `src/plugin/cache.rs` を修正
  - [x] `write_installed_at()` を削除
  - [x] `store_from_archive()` 内で `meta::write_meta()` を呼び出す
  - [x] 既存テストを `.plm-meta.json` 検証に変更

### Phase 3: 読み込み処理の移行

- [x] `src/application/plugin_info.rs` を修正
  - [x] `meta::resolve_installed_at()` を呼び出して `installed_at` を取得
  - [x] `manifest.installed_at` への直接参照を削除

### Phase 4: テスト修正

- [x] `cache.rs` のテストを `.plm-meta.json` 検証に変更
- [x] フォールバック動作のテストを追加
- [x] 破損ファイル処理のテストを追加
- [x] 優先順位テストを追加（両方に値がある場合）

## Verification

- [x] `cargo test` が全てパス（350テスト）
- [ ] `cargo clippy` で警告なし（環境未対応のためスキップ）
- [ ] `cargo fmt --check` でフォーマット確認（環境未対応のためスキップ）
- [ ] 手動テスト
  - [ ] `plm install <plugin>` 後に `.plm-meta.json` が作成される
  - [ ] `plm info <plugin>` で `installedAt` が表示される
  - [ ] 既存プラグイン（`.plm-meta.json` なし）でもフォールバックで `installedAt` が表示される
  - [ ] `.plm-meta.json` を削除後、`plugin.json` の値が使われることを確認

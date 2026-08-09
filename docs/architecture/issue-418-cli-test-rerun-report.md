# Issue #418 CLI テスト再実行レポート

## 結論

以前の **30 件の CLI（assert_cmd / `cargo_bin("plm")`）失敗は環境要因**（`/workspace/target/debug/plm` 未生成）であり、OpenCode 変更による回帰ではない。

## 実施内容

1. `cargo build` を実行し、`target/debug/plm` を生成した。
2. バイナリ依存の CLI テスト 30 件のみを再実行した。
3. OpenCode 関連ユニットテストも再確認した。

## 結果

| 対象 | 結果 |
|------|------|
| CLI / assert_cmd（バイナリ依存 30 件） | **30 passed / 0 failed** |
| OpenCode ユニットテスト（`cargo test opencode`） | **19 passed / 0 failed** |

## 対象テスト内訳（30 件）

これらは trycmd ではなく `assert_cmd` 経由で `Command::cargo_bin("plm")` を使うテスト。

- `cli::tests::*`（help / フラグ衝突など）: 20
- `commands::lifecycle::enable::tests::*`: 2
- `commands::lifecycle::disable::tests::*`: 2
- `commands::deploy::sync::tests::test_sync_*`（`plm()` ヘルパ利用）: 6

## 判定

- 失敗原因: `plm` デバッグバイナリ未ビルド（環境）。
- OpenCode 変更由来の CLI 回帰: **なし**。

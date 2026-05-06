# Issue #265 実装状況レビュー

## 結論

Issue #265「画面ごとの段階的移行」は、本実装で着手済みになった。

Issue #265 の本文の目的は「1 画面ずつ共通 state を埋め込み、PR を分けて移行する」ことになっている。依存先の Issue #264「共通 state 型の設計」は、ローカルにも `docs/issue-264-common-state-design-2026-05-05.md` が存在するため、設計内容を参照できる状態だった。

今回、#264 の方針に沿って `SelectionState<K>` を追加し、`Installed::PluginList` と `Marketplaces::MarketList` の一覧選択 state を共通型へ移行した。

## 確認結果

- `src/tui/manager/core/selection_state.rs` に `SelectionState<K>` を追加した。
- `src/tui/manager/core.rs` から `SelectionState` を re-export した。
- `src/tui/manager/screens/installed/model.rs` の `PluginList` は、`selected_id` と `ListState` の直接保持から `selection: SelectionState<PluginId>` へ移行した。
- `src/tui/manager/screens/marketplaces/model.rs` の `MarketList` は、`selected_id` と `ListState` の直接保持から `selection: SelectionState<String>` へ移行した。
- `from_cache()` / `to_cache()` の stale ID 検証と semantic cache は各画面側に残した。
- `Discover` は実データリストと stale ID の検証先が未定義であるため、今回の移行対象外にした。

## #264 設計との差分

`docs/issue-264-common-state-design-2026-05-05.md` では、共通化候補を `ListState + selected key` の小型 helper に限定し、将来の `SelectionState<K>` を画面 `Model` へ composition で埋め込む方針が書かれている。

同ドキュメントの移行順序に対する対応状況は次のとおり。

1. `src/tui/manager/core/selection_state.rs` を追加した。
2. `Installed::Model::PluginList` の `selected_id` と `state` を `SelectionState<PluginId>` に置き換えた。
3. `Marketplaces::Model::MarketList` の `selected_id` と `state` を `SelectionState<String>` に置き換えた。
4. `Discover` は後続判断に残した。
5. 既存の `from_cache()` / `to_cache()` / update のテストは新しい `selection` 構造へ追従し、挙動を維持した。

## 判定

「段階的移行の初回実装済み」が妥当。

`Installed` と `Marketplaces` の主要な重複箇所は共通 state 型へ移行済み。`Discover` は現時点で UI 未実装寄りのため、実データリストの責務が固まった時点で追加移行を検討する。

## 検証

- `cargo check`
- `cargo test`（1600 tests passed）

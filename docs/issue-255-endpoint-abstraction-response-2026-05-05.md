# Issue #255 対応結果: sync/endpoint の共通抽象化検討

- Issue: <https://github.com/DIO0550/plugin-manager/issues/255>
- 対象: `SyncSource` / `SyncDestination` / `Endpoint`
- 確認日: 2026-05-05
- 対応方針: Rust ファイルは変更せず、現状判定と検証結果を記録する

## 結論

Issue #255 は将来トリガー付きの追跡タスクであり、現時点では追加実装に着手しない。

理由は次の通り。

- `src/sync/endpoint.rs` 直下には既に `Endpoint` enum が存在する。
- `name`, `command_format`, `placed_components`, `path_for` の共通処理は `Endpoint` から `TargetBinding` 経由で扱える。
- `SyncSource` / `SyncDestination` の実装本体は `src/sync/endpoint/binding.rs` に集約済みで、重複しているのは newtype ファサードの薄い委譲にとどまる。
- production コードで `Source` / `Destination` の双方を `match` でまとめて扱う箇所は確認できない。
- 既存レビュー `docs/sync-endpoint-duplication-2026-05-04.md` の「トリガー未発火、現状維持推奨」という判定から変更なし。

## 完了条件との照合

| 完了条件 | 現状 | 判定 |
|---|---|---|
| `enum Endpoint`（または同等の抽象）が `src/sync/endpoint.rs` 直下に定義されている | `pub(super) enum Endpoint { Source(SyncSource), Destination(SyncDestination) }` が定義済み | 満たしている |
| 共通ロジックが enum 側に寄せられ、`source.rs` / `destination.rs` の重複が解消されている | 実装本体は `TargetBinding` に集約済み。`Endpoint` は crate 内部の dispatch 集約点として存在する | 実装重複は解消済み |
| 既存テストが全て通る | `cargo test` が成功 | 満たしている |

## トリガー条件の再判定

| トリガー条件 | 観測結果 | 判定 |
|---|---|---|
| 双方の実装に同等のメソッドが 3 つ以上重複 | public facade の委譲メソッドは重複しているが、実装本体は `TargetBinding` に集約済み | 未発火 |
| 双方をまとめて扱うコードが複数箇所に出現 | production コードで match-over-both は確認できない | 未発火 |

## 検証

```text
cargo test
```

結果:

```text
test result: ok. 1588 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

補足:

- `src/tui/manager/screens/marketplaces/update_test.rs` の未使用 import 警告が 2 件出ている。
- `src/cli_test.rs` などで `assert_cmd::cargo_bin` 非推奨警告が出ている。
- いずれも既存警告であり、Issue #255 の対応可否には影響しない。

## 推奨アクション

Issue #255 は「今すぐ実装」ではなく「将来トリガー条件の監視」として扱うのが妥当。

今後、production コードで `SyncSource` / `SyncDestination` をまとめて扱う `match` や共通 dispatch が複数箇所に増えた場合に、既存の `Endpoint` enum を本格利用する。

# Issue #262 Endpoint Review

## 結論

- `Endpoint` は既存の `pub(super)` enum を維持し、新規 production enum / wrapper は追加しない。
- `name` / `command_format` / `placed_components` / `path_for` は `binding()` 経由の dispatch に集約済み。
- `SyncDestination::supports` は Destination 固有の capability 判定であり、`Endpoint` には移さない。
- `sync.rs` の `sync` / `sync_with_fs` / `execute_sync` / `execute_create` / `execute_update` は Source が入力、Destination が出力として役割差分を持つため、現段階では `Endpoint` 化しない。

## 確認内容

- `Endpoint` の visibility は `pub(super)` のまま。
- public API `sync(&SyncSource, &SyncDestination, &SyncOptions)` は維持。
- `Endpoint::supports` は存在しない。
- parity test は `Source` / `Destination` の `name` / `command_format` / `placed_components` / `path_for` と invalid component name 経路を対象に維持・追加。

## 将来の移行条件

両 endpoint を完全に同じ操作として扱える private helper が複数出てきた場合のみ、限定的に `Endpoint` 利用を検討する。`supports` のように Destination の意味を持つ操作や、copy / convert のように Source と Destination の役割が分かれる処理は移行対象にしない。

# Issue #284 Code Review

## 結果

問題なし。

## 備考

`codex exec --dangerously-bypass-approvals-and-sandbox` によるレビュー実行は、権限審査で外部実行と広い unsandboxed access のリスクとして拒否された。
迂回は行わず、`.plugin-workspace/.specs/025-issue-284/code-review/context-001.md` と作業差分を同じ観点でローカル確認した。

## 確認内容

- `Endpoint<'a>` は参照 enum に変更され、`SyncSource` / `SyncDestination` の所有権移動を増やしていない。
- `sync_with_fs` の component scan は `collect_components(Endpoint<'_>, ...)` 経由に限定されている。
- `sync(&SyncSource, &SyncDestination, &SyncOptions)` の public API は維持されている。
- `Endpoint` は `pub(super)` のままで、`Endpoint::supports` は追加されていない。
- `SyncDestination::supports` は destination 固有処理として残っている。
- scan 段階のエラーは `SyncFailure` に包まず `Result::Err` として伝播するテストで保護されている。

## 検証

- `cargo fmt --check`
- `cargo clippy`
- `cargo test endpoint`
- `cargo test sync`
- `cargo test commands::deploy::sync`
- `cargo test`
- `rg -n "Endpoint::supports|pub fn supports" src/sync src/commands/deploy/sync.rs`
- `rg -n "pub fn sync\\(|sync_with_fs\\(" src/sync.rs`
- `rg -n "Endpoint::Source|Endpoint::Destination" src/sync.rs src/sync/endpoint/endpoint_test.rs`

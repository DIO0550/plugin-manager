# Issue #254 重複・命名レビュー追従結果

## 結論

Issue #254 配下の5領域について、計画上の将来実装候補を分類し、発火済みの箇所だけ限定実装した。
`sync/endpoint` は既存の `collect_components(Endpoint<'_>, ...)` 限定導入を完了確認し、追加 production helper は未発火として保留した。commands Args は共通引数部品を導入した。TUI selection は Discover の実データ責務が未確定のため追加移行を保留し、公開境界 `Model` と Result 系命名は互換 alias を追加して段階移行できる状態にした。

公開 API である `sync()` の signature と `Endpoint` の `pub(super)` 可視性は維持している。

## 分類サマリ

| 領域 | 対象Issue | 判定 | 理由 |
| --- | --- | --- | --- |
| sync/endpoint | #255, #260-#262, #283-#285 | 完了確認 | `collect_components(Endpoint<'_>, ...)` の限定導入を確認し、追加 production helper は未発火として保留した |
| TUI state | #256, #263-#265 | 保留 | Installed / Marketplaces の完了を確認し、Discover は実データ責務確定まで追加移行しない |
| commands Args | #257, #266-#268 | 実装 | `src/commands/args.rs` を追加し、list / install / import / sync に `#[command(flatten)]` を導入した |
| Model rename | #258, #269-#270 | 実装 | `InstalledModel` / `MarketplacesModel` alias を公開し、`Screen` enum の外部境界で利用した（**Update 2026-05-09 (#258 / 028)**: alias は `InstalledScreenModel` / `MarketplacesScreenModel` に置換、Discover / Errors も `*ScreenModel` に統一）|
| Result naming | #259, #271-#273 | 実装 | `OperationOutcome` / `UpdateOutcome` alias を追加し、呼び出し側の型注釈へ段階導入した |

## sync/endpoint

### 現状確認

- `src/sync/endpoint/binding.rs` の `TargetBinding` が source/destination の共通実装本体を担う。
- `Endpoint<'a>` は `src/sync/endpoint.rs` で `pub(super)` の参照 enum として定義されている。
- `src/sync.rs` の public entrypoint は `sync(source: &SyncSource, dest: &SyncDestination, options: &SyncOptions)` のまま維持されている。
- production scan は `sync_with_fs()` 内の `collect_components(Endpoint::Source(source), ...)` と `collect_components(Endpoint::Destination(dest), ...)` に限定されている。
- `SyncDestination::supports()` は destination 固有 API のまま残っており、`Endpoint` へ押し込まれていない。

### 判定

`collect_components(Endpoint<'_>)` の限定利用は許容範囲であり、#284 は完了確認扱いにできる。
追加 production helper は同一責務の候補が2件以上になるまで未発火とし、`path_for` の1箇所 pass-through helper は導入しない。

`Endpoint` は `pub(super)` のままで、外部 caller は引き続き `SyncSource` / `SyncDestination` を使う。

## TUI state

### 現状確認

- `SelectionState<K>` は `selected_id: Option<K>` と `ListState` だけを保持する。
- Installed の plugin list と Marketplaces の market list では `SelectionState<K>` が導入済み。
- Discover は `selected_id: Option<PluginId>` と `ListState` を個別に持つが、現状の update/view は実データ一覧をまだ扱っていない。
- Errors は `CacheState` を持たず、`DataStore.last_error` 表示に閉じている。

### 判定

#263 / #264 / #265 のうち、Installed / Marketplaces の2画面移行は完了確認できる。
Discover は現状の update/view が実データ一覧をまだ扱っていないため、`SelectionState<PluginId>` への追加移行は保留する。restore / 空リスト / stale ID fallback を検証できる実データ責務が確定してから移行する。

selection 以外の画面固有 state は共通化対象外であり、Errors に空の `CacheState` は追加していない。

## commands Args

### 棚卸し

| 引数 | 出現箇所 | 型 | 意味差 | 判定 |
| --- | --- | --- | --- | --- |
| `--target` | `list`, `enable`, `disable`, `update` | `Option<TargetKind>` | list は enabled 状態フィルタ、lifecycle は操作対象 | 共通化しない |
| `--target` | `install`, `import` | `Option<Vec<TargetKind>>` | deploy 対象の複数指定 | 単数 target と同一視しない |
| `--scope` | `install`, `import`, `sync` | `Option<Scope>` | deploy/sync の対象 scope | 候補だが3箇所の意味・help・default の固定テストが先 |
| `--json` / `--simple` / `--outdated` | `list` | `bool` | list 出力 family 専用。conflict 条件を共有する | list 内に閉じる |

### 判定

`src/commands/args.rs` に `ListOutputArgs`, `SingleTargetArgs`, `MultiTargetArgs`, `InteractiveScopeArgs`, `SyncScopeArgs` を追加した。
`list` は `SingleTargetArgs` と `ListOutputArgs`、`install` / `import` は `MultiTargetArgs` と `InteractiveScopeArgs`、`sync` は `SyncScopeArgs` を flatten している。

`InteractiveScopeArgs` は未指定時の TUI 選択を help に残し、`SyncScopeArgs` は未指定時に personal / project の両方を対象にする意味を help に残す。

### 将来トリガー

flatten 後の参照は `args.output.json`, `args.target.target`, `args.scope.scope` へ更新した。`cargo run -- <command> --help` で help 表示も確認した。

## Model rename

### 棚卸し

- `src/tui/manager/screens/installed.rs` と `marketplaces.rs` は `pub use model::{..., Model, ...};` で画面ローカル `Model` を再exportしている。
- `src/tui/manager/core/app.rs` の `Screen` enum は `installed::Model`, `discover::Model`, `marketplaces::Model`, `errors::Model` を variant payload として扱う。
- 画面内部では `model.rs` の `Model` がローカル文脈で使われており、具体名へ一括変更する必要はない。

### 判定

`installed::InstalledModel` / `marketplaces::MarketplacesModel` alias を追加し、`Screen` enum の payload 型で利用した。画面内部の `Model` 名は維持している。

> **Update 2026-05-09 (#258 / 028)**: ここで導入した `InstalledModel` / `MarketplacesModel` alias は、
> Issue #258 命名規則 `<Screen>ScreenModel` への統一に伴い、028 タスクで
> `InstalledScreenModel` / `MarketplacesScreenModel` に置換された。
> 同時に Discover / Errors も `DiscoverScreenModel` / `ErrorsScreenModel` に統一。
> 詳細は `docs/architecture/naming-conventions.md` §4 参照。

## Result naming

### 棚卸し

| 型 | 所属 | 公開境界 | 判定 |
| --- | --- | --- | --- |
| `SyncResult` | `src/sync/model/result.rs` | `sync` feature から re-export | sync の戻り値として具体的で維持 |
| `ConvertResult` | `src/hooks/converter/converter.rs` | hooks converter 内 | converter 処理結果として維持 |
| `ConversionResult` → `ConversionOutcome` (#273) | `src/component/convert.rs` | component deployment から参照 | command/agent conversion の既存値として維持。Issue #273 で `ConversionOutcome` にリネーム済み |
| `OperationResult` | `src/target/effect.rs` | `target` / `application` から re-export | target 操作結果として維持。将来は `OperationOutcome` 候補 |
| `UpdateResult` | `src/plugin/lifecycle/update.rs` | plugin lifecycle から re-export | lifecycle update の戻り値として維持 |
| `DeploymentOutput` | `src/component/deployment/output.rs` | component feature から re-export | `Output` 命名例として維持 |

### 判定

`OperationOutcome = OperationResult` と `UpdateOutcome = UpdateResult` の互換 alias を追加し、呼び出し側の型注釈へ一部導入した。既存 `Result` 系型は互換性のため維持している。

**Update 2026-05-10 (#271 / #273)**:
- #271 で `OperationOutcome` / `UpdateOutcome` の互換 alias を削除し、concrete struct に統一済み（`OperationResult` / `UpdateResult` は完全消滅）。
- #273 で残存していたドメイン成果値の `*Result` 型（`AddResult` / `RemoveResult` / `PlaceResult` / `ConversionResult` / `AgentConversionResult` / `VersionQueryResult` / `ExpandResult` / `MultiSelectResult` / `SingleSelectResult` / `PluginInstallResult` / `ActionResult` / `LoadMarketplacesResult` および `MarketplacesScreenModel::InstallResult` バリアント）を `*Outcome` に **clean rename**（互換 alias なし）。
- 上記表中の旧名（`SyncResult` / `ConvertResult` / `OperationResult` / `UpdateResult` / `ConversionResult`）は #271 / #273 時点で全件 `*Outcome` 系へ置換済み。`Result` 接尾辞は `std::result::Result<T, E>` エイリアス（例: `CreateOperationResult`）など本来の用途のみに残す方針へ移行した。詳細は `docs/architecture/naming-conventions.md` §1.2 / §1.3 を参照。

## 親Issue同期メモ

| 親Issue | 同期する状態 |
| --- | --- |
| #255 sync/endpoint | `TargetBinding` 集約、`collect_components` の限定導入を完了。追加 helper は未発火 |
| #256 TUI state | `SelectionState<K>` は Installed / Marketplaces まで完了。Discover は実データ責務確定まで追加移行を保留。Errors は対象外 |
| #257 commands Args | 共通 Args 部品を追加し、該当commandへ flatten 導入 |
| #258 Model rename | `InstalledModel` / `MarketplacesModel` alias を公開境界へ導入（**Update 2026-05-09 (#258 / 028)**: `InstalledScreenModel` / `MarketplacesScreenModel` へ置換、Discover / Errors も `*ScreenModel` に統一）|
| #259 Result naming | `OperationOutcome` / `UpdateOutcome` alias を追加し、段階移行を開始 |

## 検証メモ

- `Endpoint` の public 化と `sync()` signature 変更は行っていない。
- `cargo fmt --check`, `cargo clippy`, `cargo test` は成功。
- `cargo run -- list --help`, `cargo run -- install --help`, `cargo run -- import --help`, `cargo run -- sync --help` で help 表示を確認した。
- `list --json --simple`, `list --outdated --simple`, `list --outdated --json` の conflict / 許容挙動をテストで固定した。

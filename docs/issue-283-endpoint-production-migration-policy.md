# Issue #283 Endpoint production移行方針

Endpoint production 移行判断では、最初にこの文書を読む。

## 目的

Endpoint の production 移行を始める前に、外部 API、内部 API 化範囲、可視性、責務境界、段階移行順序を明確化する。

このメモは Issue #283 時点の Endpoint production 移行判断で優先される方針である。過去 docs は履歴情報として参照し、Endpoint production 移行判断に関して本メモと矛盾する場合に限り、本メモを最新方針として扱う。過去メモに `pub(crate)` と記載されている箇所は履歴情報として扱い、現コードの `pub(super)` を正とする。

このメモでいう production 内部は、実 CLI 経路から同期実行に到達する production code を基準とし、現時点では `src/commands/deploy/sync.rs`、同期実行本体の `src/sync.rs`、および `src/sync/**` を指す。将来 sync 関連コードが別 module に移動した場合は、実 CLI 経路から到達する同期実行 code を production 内部として定義し直す。

## 結論

- `sync(&SyncSource, &SyncDestination, &SyncOptions)` は維持する。
- `SyncSource` / `SyncDestination` の public newtype API は維持する。
- `Endpoint` は `pub(super)` のまま維持する。
- `Endpoint` は sync モジュール内部の dispatch 重複解消用であり、外部 caller 用 API ではない。
- `SyncDestination::supports` は destination 固有メソッドとして残し、`Endpoint::supports` は追加しない。
- production 内部の Endpoint 化は、同一責務を持つ独立した production code 上の分岐箇所が2つ以上、または source/destination 同一操作の private helper 候補が独立して2件以上出た場合だけ検討する。
- ISSUE-01-E は private helper 限定の Endpoint 活用可否、ISSUE-01-F は必要時の public API 長期設計として切り分ける。

## 現状整理

- `TargetBinding` に source/destination 共通実装は集約済みである。
- Issue #283 調査時点では production の `Endpoint::...` 呼び出しは 0 件である。
- 後続 Issue 着手時には、`TargetBinding` 集約状況、production の `Endpoint::...` 呼び出し数、`Endpoint` visibility、`supports` 配置を再確認する。
- `src/sync.rs` の private helper は source が入力、destination が出力という役割差分を明示している。
- `execute_create` / `execute_update` のような入力 path、出力 path、format 変換の非対称処理は Endpoint 化対象外とする。

## 外部 API 方針

`sync(&SyncSource, &SyncDestination, &SyncOptions)` は維持する。`Endpoint` 公開や `sync` signature 変更は ISSUE-01-F 以降で、public API 変更が必要な具体要件が出た場合だけ設計する。

ISSUE-01-E では public API、CLI 出力、エラー表現を変更しない。production 内部で Endpoint を使う場合も private helper に閉じ、外部 caller は引き続き `SyncSource` / `SyncDestination` を使う。

## Endpoint の責務

### 責務

- sync モジュール内部の共通 dispatch。
- `TargetBinding` へ委譲できる source/destination 共通操作の薄い窓口。
- public API を変えずに private helper 内で分岐重複を減らすこと。

### 非責務

- 外部 caller 向け API。
- source/destination の役割差分を隠蔽すること。
- destination 固有 capability 判定を source variant に拡張すること。
- `execute_create` / `execute_update` のような source/destination 非対称処理を同一操作に見せること。

## supports の配置

`SyncDestination::supports` は destination 固有メソッドとして残す。`Endpoint::supports` は追加しない。

Source variant で意味を持たない capability 判定を `Endpoint` に載せると、`false`、`None`、error などの不要な意味論が発生する。現状どおり destination でのみ呼び出せる形を保つことで、型レベルで呼び出し可能範囲を狭くする。

## Endpoint 化の判定基準

Endpoint 化してよいのは、次の条件をすべて満たす場合だけである。

- Source/Destination のどちらにも完全に同じ操作をする。
- 入力、戻り値、エラーの意味、必要な権限、副作用、source/destination の役割差分が一致する。
- `TargetBinding` への委譲だけで表現できる。
- public API を変えず private helper に閉じる。
- 同一責務を持つ独立した production code 上の分岐箇所が2つ以上ある、または source/destination 同一操作の private helper 候補が独立して2件以上ある。

分岐箇所は match 式、if-let 分岐、variant ごとの同等処理ブロックを1単位として数える。同一 `match` 内の複数 arm は1箇所と数え、同一 helper から展開される複数分岐も1箇所として扱う。

独立した分岐箇所とは、別関数または別責務の処理単位に存在し、片方を削除してももう片方が残る重複を指す。単に同じ関数内で source と destination を順番に使っているだけの箇所は、Endpoint 化の発火条件に含めない。

## Endpoint 化しない条件

- Source は入力、Destination は出力として役割が違う。
- `supports` のように destination 固有の意味を持つ。
- Endpoint 公開や `sync` signature 変更が必要になる。
- clone 生成や trait object 所有権の変更が必要になる。
- エラー型、CLI 表示、依存関係変更を ISSUE-01-E の範囲に混ぜる必要がある。

## ISSUE-01-E の実装方針

ISSUE-01-E では production 内部の private helper 限定で Endpoint 活用可否を再評価する。対象は、source/destination のどちらにも同じ操作をする共通 dispatch だけに限定する。

条件を満たさない場合は、現状維持を明示的な結論にする。`execute_create` / `execute_update` のような非対称処理は Endpoint 化対象外とし、source/destination の役割差分を表す現在の構造を維持する。

実装する場合も、`TargetBinding` に共通実装を置き、`Endpoint` は `binding()` 経由の薄い委譲にとどめる。

## ISSUE-01-F の実装方針

ISSUE-01-F では、public API 変更が必要になった場合だけ別途設計する。検討対象は `Endpoint` 公開、互換 layer、CLI、docs、統合テスト、後方互換方針である。

public API 変更の具体要件がない場合、`Endpoint` は `pub(super)` のまま維持する。

## 判断材料

| 主要判断 | 参照元 |
| --- | --- |
| 外部 API 維持 | `.plugin-workspace/.specs/024-issue-283/hearing-notes.md`、`.plugin-workspace/.specs/024-issue-283/exploration-report.md`、`src/sync.rs` |
| `Endpoint` visibility 維持 | `.plugin-workspace/.specs/024-issue-283/exploration-report.md`、`src/sync/endpoint.rs`、`docs/issue-262-endpoint-review.md` |
| `supports` を destination 固有に残す | `.plugin-workspace/.specs/024-issue-283/exploration-report.md`、`src/sync/endpoint/destination.rs`、`src/sync.rs` |
| `TargetBinding` 集約済み | `.plugin-workspace/.specs/024-issue-283/exploration-report.md`、`src/sync/endpoint/binding.rs` |
| production `Endpoint::...` 呼び出し 0 件 | `.plugin-workspace/.specs/024-issue-283/exploration-report.md` |
| 非対称処理の対象外化 | `.plugin-workspace/.specs/024-issue-283/exploration-report.md`、`src/sync.rs`、`docs/issue-262-endpoint-review.md` |
| ISSUE-01-E/F の切り分け | `.plugin-workspace/.specs/024-issue-283/hearing-notes.md`、`.plugin-workspace/.specs/024-issue-283/exploration-report.md`、Issue #283 |

## 既存 docs との関係

- `docs/sync-endpoint-duplication-2026-05-04.md` は、`TargetBinding` 集約済み、production match-over-both 0 件、現状維持推奨という履歴判断として参照する。
- `docs/issue-255-endpoint-abstraction-response-2026-05-05.md` は、Issue #255 を将来トリガー付き追跡タスクとして扱う判断として参照する。
- `docs/issue-262-endpoint-review.md` は、`Endpoint` の `pub(super)` 維持、`supports` 非移行、非対称処理を Endpoint 化しない判断として参照する。

過去 docs の `pub(crate)` 記述は履歴情報であり、Issue #283 時点の方針では現コードの `pub(super)` を優先する。

## Issue #283 完了条件との対応

| 完了条件 | 対応セクション |
| --- | --- |
| `docs/` 配下に移行方針メモが追加されている | 本メモ全体 |
| 外部 API 維持 / 変更の判断が明記されている | 結論、外部 API 方針 |
| `Endpoint` の可視性、責務、非責務が明記されている | 結論、Endpoint の責務 |
| ISSUE-01-E / ISSUE-01-F の実装方針が具体化されている | ISSUE-01-E の実装方針、ISSUE-01-F の実装方針 |

## 検証方針

今回の成果物は docs 追加であり、必須検証は文書内容検証とする。Rust の単体テスト追加は今回の範囲外である。

必須確認:

- Issue #283 の完了条件との対応を確認する。
- 参照資料由来の必須反映論点を含める。
- Cargo feature と誤読される表現を避け、`sync モジュール内部` または `sync 機能領域` と表記する。
- Rust 実装ファイル、依存関係、ライセンス関連ファイルを変更していないことを確認する。

任意確認:

- `cargo test endpoint`
- 必要に応じて `cargo test sync`
- CLI 外形確認が必要な場合のみ、該当 test target / module path の存在確認後に CLI 関連テストを実行する。

任意確認は既存 endpoint/sync/CLI の回帰確認であり、今回の docs 追加の必須条件ではない。任意検証が失敗した場合は原則として既存状態として記録し、文書前提と矛盾する場合だけ方針メモを見直す。

今回の作業では docs 追加のみのため、Rust テストは必須ではない。任意検証を実行した場合は、実行コマンド、結果、失敗時の扱いを PR の検証欄に記録する。任意検証を実行しない場合は、docs 追加のみのため未実行であることを PR の検証欄に記録する。

Issue #283 の docs 追加時点では、任意検証として以下を実行した。

| コマンド | 結果 | 扱い |
| --- | --- | --- |
| `cargo test endpoint` | 成功。36 passed; 0 failed; 1557 filtered out。既存警告のみ。 | 文書前提と矛盾なし |
| `cargo test sync` | 成功。55 passed; 0 failed; 1538 filtered out。既存警告のみ。 | 文書前提と矛盾なし |

## Definition of Done

- `docs/` 配下に Issue #283 用の Endpoint production 移行方針メモが追加されている。
- production 内部の対象範囲を、実 CLI 経路から同期実行に到達する production code として定義している。
- 本メモが Issue #283 時点の優先方針であることを明記している。
- Endpoint production 移行判断では最初にこの文書を読むことを明記している。
- 過去 docs と本メモが Endpoint production 移行判断について矛盾する場合に限り、本メモを優先することを明記している。
- `sync(&SyncSource, &SyncDestination, &SyncOptions)` を維持する判断を明記している。
- `Endpoint` を `pub(super)` のまま維持する判断を明記している。
- `Endpoint` の責務を sync モジュール内部の共通 dispatch に限定し、非責務を明記している。
- `SyncDestination::supports` を destination 固有メソッドに残し、`Endpoint::supports` を追加しない判断を明記している。
- ISSUE-01-E と ISSUE-01-F の実装方針を切り分けている。
- Endpoint 化の発火条件、同一責務の判定条件、分岐箇所の数え方、独立した分岐箇所の定義を明記している。
- `execute_create` / `execute_update` のような source/destination 非対称処理を Endpoint 化対象外と明記している。
- 文書内容検証と任意確認コマンドの位置づけを明記している。
- 主要判断ごとの参照元を示す「判断材料」セクションを含めている。
- Issue #283 完了条件との対応表を含めている。
